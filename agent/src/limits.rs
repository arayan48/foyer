//! Limites processeur, cœurs et mémoire par serveur, sans conteneur.
//! Windows : un Job Object par serveur. Linux : un cgroup v2 par serveur (agent lancé en root).

pub struct Limits {
    /// part maximale de tout le processeur de la machine, 0 = pas de limite
    pub cpu_percent: u8,
    /// cœurs autorisés, vide = tous
    pub cores: Vec<u16>,
    /// plafond mémoire en Mo
    pub mem_mb: Option<u32>,
}

impl Limits {
    pub fn is_empty(&self) -> bool {
        self.cpu_percent == 0 && self.cores.is_empty() && self.mem_mb.is_none()
    }
}

/// Nombre de cœurs logiques vus par l'agent.
pub fn cores() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::Limits;
    use anyhow::{Context, Result};
    use std::fs;
    use std::path::{Path, PathBuf};

    const ROOT: &str = "/sys/fs/cgroup";

    fn write(p: &Path, v: &str) -> Result<()> {
        fs::write(p, v).with_context(|| format!("{} ← {v}", p.display()))
    }

    /// Prépare le cgroup du serveur et renvoie le fichier où inscrire le processus.
    pub fn prepare(id: &str, l: &Limits) -> Result<fs::File> {
        let parent = PathBuf::from(ROOT).join("foyer");
        let dir = parent.join(id);
        fs::create_dir_all(&dir).context("création du cgroup (l'agent doit tourner en root)")?;
        write(&Path::new(ROOT).join("cgroup.subtree_control"), "+cpu +cpuset +memory")?;
        write(&parent.join("cgroup.subtree_control"), "+cpu +cpuset +memory")?;
        let period = 100_000u64;
        let cpu = match l.cpu_percent {
            0 => "max".to_string(),
            p => format!("{} {period}", period * p as u64 * super::cores() as u64 / 100),
        };
        write(&dir.join("cpu.max"), &cpu)?;
        let cores = l.cores.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",");
        write(&dir.join("cpuset.cpus"), &cores)?;
        write(&dir.join("memory.max"), &l.mem_mb.map_or("max".into(), |m| (m as u64 * 1024 * 1024).to_string()))?;
        fs::OpenOptions::new().write(true).open(dir.join("cgroup.procs")).context("cgroup.procs")
    }

    /// Supprime le cgroup d'un serveur (vide une fois le processus arrêté).
    pub fn remove(id: &str) {
        let _ = fs::remove_dir(PathBuf::from(ROOT).join("foyer").join(id));
    }
}

#[cfg(windows)]
pub mod windows {
    use super::Limits;
    use anyhow::{bail, Result};
    use std::ffi::c_void;

    type Handle = *mut c_void;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateJobObjectW(attrs: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, len: u32) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn CloseHandle(h: Handle) -> i32;
    }

    #[repr(C)]
    #[derive(Default)]
    struct Basic {
        per_process_user_time: i64,
        per_job_user_time: i64,
        flags: u32,
        min_working_set: usize,
        max_working_set: usize,
        active_processes: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct Extended {
        basic: Basic,
        io: [u64; 6],
        process_memory: usize,
        job_memory: usize,
        peak_process_memory: usize,
        peak_job_memory: usize,
    }

    #[repr(C)]
    struct CpuRate {
        flags: u32,
        rate: u32,
    }

    const EXTENDED_LIMIT: i32 = 9;
    const CPU_RATE: i32 = 15;
    const LIMIT_AFFINITY: u32 = 0x10;
    const LIMIT_JOB_MEMORY: u32 = 0x200;
    const RATE_ENABLE: u32 = 0x1;
    const RATE_HARD_CAP: u32 = 0x4;

    /// Place le processus dans un Job Object limité. Ses enfants (txAdmin → FXServer) y restent.
    pub fn apply(process: Handle, l: &Limits) -> Result<()> {
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                bail!("CreateJobObject : {}", std::io::Error::last_os_error());
            }
            let r = set(job, process, l);
            // le job reste vivant tant qu'il contient des processus
            CloseHandle(job);
            r
        }
    }

    unsafe fn set(job: Handle, process: Handle, l: &Limits) -> Result<()> {
        let mut ext = Extended::default();
        if !l.cores.is_empty() {
            ext.basic.flags |= LIMIT_AFFINITY;
            ext.basic.affinity = l.cores.iter().filter(|&&c| c < 64).fold(0usize, |m, &c| m | 1 << c);
        }
        if let Some(mb) = l.mem_mb {
            ext.basic.flags |= LIMIT_JOB_MEMORY;
            ext.job_memory = mb as usize * 1024 * 1024;
        }
        if ext.basic.flags != 0
            && SetInformationJobObject(job, EXTENDED_LIMIT, &ext as *const _ as _, std::mem::size_of::<Extended>() as u32) == 0
        {
            bail!("limites cœurs/mémoire : {}", std::io::Error::last_os_error());
        }
        if l.cpu_percent > 0 {
            let rate = CpuRate { flags: RATE_ENABLE | RATE_HARD_CAP, rate: l.cpu_percent as u32 * 100 };
            if SetInformationJobObject(job, CPU_RATE, &rate as *const _ as _, std::mem::size_of::<CpuRate>() as u32) == 0 {
                bail!("limite processeur : {}", std::io::Error::last_os_error());
            }
        }
        if AssignProcessToJobObject(job, process) == 0 {
            bail!("AssignProcessToJobObject : {}", std::io::Error::last_os_error());
        }
        Ok(())
    }
}

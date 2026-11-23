//! Import d'un serveur Minecraft existant : logiciel, version, port et RAM devinés depuis ses fichiers.
use crate::software::{self, Software};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Default, Debug, PartialEq)]
pub struct Found {
    pub software: Option<Software>,
    pub version: Option<String>,
    pub port: Option<u16>,
    pub ram_mb: Option<u32>,
}

pub fn detect(dir: &Path) -> Found {
    let (software, version) = soft(dir);
    let props = std::fs::read_to_string(dir.join("server.properties")).unwrap_or_default();
    let port = props.lines().find_map(|l| l.strip_prefix("server-port=")).and_then(|p| p.trim().parse().ok());
    Found { software, version, port, ram_mb: ram(dir) }
}

/// Noms des sous-dossiers, triés.
fn subdirs(p: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(p).into_iter().flatten().flatten().filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    v.sort_by_key(|x| software::key(x));
    v
}

fn files(p: &Path) -> Vec<String> {
    std::fs::read_dir(p).into_iter().flatten().flatten().filter(|e| e.path().is_file()).map(|e| e.file_name().to_string_lossy().into_owned()).collect()
}

fn soft(dir: &Path) -> (Option<Software>, Option<String>) {
    let lib = dir.join("libraries/net");
    // version du jeu extraite par le serveur lui-même (Vanilla, Paper 1.18+, Fabric…)
    let versions = subdirs(&dir.join("versions")).pop();
    if let Some(v) = subdirs(&lib.join("neoforged/neoforge")).pop() {
        return (Some(Software::Neoforge), software::neo_mc(&v).or(versions));
    }
    // NeoForge 1.20.1 se présentait encore comme Forge
    for d in ["minecraftforge/forge", "neoforged/forge"] {
        if let Some(v) = subdirs(&lib.join(d)).pop() {
            return (Some(Software::Forge), v.split('-').next().map(str::to_string));
        }
    }
    let root = files(dir);
    let jar = |p: &str| root.iter().find(|f| f.to_lowercase().starts_with(p) && f.ends_with(".jar")).cloned();
    // ancien Forge (1.12.2, 1.16.5) : forge-1.16.5-36.2.39.jar à la racine
    if let Some(f) = jar("forge-") {
        return (Some(Software::Forge), f.split('-').nth(1).map(str::to_string));
    }
    if dir.join(".fabric").is_dir() || lib.join("fabricmc").is_dir() || jar("fabric-server").is_some() {
        let mc = files(&dir.join(".fabric/server")).iter().find_map(|f| f.strip_suffix("-server.jar").map(|v| v.rsplit('-').next().unwrap_or(v).to_string()));
        return (Some(Software::Fabric), mc.or(versions));
    }
    // « git-Paper-123 (MC: 1.21.4) »
    let hist = std::fs::read_to_string(dir.join("version_history.json")).unwrap_or_default();
    let hist_mc = hist.split("(MC: ").nth(1).and_then(|s| s.split(')').next()).map(str::to_string);
    for (sw, p) in [(Software::Purpur, "purpur"), (Software::Paper, "paper")] {
        if hist.to_lowercase().contains(&format!("-{p}-")) || jar(p).is_some() {
            let from_jar = jar(p).and_then(|f| f.split('-').nth(1).map(|v| v.trim_end_matches(".jar").to_string()));
            return (Some(sw), hist_mc.or(versions).or(from_jar));
        }
    }
    match versions {
        Some(v) => (Some(Software::Vanilla), Some(v)),
        None => (None, None),
    }
}

/// -Xmx des scripts de lancement (user_jvm_args.txt, .bat, .sh, .cmd).
fn ram(dir: &Path) -> Option<u32> {
    files(dir).iter().filter(|f| f == &"user_jvm_args.txt" || [".bat", ".sh", ".cmd"].iter().any(|e| f.ends_with(e))).find_map(|f| {
        let t = std::fs::read_to_string(dir.join(f)).ok()?;
        t.lines().filter(|l| !l.trim_start().starts_with('#')).find_map(|l| {
            let v = l.split("-Xmx").nth(1)?;
            let n: String = v.chars().take_while(char::is_ascii_digit).collect();
            let n: u32 = n.parse().ok()?;
            match v[n.to_string().len()..].chars().next()? {
                'G' | 'g' => Some(n * 1024),
                'M' | 'm' => Some(n),
                _ => None,
            }
        })
    })
}

/// Copie récursive ; le dossier d'origine reste intact. Les liens sont ignorés.
pub fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let (e, t) = { let e = e?; let t = e.file_type()?; (e, t) };
        let dst = to.join(e.file_name());
        if t.is_dir() {
            copy(&e.path(), &dst)?;
        } else if t.is_file() {
            std::fs::copy(e.path(), dst)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("foyer-import-{name}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn neoforge() {
        let d = tmp("neo");
        std::fs::create_dir_all(d.join("libraries/net/neoforged/neoforge/21.1.77")).unwrap();
        std::fs::write(d.join("server.properties"), "motd=x\nserver-port=25570\n").unwrap();
        std::fs::write(d.join("user_jvm_args.txt"), "# -Xmx4G\n-Xmx10G\n").unwrap();
        assert_eq!(detect(&d), Found { software: Some(Software::Neoforge), version: Some("1.21.1".into()), port: Some(25570), ram_mb: Some(10240) });
    }

    #[test]
    fn forge_et_paper() {
        let d = tmp("forge");
        std::fs::create_dir_all(d.join("libraries/net/minecraftforge/forge/1.20.1-47.3.0")).unwrap();
        assert_eq!(soft(&d), (Some(Software::Forge), Some("1.20.1".into())));
        let d = tmp("forge-old");
        std::fs::write(d.join("forge-1.16.5-36.2.39.jar"), "").unwrap();
        std::fs::write(d.join("start.bat"), "java -Xms2G -Xmx6144M -jar forge.jar").unwrap();
        assert_eq!(soft(&d), (Some(Software::Forge), Some("1.16.5".into())));
        assert_eq!(ram(&d), Some(6144));
        let d = tmp("paper");
        std::fs::write(d.join("version_history.json"), r#"{"currentVersion":"git-Paper-123 (MC: 1.21.4)"}"#).unwrap();
        assert_eq!(soft(&d), (Some(Software::Paper), Some("1.21.4".into())));
        let d = tmp("vide");
        assert_eq!(soft(&d), (None, None));
    }
}

//! Jeux laissés par un agent coupé sans arrêt propre : ils gardent le port et verrouillent leurs fichiers.
use std::path::Path;
use std::time::Duration;
use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, Signal, System, UpdateKind};

/// Programme lancé par Foyer (rangé dans `data`) qui travaille dans un dossier de serveur.
fn is_game(p: &Process, data: &Path) -> bool {
    p.thread_kind().is_none() && p.exe().is_some_and(|e| e.starts_with(data)) && p.cwd().is_some_and(|c| c.starts_with(data.join("servers")))
}

/// À appeler avant tout démarrage de jeu : l'agent n'en contrôle encore aucun.
pub async fn stop(data: &Path) {
    // Linux donne des chemins sans liens symboliques ; Windows non, et canonicalize y ajouterait « \\?\ »
    let data = if cfg!(unix) { std::fs::canonicalize(data).unwrap_or_else(|_| data.into()) } else { data.into() };
    let kind = ProcessRefreshKind::nothing().with_exe(UpdateKind::Always).with_cwd(UpdateKind::Always);
    let mut sys = System::new();
    for i in 0..40 {
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
        let found: Vec<Pid> = sys.processes().values().filter(|p| is_game(p, &data)).map(|p| p.pid()).collect();
        if found.is_empty() {
            return;
        }
        for p in found.iter().filter_map(|pid| sys.process(*pid)) {
            if i == 0 {
                crate::journal::agent(&format!("jeu orphelin arrêté : pid {} dans {}", p.pid(), p.cwd().unwrap_or(&data).display()));
            }
            // SIGTERM laisse Minecraft sauver le monde ; Windows n'a pas d'équivalent sans console
            match i {
                0 if p.kill_with(Signal::Term) == Some(true) => {}
                0 | 30 => { p.kill(); }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    crate::journal::agent("un jeu orphelin refuse de s'arrêter");
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[tokio::test]
    async fn orphelin_arrete() {
        let data = std::env::temp_dir().join("foyer-test-orphelin");
        let dir = data.join("servers").join("a1");
        std::fs::create_dir_all(&dir).unwrap();
        let exe = data.join("sleep");
        std::fs::copy("/bin/sleep", &exe).unwrap();
        // un autre test qui lance un programme pendant la copie peut garder le fichier ouvert (« Text file busy »)
        let mut game = loop {
            match tokio::process::Command::new(&exe).arg("60").current_dir(&dir).spawn() {
                Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy => std::thread::sleep(std::time::Duration::from_millis(20)),
                r => break r.unwrap(),
            }
        };
        // même dossier, mais pas un programme de Foyer : intouchable
        let mut other = tokio::process::Command::new("/bin/sleep").arg("60").current_dir(&dir).spawn().unwrap();
        let (_, st) = tokio::join!(super::stop(&data), game.wait());
        assert_eq!(std::os::unix::process::ExitStatusExt::signal(&st.unwrap()), Some(15));
        assert!(other.try_wait().unwrap().is_none());
        other.kill().await.unwrap();
        std::fs::remove_dir_all(&data).ok();
    }
}

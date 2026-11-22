//! Sauvegardes : `backups/<serveur>/<id>.tar.gz` et sa fiche `<id>.json`. `id` = secondes Unix.
use crate::{games, journal, logger, save, App, Err, Status, S};
use anyhow::{anyhow, bail, Context, Result};
use axum::extract::{Path as UrlPath, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;

/// Sauvegardes automatiques gardées par serveur ; les manuelles ne sont jamais effacées seules.
const KEEP_AUTO: usize = 5;

#[derive(Serialize, Deserialize, Clone)]
pub struct Backup {
    id: u64,
    auto: bool,
    reason: String,
    version: Option<String>,
    java: u8,
    size: u64,
}

fn folder(data: &Path, id: &str) -> PathBuf {
    data.join("backups").join(id)
}

async fn all(dir: &Path) -> Vec<Backup> {
    let mut out: Vec<Backup> = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(e)) = rd.next_entry().await {
            if e.path().extension().is_some_and(|x| x == "json") {
                out.extend(tokio::fs::read(e.path()).await.ok().and_then(|b| serde_json::from_slice(&b).ok()));
            }
        }
    }
    out.sort_by_key(|b| std::cmp::Reverse(b.id));
    out
}

async fn erase(dir: &Path, id: u64) -> std::io::Result<()> {
    tokio::fs::remove_file(dir.join(format!("{id}.tar.gz"))).await?;
    tokio::fs::remove_file(dir.join(format!("{id}.json"))).await
}

async fn tar(args: &[&std::ffi::OsStr], tolerate_changes: bool) -> Result<()> {
    let out = tokio::process::Command::new("tar").args(args).kill_on_drop(true).output().await?;
    // GNU tar renvoie 1 quand un fichier a changé pendant la lecture, sans conséquence à chaud
    if out.status.success() || (cfg!(target_os = "linux") && tolerate_changes && out.status.code() == Some(1)) {
        return Ok(());
    }
    bail!("{}", String::from_utf8_lossy(&out.stderr).trim().lines().last().unwrap_or("échec de l'archive"))
}

/// Archive un serveur que personne d'autre ne modifie (arrêté, ou monde figé par `freeze`).
pub async fn create(app: &App, id: &str, reason: &str, auto: bool, hot: bool) -> Result<()> {
    let s = app.servers.lock().await.iter().find(|s| s.id == id).cloned().ok_or_else(|| anyhow!("serveur inconnu"))?;
    let from = app.data.join("servers").join(id);
    let dir = folder(&app.data, id);
    tokio::fs::create_dir_all(&dir).await?;
    let mut at = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    while dir.join(format!("{at}.json")).exists() {
        at += 1;
    }
    let part = dir.join("sauvegarde.part");
    let pid = || async { app.live.lock().await.get(id).and_then(|l| l.pid) };
    let before = pid().await;
    let o = std::ffi::OsStr::new;
    let r = tar(&[o("-czf"), part.as_os_str(), o("--exclude"), o("session.lock"), o("-C"), from.as_os_str(), o(".")], hot).await;
    let r = match r {
        Ok(()) if pid().await != before => Err(anyhow!("le serveur s'est arrêté ou relancé pendant la sauvegarde")),
        r => r,
    };
    if let Err(e) = r {
        tokio::fs::remove_file(&part).await.ok();
        return Err(e.context("sauvegarde impossible"));
    }
    let size = tokio::fs::metadata(&part).await?.len();
    tokio::fs::rename(&part, dir.join(format!("{at}.tar.gz"))).await?;
    let b = Backup { id: at, auto, reason: reason.into(), version: s.version, java: s.java, size };
    tokio::fs::write(dir.join(format!("{at}.json")), serde_json::to_vec_pretty(&b)?).await?;
    for old in all(&dir).await.iter().filter(|b| b.auto).skip(KEEP_AUTO) {
        erase(&dir, old.id).await.ok();
    }
    Ok(())
}

async fn command(app: &App, id: &str, cmd: &str) -> Result<()> {
    let mut live = app.live.lock().await;
    let stdin = live.get_mut(id).and_then(|l| l.stdin.as_mut()).ok_or_else(|| anyhow!("serveur arrêté"))?;
    Ok(stdin.write_all(format!("{cmd}\n").as_bytes()).await?)
}

/// Le début du message, pas la ligne : un joueur qui écrit « Saved the game » ne compte pas.
fn saved(line: &str) -> bool {
    line.split_once("]: ").is_some_and(|(_, m)| m.starts_with("Saved the "))
}

/// Minecraft : fige les écritures du monde et attend que tout soit sur le disque.
async fn freeze(app: &App, id: &str) -> Result<()> {
    let mut rx = app.live.lock().await.get(id).ok_or_else(|| anyhow!("serveur inconnu"))?.tx.subscribe();
    command(app, id, "save-off").await?;
    command(app, id, "save-all flush").await?;
    let wait = async {
        loop {
            match rx.recv().await {
                Ok(l) if saved(&l) => return Ok(()),
                Err(tokio::sync::broadcast::error::RecvError::Closed) => bail!("serveur arrêté"),
                _ => {}
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(120), wait).await.context("le monde ne se sauvegarde pas")?
}

/// Réserve le serveur. Renvoie vrai s'il tourne (sauvegarde à chaud).
async fn claim(app: &App, id: &str, hot_ok: bool) -> Result<bool, Err> {
    let mut live = app.live.lock().await;
    let l = live.get_mut(id).ok_or((StatusCode::NOT_FOUND, "serveur inconnu".into()))?;
    if l.backing {
        return Err((StatusCode::CONFLICT, "Une sauvegarde est déjà en cours".into()));
    }
    let hot = match l.status {
        Status::Stopped | Status::Error => false,
        Status::Running if hot_ok => true,
        _ => return Err((StatusCode::CONFLICT, "Arrête le serveur d'abord".into())),
    };
    if !hot {
        l.status = Status::Busy;
    }
    l.backing = true;
    l.backup_error = None;
    Ok(hot)
}

async fn release(app: &App, id: &str, r: Result<String>) {
    let mut live = app.live.lock().await;
    let Some(l) = live.get_mut(id) else { return };
    l.backing = false;
    if l.status == Status::Busy {
        l.status = Status::Stopped;
    }
    match r {
        Ok(m) => l.push(format!("[foyer] {m}")),
        Err(e) => {
            l.push(format!("[foyer] Erreur : {e:#}"));
            l.backup_error = Some(format!("{e:#}"));
            journal::agent(&format!("sauvegarde de {id} : {e:#}"));
        }
    }
}

pub async fn list(State(app): S, UrlPath(id): UrlPath<String>) -> Result<Json<Value>, Err> {
    let (running, error) = app.live.lock().await.get(&id).map(|l| (l.backing, l.backup_error.clone())).ok_or((StatusCode::NOT_FOUND, "serveur inconnu".into()))?;
    Ok(Json(json!({ "backups": all(&folder(&app.data, &id)).await, "running": running, "error": error })))
}

pub async fn add(State(app): S, UrlPath(id): UrlPath<String>) -> Result<StatusCode, Err> {
    let hot = claim(&app, &id, true).await?;
    tokio::spawn(async move {
        let mc = app.servers.lock().await.iter().any(|s| s.id == id && s.game == games::Game::Minecraft);
        let r = async {
            if hot && mc {
                freeze(&app, &id).await?;
            }
            create(&app, &id, "Manuelle", false, hot).await
        }
        .await;
        if hot && mc {
            command(&app, &id, "save-on").await.ok();
        }
        release(&app, &id, r.map(|()| "Sauvegarde terminée.".into())).await;
    });
    Ok(StatusCode::NO_CONTENT)
}

async fn find(app: &App, id: &str, b: u64) -> Result<Backup, Err> {
    if !app.servers.lock().await.iter().any(|s| s.id == id) {
        return Err((StatusCode::NOT_FOUND, "serveur inconnu".into()));
    }
    all(&folder(&app.data, id)).await.into_iter().find(|x| x.id == b).ok_or((StatusCode::NOT_FOUND, "sauvegarde introuvable".into()))
}

pub async fn delete(State(app): S, UrlPath((id, b)): UrlPath<(String, u64)>) -> Result<StatusCode, Err> {
    find(&app, &id, b).await?;
    if app.live.lock().await.get(&id).is_some_and(|l| l.backing) {
        return Err((StatusCode::CONFLICT, "Une sauvegarde est en cours".into()));
    }
    erase(&folder(&app.data, &id), b).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore(State(app): S, UrlPath((id, b)): UrlPath<(String, u64)>) -> Result<StatusCode, Err> {
    let b = find(&app, &id, b).await?;
    claim(&app, &id, false).await?;
    tokio::spawn(async move {
        let r = put_back(&app, &id, &b).await;
        release(&app, &id, r.map(|()| "Sauvegarde restaurée.".into())).await;
    });
    Ok(StatusCode::NO_CONTENT)
}

/// Extrait à côté, sauvegarde l'état actuel, puis échange les dossiers.
async fn put_back(app: &Arc<App>, id: &str, b: &Backup) -> Result<()> {
    let log = logger(app, id);
    let dir = folder(&app.data, id);
    let live = app.data.join("servers").join(id);
    let (new, old) = (dir.join("restore"), dir.join("old"));
    tokio::fs::remove_dir_all(&new).await.ok();
    log("Extraction de la sauvegarde…".into());
    tokio::fs::create_dir_all(&new).await?;
    let o = std::ffi::OsStr::new;
    let r = async {
        tar(&[o("-xzf"), dir.join(format!("{}.tar.gz", b.id)).as_os_str(), o("-C"), new.as_os_str()], false).await.context("sauvegarde illisible")?;
        // le port a pu changer depuis : celui du serveur fait foi (pare-feu, conflits)
        let port = app.servers.lock().await.iter().find(|s| s.id == id && s.game == games::Game::Minecraft).map(|s| s.port.to_string());
        if let Some(p) = port {
            let file = new.join("server.properties");
            let text = games::read_props(&file).await?;
            tokio::fs::write(&file, games::set_props(&text, &[("server-port", &p), ("query.port", &p)])).await?;
        }
        if live.exists() {
            log("Sauvegarde de l'état actuel…".into());
            create(app, id, "Avant restauration", true, false).await?;
            move_dir(&live, &old).await.context("un fichier du serveur est ouvert ailleurs")?;
        }
        if let Err(e) = move_dir(&new, &live).await {
            move_dir(&old, &live).await.ok();
            return Err(e.into());
        }
        anyhow::Ok(())
    }
    .await;
    tokio::fs::remove_dir_all(&new).await.ok();
    r?;
    tokio::fs::remove_dir_all(&old).await.ok();
    let mut servers = app.servers.lock().await;
    if let Some(s) = servers.iter_mut().find(|s| s.id == id) {
        s.version = b.version.clone();
        s.java = b.java;
    }
    save(app, &servers).await;
    Ok(())
}

/// Windows garde parfois un fichier ouvert un instant après l'arrêt.
async fn move_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    for _ in 0..5 {
        if tokio::fs::rename(from, to).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    tokio::fs::rename(from, to).await
}

/// Au démarrage : termine ou annule une restauration coupée, efface les archives incomplètes.
pub async fn recover(data: &Path) {
    let Ok(mut rd) = tokio::fs::read_dir(data.join("backups")).await else { return };
    while let Ok(Some(e)) = rd.next_entry().await {
        let (dir, live) = (e.path(), data.join("servers").join(e.file_name()));
        if dir.join("old").exists() {
            if live.exists() {
                tokio::fs::remove_dir_all(dir.join("old")).await.ok();
            } else if let Err(e) = tokio::fs::rename(dir.join("old"), &live).await {
                journal::agent(&format!("restauration coupée de {} : {e}", live.display()));
                continue;
            }
        }
        tokio::fs::remove_dir_all(dir.join("restore")).await.ok();
        tokio::fs::remove_file(dir.join("sauvegarde.part")).await.ok();
    }
}

/// Serveur effacé définitivement.
pub async fn forget(data: &Path, id: &str) {
    tokio::fs::remove_dir_all(folder(data, id)).await.ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Live, Server};

    #[test]
    fn message_de_sauvegarde() {
        assert!(saved("[20:00:00] [Server thread/INFO]: Saved the game") && saved("[20:00:00 INFO]: Saved the world"));
        assert!(!saved("[20:00:00 INFO]: <Bob> Saved the game"));
    }

    #[tokio::test]
    async fn sauver_casser_restaurer() {
        let data = std::env::temp_dir().join(format!("foyer-b-{:x}", rand::random::<u64>()));
        let dir = data.join("servers/abc");
        std::fs::create_dir_all(dir.join("world")).unwrap();
        std::fs::create_dir_all(dir.join("plugins/p/logs")).unwrap();
        std::fs::write(dir.join("plugins/p/logs/y"), "x").unwrap();
        std::fs::write(dir.join("world/niveau é"), "bon").unwrap();
        std::fs::write(dir.join("world/session.lock"), "x").unwrap();
        std::fs::write(dir.join("server.properties"), "motd=a\nserver-port=1\n").unwrap();
        let s: Server = serde_json::from_str(r#"{"id":"abc","name":"x","game":"minecraft","port":25570,"ram_mb":1024,"version":"Paper 1.10.2","java":8}"#).unwrap();
        let app = Arc::new(App { data: data.clone(), token: String::new(), servers: vec![s].into(), live: Default::default(), failures: Default::default(), host: Default::default() });
        app.live.lock().await.insert("abc".into(), Live::new(Status::Stopped));
        let bdir = folder(&data, "abc");

        create(&app, "abc", "Manuelle", false, false).await.unwrap();
        let b = all(&bdir).await.remove(0);
        std::fs::write(dir.join("world/niveau é"), "cassé").unwrap();
        {
            let mut sv = app.servers.lock().await;
            sv[0].version = Some("Vanilla 26.3".into());
            sv[0].java = 25;
        }
        put_back(&app, "abc", &b).await.unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("world/niveau é")).unwrap(), "bon");
        assert!(dir.join("plugins/p/logs/y").exists() && !dir.join("world/session.lock").exists() && !bdir.join("old").exists() && !bdir.join("restore").exists());
        assert!(std::fs::read_to_string(dir.join("server.properties")).unwrap().contains("server-port=25570"));
        assert_eq!(app.servers.lock().await[0].version.as_deref(), Some("Paper 1.10.2"));
        // l'état cassé est récupérable
        let undo = all(&bdir).await.into_iter().find(|b| b.auto).unwrap();
        put_back(&app, "abc", &undo).await.unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("world/niveau é")).unwrap(), "cassé");
        assert_eq!(app.servers.lock().await[0].java, 25);

        // archive abîmée : rien ne bouge
        std::fs::write(bdir.join(format!("{}.tar.gz", b.id)), "rien").unwrap();
        assert!(put_back(&app, "abc", &b).await.is_err());
        assert_eq!(std::fs::read_to_string(dir.join("world/niveau é")).unwrap(), "cassé");

        // automatiques limitées, manuelle gardée
        for _ in 0..KEEP_AUTO + 2 {
            create(&app, "abc", "Auto", true, false).await.unwrap();
        }
        let l = all(&bdir).await;
        assert_eq!((l.iter().filter(|b| b.auto).count(), l.iter().filter(|b| !b.auto).count()), (KEEP_AUTO, 1));

        // réservé : pas deux à la fois, pas de démarrage possible
        assert!(!claim(&app, "abc", true).await.unwrap());
        assert!(claim(&app, "abc", true).await.is_err());
        assert!(crate::start(&app, "abc").await.is_err());
        release(&app, "abc", Ok(String::new())).await;

        // coupure entre les deux renommages
        std::fs::rename(&dir, bdir.join("old")).unwrap();
        recover(&data).await;
        assert!(dir.join("server.properties").exists());
        std::fs::remove_dir_all(data).unwrap();
    }
}

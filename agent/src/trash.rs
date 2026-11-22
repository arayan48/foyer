//! Corbeille : un serveur supprimé attend dans `trash/<id>` (server.json + files) avant effacement.
use crate::{firewall, save, App, Err, Live, Server, Status, S};
use axum::extract::{Path as UrlPath, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEEP_DAYS: u64 = 30;

#[derive(Serialize, Deserialize)]
pub struct Trashed {
    #[serde(flatten)]
    server: Server,
    /// secondes Unix
    deleted_at: u64,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn fail(e: impl std::fmt::Display) -> Err {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Corbeille : {e}"))
}

/// L'id vient de l'URL : lettres et chiffres seulement.
fn slot(app: &App, id: &str) -> Result<PathBuf, Err> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err((StatusCode::NOT_FOUND, "absent de la corbeille".into()));
    }
    Ok(app.data.join("trash").join(id))
}

async fn load(app: &App, id: &str) -> Result<Trashed, Err> {
    let b = tokio::fs::read(slot(app, id)?.join("server.json")).await.map_err(|_| (StatusCode::NOT_FOUND, "absent de la corbeille".into()))?;
    serde_json::from_slice(&b).map_err(fail)
}

/// Met le dossier d'un serveur arrêté à la corbeille ; en cas d'échec rien n'a bougé.
pub async fn put(app: &App, s: &Server) -> Result<(), Err> {
    let slot = slot(app, &s.id)?;
    tokio::fs::create_dir_all(&slot).await.map_err(fail)?;
    let json = serde_json::to_vec_pretty(&Trashed { server: s.clone(), deleted_at: now() }).map_err(fail)?;
    tokio::fs::write(slot.join("server.json"), json).await.map_err(fail)?;
    let from = app.data.join("servers").join(&s.id);
    // Windows garde parfois un fichier ouvert un instant après l'arrêt
    for attempt in 0..6 {
        match tokio::fs::rename(&from, slot.join("files")).await {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()), // installation ratée : aucun dossier
            Err(e) if attempt == 5 => {
                let _ = tokio::fs::remove_file(slot.join("server.json")).await;
                let _ = tokio::fs::remove_dir(&slot).await;
                return Err(fail(e));
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(500)).await,
        }
    }
    Ok(())
}

async fn all(app: &App) -> Vec<Trashed> {
    let mut out = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(app.data.join("trash")).await {
        while let Ok(Some(e)) = rd.next_entry().await {
            if let Ok(t) = load(app, &e.file_name().to_string_lossy()).await {
                out.push(t);
            }
        }
    }
    out.sort_by_key(|t| std::cmp::Reverse(t.deleted_at));
    out
}

pub async fn list(State(app): S) -> Json<Vec<Trashed>> {
    Json(all(&app).await)
}

pub async fn restore(State(app): S, UrlPath(id): UrlPath<String>) -> Result<Json<Server>, Err> {
    let mut servers = app.servers.lock().await;
    let s = load(&app, &id).await?.server;
    if servers.iter().any(|o| o.id == id) {
        return Err((StatusCode::CONFLICT, "Ce serveur existe déjà".into()));
    }
    let used: Vec<u16> = servers.iter().flat_map(|o| o.ports()).map(|p| p.0).collect();
    if s.ports().iter().any(|p| used.contains(&p.0)) {
        return Err((StatusCode::CONFLICT, "Port déjà utilisé par un autre serveur".into()));
    }
    let slot = slot(&app, &id)?;
    if slot.join("files").exists() {
        tokio::fs::rename(slot.join("files"), app.data.join("servers").join(&id)).await.map_err(fail)?;
    }
    servers.push(s.clone());
    save(&app, &servers).await;
    app.live.lock().await.insert(id, Live::new(Status::Stopped));
    let _ = tokio::fs::remove_dir_all(slot).await;
    drop(servers);
    firewall(&s, true).await;
    Ok(Json(s))
}

/// Effacement définitif.
pub async fn purge(State(app): S, UrlPath(id): UrlPath<String>) -> Result<StatusCode, Err> {
    let _servers = app.servers.lock().await; // pas en même temps qu'une restauration
    load(&app, &id).await?;
    tokio::fs::remove_dir_all(slot(&app, &id)?).await.map_err(fail)?;
    crate::backups::forget(&app.data, &id).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn purge_old(app: &App, now: u64) {
    let servers = app.servers.lock().await;
    for t in all(app).await {
        let old = now.saturating_sub(t.deleted_at) > KEEP_DAYS * 86400;
        // encore dans la liste = suppression interrompue : on n'efface pas
        if old && !servers.iter().any(|s| s.id == t.server.id) {
            if let Ok(p) = slot(app, &t.server.id) {
                let _ = tokio::fs::remove_dir_all(p).await;
                crate::backups::forget(&app.data, &t.server.id).await;
            }
        }
    }
}

pub async fn purge_loop(app: Arc<App>) {
    loop {
        purge_old(&app, now()).await;
        tokio::time::sleep(Duration::from_secs(6 * 3600)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn supprimer_restaurer_expirer() {
        let data = std::env::temp_dir().join(format!("foyer-t-{:x}", rand::random::<u64>()));
        let dir = data.join("servers/abc");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("monde"), "x").unwrap();
        let s: Server = serde_json::from_str(r#"{"id":"abc","name":"x","game":"minecraft","port":25565,"ram_mb":1024}"#).unwrap();
        let app = Arc::new(App { data: data.clone(), token: String::new(), servers: vec![s].into(), live: Default::default(), failures: Default::default(), host: Default::default() });
        app.live.lock().await.insert("abc".into(), Live::new(Status::Stopped));
        let del = || crate::remove(State(app.clone()), UrlPath("abc".into()));

        // installation en cours : refus
        app.live.lock().await.get_mut("abc").unwrap().status = Status::Installing;
        assert!(del().await.is_err());
        app.live.lock().await.get_mut("abc").unwrap().status = Status::Stopped;

        del().await.unwrap();
        assert!(!dir.exists() && app.servers.lock().await.is_empty());
        assert_eq!(all(&app).await.len(), 1);

        // port repris entre-temps : refus, rien ne bouge
        let mut other = all(&app).await.remove(0).server;
        other.id = "def".into();
        app.servers.lock().await.push(other);
        assert!(restore(State(app.clone()), UrlPath("abc".into())).await.is_err());
        app.servers.lock().await.clear();

        assert!(restore(State(app.clone()), UrlPath("abc".into())).await.is_ok());
        assert_eq!(std::fs::read_to_string(dir.join("monde")).unwrap(), "x");
        assert!(all(&app).await.is_empty());

        del().await.unwrap();
        purge_old(&app, now() + 29 * 86400).await;
        assert_eq!(all(&app).await.len(), 1);
        purge_old(&app, now() + 31 * 86400).await;
        assert!(all(&app).await.is_empty() && !data.join("trash/abc").exists());

        assert!(purge(State(app.clone()), UrlPath("..".into())).await.is_err());
        std::fs::remove_dir_all(data).unwrap();
    }
}

//! Journal des actions : une ligne JSON par changement demandé à l'agent.
use crate::{App, S};
use axum::extract::{ConnectInfo, Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use axum::Json;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;

const MAX_BYTES: u64 = 1024 * 1024;
const SHOWN: usize = 200;
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Un journal illisible ou un disque plein ne doit jamais bloquer une action.
pub async fn add(app: &App, ip: &str, action: &str, status: Option<u16>) {
    let at = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let line = format!("{}\n", json!({ "at": at, "ip": ip, "action": action, "status": status }));
    let path = app.data.join("actions.log");
    let _guard = LOCK.lock().await;
    if tokio::fs::metadata(&path).await.is_ok_and(|m| m.len() > MAX_BYTES) {
        let _ = tokio::fs::rename(&path, app.data.join("actions.log.1")).await;
    }
    if let Ok(mut f) = tokio::fs::OpenOptions::new().create(true).append(true).open(&path).await {
        let _ = f.write_all(line.as_bytes()).await;
        let _ = f.flush().await;
    }
}

/// Placé après l'authentification : seules les requêtes acceptées qui changent quelque chose.
pub async fn layer(State(app): S, ConnectInfo(addr): ConnectInfo<SocketAddr>, req: Request, next: Next) -> Response {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }
    let target = req.uri().path_and_query().map_or("", |p| p.as_str());
    let action = format!("{} {target}", req.method());
    let r = next.run(req).await;
    add(&app, &addr.ip().to_string(), &action, Some(r.status().as_u16())).await;
    r
}

/// Les dernières actions, la plus récente d'abord.
pub async fn list(State(app): S) -> Json<Vec<Value>> {
    let _guard = LOCK.lock().await;
    let old = tokio::fs::read_to_string(app.data.join("actions.log.1")).await.unwrap_or_default();
    let new = tokio::fs::read_to_string(app.data.join("actions.log")).await.unwrap_or_default();
    Json(new.lines().rev().chain(old.lines().rev()).filter_map(|l| serde_json::from_str(l).ok()).take(SHOWN).collect())
}

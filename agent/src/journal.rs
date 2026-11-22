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

static AGENT_LOG: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

/// Active `agent.log`, panics comprises.
pub fn init(data: &std::path::Path) {
    *AGENT_LOG.lock().unwrap_or_else(|e| e.into_inner()) = Some(data.join("agent.log"));
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |p| {
        agent(&format!("panique : {p}"));
        default(p);
    }));
}

/// Journal technique de l'agent (console + fichier). Ne jamais y mettre le jeton.
pub fn agent(msg: &str) {
    use std::io::Write;
    let at = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    // writeln et pas println : sans console, println paniquerait
    let _ = writeln!(std::io::stdout(), "{msg}");
    let guard = AGENT_LOG.lock().unwrap_or_else(|e| e.into_inner());
    let Some(path) = guard.as_ref() else { return };
    if std::fs::metadata(path).is_ok_and(|m| m.len() > MAX_BYTES) {
        let _ = std::fs::rename(path, path.with_extension("log.1"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(format!("{} {msg}\n", utc(at)).as_bytes());
    }
}

/// Secondes Unix en `2026-11-22T19:43:13Z` (calendrier civil de Howard Hinnant).
fn utc(secs: u64) -> String {
    let (z, t) = (secs / 86400 + 719_468, secs % 86400);
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + z / 146_097 * 400 + u64::from(month <= 2);
    format!("{year}-{month:02}-{:02}T{:02}:{:02}:{:02}Z", doy - (153 * mp + 2) / 5 + 1, t / 3600, t / 60 % 60, t % 60)
}

#[cfg(test)]
mod tests {
    #[test]
    fn dates_utc() {
        assert_eq!(super::utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(super::utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(super::utc(1_795_376_593), "2026-11-22T19:43:13Z");
    }
}

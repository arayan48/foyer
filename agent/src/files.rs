//! Fichiers d'un serveur : tout reste dans `servers/<id>`.
use crate::{Err, S};
use axum::body::Body;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Deserialize)]
pub struct At {
    #[serde(default)]
    path: String,
}

#[derive(Serialize)]
pub struct Entry {
    name: String,
    dir: bool,
    size: u64,
    /// secondes Unix
    modified: u64,
}

fn bad(m: &str) -> Err {
    (StatusCode::BAD_REQUEST, m.into())
}

fn io(e: std::io::Error) -> Err {
    use std::io::ErrorKind::*;
    match e.kind() {
        NotFound => (StatusCode::NOT_FOUND, "Fichier introuvable".into()),
        AlreadyExists => (StatusCode::CONFLICT, "Ce nom existe déjà".into()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// Refuse aussi ce que Windows interprète : `C:`, flux `a:b`, `NUL`, point ou espace final.
fn name_ok(n: &str) -> bool {
    let stem = n.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || ((stem.starts_with("COM") || stem.starts_with("LPT")) && stem.chars().count() == 4);
    !n.is_empty() && !n.ends_with(['.', ' ']) && !n.chars().any(|c| c.is_control() || "\\:*?\"<>|".contains(c)) && !reserved
}

/// Chemin réel sous `root`, liens symboliques suivis ; `root` lui-même si `rel` est vide.
fn resolve(root: &Path, rel: &str) -> Result<PathBuf, Err> {
    let root = root.canonicalize().map_err(io)?;
    let mut p = root.clone();
    for n in rel.split('/').filter(|n| !n.is_empty()) {
        if !name_ok(n) {
            return Err(bad("Chemin invalide"));
        }
        p.push(n);
        // symlink_metadata : un lien cassé existe et doit être refusé, pas suivi à l'écriture
        if p.symlink_metadata().is_ok() {
            p = p.canonicalize().map_err(io)?;
            if !p.starts_with(&root) {
                return Err(bad("Chemin hors du dossier du serveur"));
            }
        }
    }
    Ok(p)
}

/// Comme `resolve`, sans suivre le dernier élément (supprimer ou renommer un lien, pas sa cible).
fn resolve_entry(root: &Path, rel: &str) -> Result<PathBuf, Err> {
    let rel = rel.trim_matches('/');
    let (parent, name) = rel.rsplit_once('/').unwrap_or(("", rel));
    if !name_ok(name) {
        return Err(bad("Chemin invalide"));
    }
    Ok(resolve(root, parent)?.join(name))
}

async fn root(app: &crate::App, id: &str) -> Result<PathBuf, Err> {
    if !app.servers.lock().await.iter().any(|s| s.id == id) {
        return Err((StatusCode::NOT_FOUND, "serveur inconnu".into()));
    }
    Ok(app.data.join("servers").join(id))
}

pub async fn list(State(app): S, UrlPath(id): UrlPath<String>, Query(q): Query<At>) -> Result<Json<Vec<Entry>>, Err> {
    let dir = resolve(&root(&app, &id).await?, &q.path)?;
    let mut rd = tokio::fs::read_dir(dir).await.map_err(io)?;
    let mut out = Vec::new();
    while let Some(e) = rd.next_entry().await.map_err(io)? {
        // un fichier peut disparaître pendant la lecture (serveur en marche)
        let Ok(m) = e.metadata().await else { continue };
        let modified = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
        out.push(Entry { name: e.file_name().to_string_lossy().into_owned(), dir: m.is_dir(), size: m.len(), modified });
    }
    out.sort_by_key(|e| (!e.dir, e.name.to_lowercase()));
    Ok(Json(out))
}

pub async fn read(State(app): S, UrlPath(id): UrlPath<String>, Query(q): Query<At>) -> Result<Response, Err> {
    let path = resolve(&root(&app, &id).await?, &q.path)?;
    let m = tokio::fs::metadata(&path).await.map_err(io)?;
    if !m.is_file() {
        return Err(bad("Ce n'est pas un fichier"));
    }
    let f = tokio::fs::File::open(&path).await.map_err(io)?;
    let body = Body::from_stream(stream::try_unfold(f, |mut f| async move {
        let mut buf = vec![0; 64 * 1024];
        let n = f.read(&mut buf).await?;
        buf.truncate(n);
        Ok::<_, std::io::Error>((n > 0).then_some((buf, f)))
    }));
    // pas de Content-Length : un journal peut grandir pendant l'envoi
    Ok(([(header::CONTENT_TYPE, "application/octet-stream")], body).into_response())
}

/// Efface le fichier temporaire si l'envoi échoue ou si le client coupe.
struct Tmp(PathBuf);
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Écrit ou remplace un fichier, d'un seul coup une fois tout reçu.
pub async fn write(State(app): S, UrlPath(id): UrlPath<String>, Query(q): Query<At>, body: Body) -> Result<StatusCode, Err> {
    let path = resolve(&root(&app, &id).await?, &q.path)?;
    if path.is_dir() {
        return Err(bad("C'est un dossier"));
    }
    let dir = app.data.join("tmp");
    tokio::fs::create_dir_all(&dir).await.map_err(io)?;
    let tmp = Tmp(dir.join(format!("{:016x}", rand::random::<u64>())));
    let mut f = tokio::fs::File::create(&tmp.0).await.map_err(io)?;
    let mut chunks = body.into_data_stream();
    while let Some(c) = chunks.next().await {
        f.write_all(&c.map_err(|e| bad(&format!("Envoi interrompu : {e}")))?).await.map_err(io)?;
    }
    f.sync_all().await.map_err(io)?;
    drop(f.into_std().await);
    if let Ok(m) = tokio::fs::metadata(&path).await {
        tokio::fs::set_permissions(&tmp.0, m.permissions()).await.map_err(io)?;
    }
    tokio::fs::rename(&tmp.0, &path).await.map_err(io)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn mkdir(State(app): S, UrlPath(id): UrlPath<String>, Query(q): Query<At>) -> Result<StatusCode, Err> {
    let path = resolve_entry(&root(&app, &id).await?, &q.path)?;
    tokio::fs::create_dir(path).await.map_err(io)?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
pub struct Move {
    to: String,
}

pub async fn rename(State(app): S, UrlPath(id): UrlPath<String>, Query(q): Query<At>, Json(m): Json<Move>) -> Result<StatusCode, Err> {
    let root = root(&app, &id).await?;
    let (from, to) = (resolve_entry(&root, &q.path)?, resolve_entry(&root, &m.to)?);
    if to.symlink_metadata().is_ok() {
        return Err((StatusCode::CONFLICT, "Ce nom existe déjà".into()));
    }
    tokio::fs::rename(from, to).await.map_err(io)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(State(app): S, UrlPath(id): UrlPath<String>, Query(q): Query<At>) -> Result<StatusCode, Err> {
    let path = resolve_entry(&root(&app, &id).await?, &q.path)?;
    let m = tokio::fs::symlink_metadata(&path).await.map_err(io)?;
    if m.is_dir() {
        tokio::fs::remove_dir_all(path).await.map_err(io)?;
    } else {
        tokio::fs::remove_file(path).await.map_err(io)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chemins_hors_du_dossier_refuses() {
        let base = std::env::temp_dir().join(format!("foyer-f-{:x}", rand::random::<u64>()));
        let root = base.join("srv");
        std::fs::create_dir_all(root.join("world")).unwrap();
        std::fs::write(base.join("secret"), "x").unwrap();
        for p in ["..", "../secret", "world/../../secret", ".", "C:/x", "a\\b", "nul", "NUL.txt", "a:b", "x.", "com1"] {
            assert!(resolve(&root, p).is_err(), "{p}");
        }
        for p in ["", "/", "../x", "world/.."] {
            assert!(resolve_entry(&root, p).is_err(), "{p}");
        }
        let real = root.canonicalize().unwrap();
        assert_eq!(resolve(&root, "").unwrap(), real);
        assert_eq!(resolve(&root, "/world/neuf.txt").unwrap(), real.join("world/neuf.txt"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(base.join("secret"), root.join("lien")).unwrap();
            symlink(base.join("absent"), root.join("casse")).unwrap();
            symlink(&base, root.join("dossier")).unwrap();
            for p in ["lien", "casse", "dossier/secret", "dossier/neuf"] {
                assert!(resolve(&root, p).is_err(), "{p}");
            }
            // le lien lui-même reste supprimable
            assert_eq!(resolve_entry(&root, "lien").unwrap(), real.join("lien"));
        }
        std::fs::remove_dir_all(base).unwrap();
    }
}

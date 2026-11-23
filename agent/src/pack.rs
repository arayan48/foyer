//! Modpacks : un .mrpack (Modrinth) ou le « server pack » .zip d'un pack (CurseForge ou autre).
use crate::{files, games, software::Software};
use anyhow::{anyhow, bail, Result};
use futures_util::{stream, StreamExt};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

/// Télécharge et ouvre le pack dans `tmp`. Renvoie le dossier à copier (dossier unique en tête retiré).
pub async fn fetch(url: &str, tmp: &Path, log: &impl Fn(String)) -> Result<PathBuf> {
    if !url.starts_with("https://") {
        bail!("lien de modpack invalide : il doit commencer par https://");
    }
    tokio::fs::create_dir_all(tmp).await?;
    let file = tmp.join("pack.zip");
    log("Téléchargement du modpack…".into());
    games::download(url, &file).await?;
    let src = open(&file, tmp, log).await?;
    tokio::fs::remove_file(&file).await.ok();
    Ok(src)
}

/// Ouvre une archive (zip, tar.gz…) dans `tmp/x`. Renvoie le dossier utile.
pub async fn open(file: &Path, tmp: &Path, log: &impl Fn(String)) -> Result<PathBuf> {
    log("Extraction…".into());
    let out = tmp.join("x");
    games::extract(file, &out).await?;
    let src = tokio::task::spawn_blocking(move || -> std::io::Result<PathBuf> {
        // les liens de l'archive pourraient viser hors du dossier du serveur
        files::drop_links(&out)?;
        let top: Vec<_> = std::fs::read_dir(&out)?.collect::<Result<_, _>>()?;
        Ok(match top.as_slice() {
            [e] if e.file_type()?.is_dir() => e.path(),
            _ => out,
        })
    })
    .await??;
    Ok(src)
}

/// Logiciel et version de Minecraft demandés par un .mrpack. Rien pour un server pack .zip.
pub fn target(src: &Path) -> Option<(Software, String)> {
    let v: Value = serde_json::from_slice(&std::fs::read(src.join("modrinth.index.json")).ok()?).ok()?;
    Some(deps(&v["dependencies"])?)
}

fn deps(d: &Value) -> Option<(Software, String)> {
    let mc = d["minecraft"].as_str()?.to_string();
    let sw = if d.get("neoforge").is_some() {
        Software::Neoforge
    } else if d.get("forge").is_some() {
        Software::Forge
    } else if d.get("fabric-loader").is_some() {
        Software::Fabric
    } else {
        Software::Vanilla
    };
    Some((sw, mc))
}

/// Copie le pack dans le dossier du serveur. Le server.properties et l'EULA de Foyer sont gardés.
pub async fn apply(src: &Path, dir: &Path, log: &impl Fn(String)) -> Result<()> {
    let index = src.join("modrinth.index.json");
    if !index.is_file() {
        return copy(src, dir).await;
    }
    let v: Value = serde_json::from_slice(&tokio::fs::read(&index).await?)?;
    // shortcut: les sommes sha1 de l'index ne sont pas vérifiées, à ajouter si un miroir douteux apparaît
    let list: Vec<(PathBuf, String)> = v["files"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| f["env"]["server"] != "unsupported")
        .map(|f| Ok((safe(f["path"].as_str().unwrap_or(""))?, f["downloads"][0].as_str().ok_or_else(|| anyhow!("fichier du modpack sans lien"))?.to_string())))
        .collect::<Result<_>>()?;
    log(format!("Téléchargement de {} fichiers du modpack…", list.len()));
    let errs: Vec<anyhow::Error> = stream::iter(list)
        .map(|(p, url)| {
            let to = dir.join(p);
            async move {
                if let Some(d) = to.parent() {
                    tokio::fs::create_dir_all(d).await?;
                }
                games::download(&url, &to).await
            }
        })
        .buffer_unordered(8)
        .filter_map(|r| async { r.err() })
        .collect()
        .await;
    if let Some(e) = errs.first() {
        bail!("{} fichier(s) du modpack non téléchargé(s) : {e:#}", errs.len());
    }
    for o in ["overrides", "server-overrides"] {
        copy(&src.join(o), dir).await?;
    }
    Ok(())
}

async fn copy(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Ok(());
    }
    for keep in ["server.properties", "eula.txt"] {
        tokio::fs::remove_file(from.join(keep)).await.ok();
    }
    let (from, to) = (from.to_path_buf(), to.to_path_buf());
    tokio::task::spawn_blocking(move || files::merge(&from, &to)).await??;
    Ok(())
}

/// Chemin relatif sans « .. » : un pack ne doit rien écrire hors du serveur.
fn safe(p: &str) -> Result<PathBuf> {
    let p = PathBuf::from(p);
    if p.as_os_str().is_empty() || !p.components().all(|c| matches!(c, Component::Normal(_))) {
        bail!("chemin interdit dans le modpack : {}", p.display());
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chemins() {
        assert!(safe("mods/a.jar").is_ok());
        for p in ["", "../x", "/etc/x", "mods/../../x"] {
            assert!(safe(p).is_err(), "{p}");
        }
    }

    #[test]
    fn loader() {
        let d = serde_json::json!({ "minecraft": "1.21.1", "neoforge": "21.1.77" });
        assert_eq!(deps(&d), Some((Software::Neoforge, "1.21.1".into())));
        assert_eq!(deps(&serde_json::json!({ "minecraft": "1.20.1", "fabric-loader": "0.16" })), Some((Software::Fabric, "1.20.1".into())));
    }
}

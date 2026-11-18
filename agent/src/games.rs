//! Recettes de jeux : comment installer et lancer chaque serveur.
use anyhow::{anyhow, bail, Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Minecraft,
    Fivem,
}

const WIN: bool = cfg!(windows);

impl Game {
    pub fn default_port(self) -> u16 {
        match self {
            Game::Minecraft => 25565,
            Game::Fivem => 30120,
        }
    }

    /// Ports à ouvrir : (port, tcp, udp).
    pub fn ports(self, port: u16) -> Vec<(u16, bool, bool)> {
        match self {
            Game::Minecraft => vec![(port, true, false)],
            // 40120 = interface txAdmin (LAN seulement, ne jamais l'ouvrir sur la box)
            Game::Fivem => vec![(port, true, true), (txadmin_port(port), true, false)],
        }
    }

    /// Commande envoyée sur l'entrée standard pour un arrêt propre.
    pub fn stop_command(self) -> Option<&'static str> {
        match self {
            Game::Minecraft => Some("stop"),
            Game::Fivem => None,
        }
    }

    /// Ligne de log qui veut dire « prêt ».
    pub fn ready_marker(self) -> &'static str {
        match self {
            Game::Minecraft => "Done (",
            Game::Fivem => "txAdmin",
        }
    }
}

pub fn txadmin_port(port: u16) -> u16 {
    port + 10000
}

pub async fn install(game: Game, data: &Path, dir: &Path, port: u16, log: &impl Fn(String)) -> Result<()> {
    tokio::fs::create_dir_all(dir).await?;
    match game {
        Game::Minecraft => {
            java(data, log).await?;
            log("Recherche de la dernière version de Paper…".into());
            let p: Value = get_json("https://fill.papermc.io/v3/projects/paper").await?;
            let version = p["versions"]
                .as_object()
                .and_then(|m| m.values().next())
                .and_then(|v| v.as_array())
                .and_then(|v| v.iter().filter_map(|x| x.as_str()).find(|x| !x.contains('-')))
                .ok_or_else(|| anyhow!("aucune version Paper stable"))?
                .to_string();
            let b: Value = get_json(&format!("https://fill.papermc.io/v3/projects/paper/versions/{version}/builds/latest")).await?;
            let url = b["downloads"]["server:default"]["url"].as_str().ok_or_else(|| anyhow!("pas d'URL Paper"))?;
            log(format!("Téléchargement de Paper {version}…"));
            download(url, &dir.join("server.jar")).await?;
            tokio::fs::write(dir.join("eula.txt"), "eula=true\n").await?;
            tokio::fs::write(dir.join("server.properties"), format!("server-port={port}\nquery.port={port}\n")).await?;
        }
        Game::Fivem => {
            let os = if WIN { "win32" } else { "linux" };
            let v: Value = get_json(&format!("https://changelogs-live.fivem.net/api/changelog/versions/{os}/server")).await?;
            let url = v["recommended_download"].as_str().ok_or_else(|| anyhow!("pas d'URL FiveM"))?;
            log(format!("Téléchargement de FiveM (build {})…", v["recommended"].as_str().unwrap_or("?")));
            let file = dir.join(if WIN { "fx.zip" } else { "fx.tar.xz" });
            download(url, &file).await?;
            log("Extraction…".into());
            extract(&file, &dir.join("artifacts")).await?;
            tokio::fs::remove_file(&file).await.ok();
        }
    }
    log("Installation terminée.".into());
    Ok(())
}

/// Programme et arguments pour lancer le serveur.
pub async fn command(game: Game, data: &Path, dir: &Path, port: u16, ram_mb: u32) -> Result<(PathBuf, Vec<String>)> {
    Ok(match game {
        Game::Minecraft => (
            java_bin(data).await?,
            vec![format!("-Xms{ram_mb}M"), format!("-Xmx{ram_mb}M"), "-jar".into(), "server.jar".into(), "nogui".into()],
        ),
        Game::Fivem => {
            let a = dir.join("artifacts");
            let exe = if WIN { a.join("FXServer.exe") } else { a.join("run.sh") };
            (exe, vec!["+set".into(), "txAdminPort".into(), txadmin_port(port).to_string()])
        }
    })
}

/// Java (Temurin JRE 25) partagé par tous les serveurs Minecraft.
async fn java(data: &Path, log: &impl Fn(String)) -> Result<()> {
    if java_bin(data).await.is_ok() {
        return Ok(());
    }
    let os = if WIN { "windows" } else { "linux" };
    let dir = data.join("runtime").join("java25");
    let file = data.join("runtime").join("java.archive");
    tokio::fs::create_dir_all(&dir).await?;
    log("Téléchargement de Java 25…".into());
    download(&format!("https://api.adoptium.net/v3/binary/latest/25/ga/{os}/x64/jre/hotspot/normal/eclipse"), &file).await?;
    extract(&file, &dir).await?;
    tokio::fs::remove_file(&file).await.ok();
    java_bin(data).await.map(|_| ())
}

async fn java_bin(data: &Path) -> Result<PathBuf> {
    let mut rd = tokio::fs::read_dir(data.join("runtime").join("java25")).await?;
    while let Some(e) = rd.next_entry().await? {
        let bin = e.path().join("bin").join(if WIN { "java.exe" } else { "java" });
        if bin.exists() {
            return Ok(bin);
        }
    }
    bail!("Java introuvable")
}

async fn get_json(url: &str) -> Result<Value> {
    Ok(client().get(url).send().await?.error_for_status()?.json().await?)
}

async fn download(url: &str, to: &Path) -> Result<()> {
    let r = client().get(url).send().await?.error_for_status().with_context(|| url.to_string())?;
    let tmp = to.with_extension("part");
    let mut f = tokio::fs::File::create(&tmp).await?;
    let mut s = r.bytes_stream();
    while let Some(chunk) = s.next().await {
        f.write_all(&chunk?).await?;
    }
    f.flush().await?;
    drop(f);
    tokio::fs::rename(&tmp, to).await?;
    Ok(())
}

/// `tar` (bsdtar sous Windows) sait lire zip, tar.gz et tar.xz.
async fn extract(file: &Path, to: &Path) -> Result<()> {
    tokio::fs::create_dir_all(to).await?;
    let st = tokio::process::Command::new("tar").arg("-xf").arg(file).arg("-C").arg(to).status().await?;
    if !st.success() {
        bail!("échec de l'extraction de {}", file.display());
    }
    Ok(())
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().user_agent("foyer-agent").build().expect("client http")
}

/// Remplace ou ajoute des clés dans un texte `server.properties`, sans toucher au reste.
pub fn set_props(text: &str, kv: &[(&str, &str)]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut seen = vec![false; kv.len()];
    for line in text.lines() {
        let key = line.split('=').next().unwrap_or("").trim();
        match kv.iter().position(|(k, _)| *k == key && !line.trim_start().starts_with('#')) {
            Some(i) => { out.push(format!("{}={}", kv[i].0, kv[i].1)); seen[i] = true; }
            None => out.push(line.to_string()),
        }
    }
    for (i, (k, v)) in kv.iter().enumerate() {
        if !seen[i] { out.push(format!("{k}={v}")); }
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    #[test]
    fn set_props_keeps_rest() {
        let r = super::set_props("#c\nmotd=hi\nserver-port=1\n", &[("server-port", "2"), ("query.port", "2")]);
        assert_eq!(r, "#c\nmotd=hi\nserver-port=2\nquery.port=2\n");
    }
}

//! Logiciels serveur Minecraft : Vanilla, Paper, Purpur, Fabric, Forge, NeoForge.
use crate::games::{download, get_json, java, java_bin, java_for};
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Software {
    Vanilla,
    Paper,
    Purpur,
    Fabric,
    Forge,
    Neoforge,
}

/// Arguments de lancement propres au logiciel, un par ligne. Absent : `-jar server.jar`.
pub const LAUNCH: &str = "foyer-launch.txt";

const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const PROMOS: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
const FORGE: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";
const NEO_LIST: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
const NEO: &str = "https://maven.neoforged.net/releases/net/neoforged/neoforge";

impl Software {
    pub const ALL: [Software; 6] = [Self::Vanilla, Self::Paper, Self::Purpur, Self::Fabric, Self::Forge, Self::Neoforge];

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Paper => "Paper",
            Self::Purpur => "Purpur",
            Self::Fabric => "Fabric",
            Self::Forge => "Forge",
            Self::Neoforge => "NeoForge",
        }
    }

    /// Dossier des extensions, type Modrinth et chargeur Modrinth ; None si le logiciel n'en accepte pas.
    pub fn addons(self) -> Option<(&'static str, &'static str, &'static str)> {
        match self {
            Self::Vanilla => None,
            Self::Paper => Some(("plugins", "plugin", "paper")),
            Self::Purpur => Some(("plugins", "plugin", "purpur")),
            Self::Fabric => Some(("mods", "mod", "fabric")),
            Self::Forge => Some(("mods", "mod", "forge")),
            Self::Neoforge => Some(("mods", "mod", "neoforge")),
        }
    }
}

/// « Paper 1.21.11 » → (Paper, 1.21.11). Sans version connue : Paper, seul logiciel des premières versions de Foyer.
pub fn parse(label: Option<&str>) -> (Software, Option<String>) {
    let (name, mc) = label.and_then(|l| l.split_once(' ')).map_or(("Paper", None), |(n, v)| (n, Some(v.to_string())));
    (Software::ALL.into_iter().find(|s| s.label() == name).unwrap_or(Software::Paper), mc)
}

pub fn key(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|n| n.parse().ok()).collect()
}

/// Garde les versions stables à partir de 1.8, sans doublon, de la plus récente à la plus ancienne.
fn tidy(list: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = list.into_iter().filter(|v| v.chars().all(|c| c.is_ascii_digit() || c == '.') && key(v) >= vec![1, 8]).collect();
    v.sort_by_key(|v| std::cmp::Reverse(key(v)));
    v.dedup();
    v
}

/// Version de Minecraft d'une version stable de NeoForge : 21.1.77 → 1.21.1, 26.3.0.115 → 26.3.
pub fn neo_mc(v: &str) -> Option<String> {
    let n: Vec<u32> = v.split('.').map(|x| x.parse().ok()).collect::<Option<_>>()?;
    match n.as_slice() {
        [a, 0, _] if (20..26).contains(a) => Some(format!("1.{a}")),
        [a, b, _] if (20..26).contains(a) => Some(format!("1.{a}.{b}")),
        [a, b, 0, _] if *a >= 26 => Some(format!("{a}.{b}")),
        [a, b, c, _] if *a >= 26 => Some(format!("{a}.{b}.{c}")),
        _ => None,
    }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array().into_iter().flatten().filter_map(|x| x.as_str()).map(str::to_string).collect()
}

/// Versions de Minecraft installables, la plus récente d'abord.
pub async fn versions(sw: Software) -> Result<Vec<String>> {
    let list = match sw {
        Software::Vanilla => {
            let m = get_json(MANIFEST).await?;
            m["versions"].as_array().into_iter().flatten().filter(|v| v["type"] == "release").filter_map(|v| v["id"].as_str()).map(str::to_string).collect()
        }
        Software::Paper => {
            let p = get_json("https://fill.papermc.io/v3/projects/paper").await?;
            p["versions"].as_object().into_iter().flat_map(|m| m.values()).flat_map(strings).collect()
        }
        Software::Purpur => strings(&get_json("https://api.purpurmc.org/v2/purpur").await?["versions"]),
        Software::Fabric => {
            let f = get_json("https://meta.fabricmc.net/v2/versions/game").await?;
            f.as_array().into_iter().flatten().filter(|v| v["stable"] == true).filter_map(|v| v["version"].as_str()).map(str::to_string).collect::<Vec<_>>()
        }
        Software::Forge => {
            let p = get_json(PROMOS).await?;
            // avant 1.17, seulement les deux versions encore jouées (et corrigées contre Log4Shell)
            p["promos"].as_object().into_iter().flat_map(|m| m.keys()).filter_map(|k| k.split_once('-')).map(|(v, _)| v.to_string())
                .filter(|v| key(v) >= vec![1, 17] || v == "1.16.5" || v == "1.12.2").collect()
        }
        Software::Neoforge => strings(&get_json(NEO_LIST).await?["versions"]).iter().filter_map(|v| neo_mc(v)).collect(),
    };
    Ok(tidy(list))
}

/// Pose le logiciel dans `dir` sans toucher aux mondes ni aux réglages. Renvoie « Logiciel version ».
pub async fn install(sw: Software, data: &Path, dir: &Path, version: Option<&str>, j: Option<u8>, log: &impl Fn(String)) -> Result<String> {
    let name = sw.label();
    let list = versions(sw).await?;
    let mc = match version.filter(|v| !v.is_empty()) {
        Some(v) => list.iter().find(|x| *x == v).ok_or_else(|| anyhow!("{name} n'existe pas pour Minecraft {v}"))?,
        None => list.first().ok_or_else(|| anyhow!("aucune version de {name}"))?,
    }
    .as_str();
    let j = j.unwrap_or_else(|| java_for(mc));
    java(data, j, log).await?;
    log(format!("Téléchargement de {name} {mc}…"));
    let jar = dir.join("server.jar");
    let mut launch = None;
    match sw {
        Software::Vanilla => {
            let m = get_json(MANIFEST).await?;
            let url = m["versions"].as_array().into_iter().flatten().find(|v| v["id"] == mc).and_then(|v| v["url"].as_str()).ok_or_else(|| anyhow!("version introuvable"))?.to_string();
            let v = get_json(&url).await?;
            // Log4Shell : réglages officiels de Mojang pour les versions que Mojang n'a pas corrigées
            let fix = match key(mc) {
                k if k < vec![1, 12] => Some("4bb89a97a66f350bc9f73b3ca8509632682aea2e/log4j2_17-111.xml"),
                k if k < vec![1, 17] => Some("02937d122c86ce73319ef9975b58896fc1b491d1/log4j2_112-116.xml"),
                _ => None,
            };
            if let Some(f) = fix {
                download(&format!("https://launcher.mojang.com/v1/objects/{f}"), &dir.join("log4j2.xml")).await?;
                launch = Some("-Dlog4j.configurationFile=log4j2.xml\n-jar\nserver.jar".to_string());
            }
            download(v["downloads"]["server"]["url"].as_str().ok_or_else(|| anyhow!("pas de serveur pour cette version"))?, &jar).await?;
        }
        Software::Paper => {
            let b = get_json(&format!("https://fill.papermc.io/v3/projects/paper/versions/{mc}/builds/latest")).await?;
            download(b["downloads"]["server:default"]["url"].as_str().ok_or_else(|| anyhow!("pas d'URL Paper"))?, &jar).await?;
        }
        Software::Purpur => download(&format!("https://api.purpurmc.org/v2/purpur/{mc}/latest/download"), &jar).await?,
        Software::Fabric => {
            let stable = |v: &Value| v.as_array().and_then(|a| a.iter().find(|x| x["stable"] == true).or(a.first())).and_then(|x| x["version"].as_str()).map(str::to_string);
            let l = stable(&get_json("https://meta.fabricmc.net/v2/versions/loader").await?).ok_or_else(|| anyhow!("pas de chargeur Fabric"))?;
            let i = stable(&get_json("https://meta.fabricmc.net/v2/versions/installer").await?).ok_or_else(|| anyhow!("pas d'installeur Fabric"))?;
            download(&format!("https://meta.fabricmc.net/v2/versions/loader/{mc}/{l}/{i}/server/jar"), &jar).await?;
        }
        Software::Forge => {
            let p = get_json(PROMOS).await?;
            let p = &p["promos"];
            let v = p[format!("{mc}-recommended")].as_str().or(p[format!("{mc}-latest")].as_str()).ok_or_else(|| anyhow!("pas de Forge"))?;
            let full = format!("{mc}-{v}");
            installer(data, dir, &format!("{FORGE}/{full}/forge-{full}-installer.jar"), j, log).await?;
            launch = args_file(dir, &format!("libraries/net/minecraftforge/forge/{full}"));
            if launch.is_none() {
                tokio::fs::rename(dir.join(format!("forge-{full}.jar")), &jar).await?;
            }
        }
        Software::Neoforge => {
            let v = strings(&get_json(NEO_LIST).await?["versions"]).into_iter().filter(|v| neo_mc(v).as_deref() == Some(mc)).max_by_key(|v| key(v)).ok_or_else(|| anyhow!("pas de NeoForge"))?;
            installer(data, dir, &format!("{NEO}/{v}/neoforge-{v}-installer.jar"), j, log).await?;
            launch = Some(args_file(dir, &format!("libraries/net/neoforged/neoforge/{v}")).ok_or_else(|| anyhow!("installation de NeoForge incomplète"))?);
        }
    }
    let file = dir.join(LAUNCH);
    match launch {
        Some(a) => tokio::fs::write(&file, a).await?,
        None if file.exists() => tokio::fs::remove_file(&file).await?,
        None => {}
    }
    Ok(format!("{name} {mc}"))
}

/// Fichier d'arguments écrit par l'installeur Forge (1.17 et plus) ou NeoForge.
fn args_file(dir: &Path, sub: &str) -> Option<String> {
    let f = format!("{sub}/{}", if cfg!(windows) { "win_args.txt" } else { "unix_args.txt" });
    dir.join(&f).exists().then(|| format!("@{f}"))
}

async fn installer(data: &Path, dir: &Path, url: &str, j: u8, log: &impl Fn(String)) -> Result<()> {
    let file = dir.join("installer.jar");
    download(url, &file).await?;
    log("Installation, quelques minutes…".into());
    let mut c = tokio::process::Command::new(java_bin(data, j).await?);
    c.args(["-jar", "installer.jar", "--installServer"]).current_dir(dir).stdin(std::process::Stdio::null()).kill_on_drop(true);
    #[cfg(windows)]
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let out = tokio::time::timeout(std::time::Duration::from_secs(1200), c.output()).await.map_err(|_| anyhow!("installation trop longue"))??;
    tokio::fs::remove_file(&file).await.ok();
    tokio::fs::remove_file(dir.join("installer.jar.log")).await.ok();
    if !out.status.success() {
        let text = format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        bail!("l'installeur a échoué : {}", text.trim().lines().last().unwrap_or(""));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plus_recente_d_abord() {
        let v = tidy(["1.10.2", "1.7.10", "1.21.11", "26.3", "1.21.9", "26.4-snapshot-1", "1.21.9"].map(String::from));
        assert_eq!(v, ["26.3", "1.21.11", "1.21.9", "1.10.2"]);
    }

    #[test]
    fn versions_neoforge() {
        assert_eq!(neo_mc("21.1.77").as_deref(), Some("1.21.1"));
        assert_eq!(neo_mc("21.0.1").as_deref(), Some("1.21"));
        assert_eq!(neo_mc("26.3.0.115").as_deref(), Some("26.3"));
        assert_eq!(neo_mc("26.1.2.4").as_deref(), Some("26.1.2"));
        assert_eq!((neo_mc("21.1.1-beta"), neo_mc("0.25w14craftmine.3-beta")), (None, None));
    }

    #[test]
    fn logiciel_depuis_le_libelle() {
        assert_eq!(parse(Some("NeoForge 1.21.1")), (Software::Neoforge, Some("1.21.1".into())));
        assert_eq!(parse(Some("Vanilla 26.3")).0, Software::Vanilla);
        assert_eq!(parse(None), (Software::Paper, None));
    }
}

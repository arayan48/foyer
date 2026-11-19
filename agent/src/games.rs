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
    pub fn ports(self, port: u16, txadmin: bool) -> Vec<(u16, bool, bool)> {
        match self {
            Game::Minecraft => vec![(port, true, false)],
            // 40120 = interface txAdmin (LAN seulement, ne jamais l'ouvrir sur la box)
            Game::Fivem if txadmin => vec![(port, true, true), (txadmin_port(port), true, false)],
            Game::Fivem => vec![(port, true, true)],
        }
    }

    /// Commande envoyée sur l'entrée standard pour un arrêt propre.
    pub fn stop_command(self, txadmin: bool) -> Option<&'static str> {
        match self {
            Game::Minecraft => Some("stop"),
            Game::Fivem if txadmin => None,
            Game::Fivem => Some("quit"),
        }
    }

    /// Ligne de log qui veut dire « prêt ».
    pub fn ready_marker(self, txadmin: bool) -> &'static str {
        match self {
            Game::Minecraft => "Done (",
            Game::Fivem if txadmin => "txAdmin",
            Game::Fivem => "Started resource",
        }
    }
}

pub fn txadmin_port(port: u16) -> u16 {
    port + 10000
}

/// Choix faits à la création du serveur.
#[derive(Deserialize, Default, Clone)]
pub struct Setup {
    /// Minecraft : version du jeu (« 1.21.11 »), vide = dernière stable.
    /// FiveM : canal des artefacts (recommended, optional, latest).
    pub version: Option<String>,
    /// Version de Java pour Minecraft, vide = celle conseillée pour la version du jeu.
    pub java: Option<u8>,
    #[serde(default = "yes")]
    pub txadmin: bool,
    /// Sans txAdmin : vanilla, esx ou qbcore.
    pub framework: Option<String>,
    pub license_key: Option<String>,
    pub max_players: Option<u16>,
    /// Chaîne de connexion MySQL/MariaDB pour ESX et QBCore.
    pub db_url: Option<String>,
}

/// Base créée par Foyer dans la MariaDB locale.
pub const DEFAULT_DB: &str = "mysql://root@localhost/fivem";

pub fn yes() -> bool {
    true
}

pub const JAVAS: [u8; 8] = [8, 11, 17, 21, 22, 23, 24, 25];

/// Java conseillé pour une version de Minecraft.
pub fn java_for(mc: &str) -> u8 {
    let n: Vec<u32> = mc.split('.').filter_map(|x| x.parse().ok()).collect();
    match n.as_slice() {
        [a, ..] if *a >= 26 => 25,
        [1, m, ..] if *m >= 21 => 21,
        [1, 20, p] if *p >= 5 => 21,
        [1, m, ..] if *m >= 17 => 17,
        _ => 8,
    }
}

/// Versions proposées à la création : Minecraft (Paper stables) et builds FiveM.
pub async fn catalog() -> Result<Value> {
    let p: Value = get_json("https://fill.papermc.io/v3/projects/paper").await?;
    let mc: Vec<Value> = p["versions"]
        .as_object()
        .map(|m| m.values().filter_map(|v| v.as_array()).flatten().filter_map(|x| x.as_str()).filter(|x| !x.contains('-')).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .map(|v| serde_json::json!({ "id": v, "java": java_for(v) }))
        .collect();
    let os = if WIN { "win32" } else { "linux" };
    let f: Value = get_json(&format!("https://changelogs-live.fivem.net/api/changelog/versions/{os}/server")).await.unwrap_or_default();
    let fivem: Vec<Value> = ["recommended", "optional", "latest"]
        .iter()
        .filter_map(|c| f[c].as_str().map(|b| serde_json::json!({ "id": c, "build": b })))
        .collect();
    Ok(serde_json::json!({ "minecraft": mc, "java": JAVAS, "fivem": fivem }))
}

/// Installe le serveur et renvoie la version installée.
pub async fn install(game: Game, data: &Path, dir: &Path, port: u16, name: &str, o: &Setup, log: &impl Fn(String)) -> Result<String> {
    tokio::fs::create_dir_all(dir).await?;
    let label = match game {
        Game::Minecraft => {
            let version = match o.version.as_deref().filter(|v| !v.is_empty()) {
                Some(v) => v.to_string(),
                None => {
                    log("Recherche de la dernière version de Paper…".into());
                    let p: Value = get_json("https://fill.papermc.io/v3/projects/paper").await?;
                    p["versions"]
                        .as_object()
                        .and_then(|m| m.values().next())
                        .and_then(|v| v.as_array())
                        .and_then(|v| v.iter().filter_map(|x| x.as_str()).find(|x| !x.contains('-')))
                        .ok_or_else(|| anyhow!("aucune version Paper stable"))?
                        .to_string()
                }
            };
            if !version.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
                bail!("version invalide");
            }
            java(data, o.java.unwrap_or_else(|| java_for(&version)), log).await?;
            let b: Value = get_json(&format!("https://fill.papermc.io/v3/projects/paper/versions/{version}/builds/latest")).await?;
            let url = b["downloads"]["server:default"]["url"].as_str().ok_or_else(|| anyhow!("pas d'URL Paper"))?;
            log(format!("Téléchargement de Paper {version}…"));
            download(url, &dir.join("server.jar")).await?;
            tokio::fs::write(dir.join("eula.txt"), "eula=true\n").await?;
            tokio::fs::write(dir.join("server.properties"), format!("server-port={port}\nquery.port={port}\n")).await?;
            format!("Paper {version}")
        }
        Game::Fivem => {
            let os = if WIN { "win32" } else { "linux" };
            let channel = match o.version.as_deref() {
                Some(c @ ("optional" | "latest")) => c,
                _ => "recommended",
            };
            let v: Value = get_json(&format!("https://changelogs-live.fivem.net/api/changelog/versions/{os}/server")).await?;
            let url = v[format!("{channel}_download")].as_str().ok_or_else(|| anyhow!("pas d'URL FiveM"))?;
            let build = v[channel].as_str().unwrap_or("?").to_string();
            log(format!("Téléchargement de FiveM (build {build}, {channel})…"));
            let file = dir.join(if WIN { "fx.zip" } else { "fx.tar.xz" });
            download(url, &file).await?;
            log("Extraction…".into());
            extract(&file, &dir.join("artifacts")).await?;
            tokio::fs::remove_file(&file).await.ok();
            let fw = o.framework.as_deref().unwrap_or("vanilla");
            let own_db = o.db_url.as_deref().is_some_and(|u| !u.is_empty() && u != DEFAULT_DB);
            let need_db = !own_db && (o.txadmin || fw == "esx" || fw == "qbcore");
            if need_db {
                if let Err(e) = mariadb(data, log).await {
                    log(format!("MariaDB non installée : {e:#}"));
                }
            }
            if !o.txadmin {
                server_data(dir, port, name, o, log).await?;
                if need_db && fw != "vanilla" {
                    if let Err(e) = import_sql(data, &dir.join("server-data").join("resources"), log).await {
                        log(format!("Import SQL incomplet : {e:#}"));
                    }
                }
            }
            format!("Build {build}")
        }
    };
    log("Installation terminée.".into());
    Ok(label)
}

/// Sans txAdmin : ressources de base, framework choisi et server.cfg prêt à lancer.
async fn server_data(dir: &Path, port: u16, name: &str, o: &Setup, log: &impl Fn(String)) -> Result<()> {
    let key = o.license_key.as_deref().unwrap_or("").trim();
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        bail!("clé de licence invalide");
    }
    let clean = |s: &str| s.chars().filter(|c| !matches!(c, '"' | '\n' | '\r')).collect::<String>();
    let sd = dir.join("server-data");
    let res = sd.join("resources");
    log("Téléchargement des ressources de base (cfx-server-data)…".into());
    fetch_tree("https://github.com/citizenfx/cfx-server-data/archive/refs/heads/master.zip", &sd, dir).await?;
    let fw = o.framework.as_deref().unwrap_or("vanilla");
    let mut ensure = vec!["mapmanager", "chat", "spawnmanager", "sessionmanager", "hardcap"];
    let mut extra = String::new();
    if fw == "esx" || fw == "qbcore" {
        log("Téléchargement d'oxmysql…".into());
        fetch_tree("https://github.com/overextended/oxmysql/releases/latest/download/oxmysql.zip", &res.join("[ox]").join("oxmysql"), dir).await?;
        let db = clean(o.db_url.as_deref().unwrap_or(DEFAULT_DB));
        extra += &format!("set mysql_connection_string \"{db}\"\n");
        ensure.push("oxmysql");
    }
    match fw {
        "esx" => {
            let r: Value = get_json("https://api.github.com/repos/esx-framework/esx_core/releases/latest").await?;
            let url = r["zipball_url"].as_str().ok_or_else(|| anyhow!("pas d'archive ESX"))?;
            log(format!("Téléchargement d'ESX Legacy {}…", r["tag_name"].as_str().unwrap_or("")));
            fetch_tree(url, &res.join("[esx]"), dir).await?;
            ensure.push("[core]");
        }
        "qbcore" => {
            log("Téléchargement de QBCore…".into());
            fetch_tree("https://github.com/qbcore-framework/qb-core/archive/refs/heads/main.zip", &res.join("[qb]").join("qb-core"), dir).await?;
            ensure.push("qb-core");
        }
        _ => ensure.push("basic-gamemode"),
    }
    let ensure: String = ensure.iter().map(|r| format!("ensure {r}\n")).collect();
    let cfg = format!(
        "endpoint_add_tcp \"0.0.0.0:{port}\"\nendpoint_add_udp \"0.0.0.0:{port}\"\n\n{ensure}\n{extra}sv_hostname \"{name}\"\nsets sv_projectName \"{name}\"\nsv_maxclients {max}\nset onesync on\nsv_scriptHookAllowed 0\nsv_licenseKey \"{key}\"\n",
        name = clean(name),
        max = o.max_players.unwrap_or(48).clamp(1, 2048),
    );
    tokio::fs::write(sd.join("server.cfg"), cfg).await?;
    if key.is_empty() {
        log("Aucune clé de licence : ajoute sv_licenseKey dans server.cfg (gratuite sur portal.cfx.re).".into());
    }
    Ok(())
}

/// Télécharge une archive qui contient un seul dossier et le place à `to`.
async fn fetch_tree(url: &str, to: &Path, work: &Path) -> Result<()> {
    let tmp = work.join("tmp-extract");
    let file = work.join("tmp.zip");
    tokio::fs::remove_dir_all(&tmp).await.ok();
    download(url, &file).await?;
    extract(&file, &tmp).await?;
    tokio::fs::remove_file(&file).await.ok();
    let mut rd = tokio::fs::read_dir(&tmp).await?;
    let top = rd.next_entry().await?.ok_or_else(|| anyhow!("archive vide : {url}"))?.path();
    if let Some(p) = to.parent() {
        tokio::fs::create_dir_all(p).await?;
    }
    tokio::fs::rename(&top, to).await?;
    tokio::fs::remove_dir_all(&tmp).await.ok();
    Ok(())
}

impl Game {
    /// Plafond mémoire du processus : le tas Java plus la marge de la JVM, ou la RAM telle quelle pour FiveM.
    pub fn mem_cap(self, ram_mb: u32) -> u32 {
        match self {
            Game::Minecraft => ram_mb + (ram_mb / 4).max(512),
            Game::Fivem => ram_mb,
        }
    }
}

/// Programme, arguments et dossier de travail pour lancer le serveur.
pub async fn command(game: Game, data: &Path, dir: &Path, port: u16, ram_mb: u32, java: u8, txadmin: bool) -> Result<(PathBuf, Vec<String>, PathBuf)> {
    Ok(match game {
        Game::Minecraft => (
            java_bin(data, java).await?,
            vec![format!("-Xms{ram_mb}M"), format!("-Xmx{ram_mb}M"), "-jar".into(), "server.jar".into(), "nogui".into()],
            dir.to_path_buf(),
        ),
        Game::Fivem => {
            let a = dir.join("artifacts");
            let exe = if WIN { a.join("FXServer.exe") } else { a.join("run.sh") };
            if txadmin {
                (exe, vec!["+set".into(), "txAdminPort".into(), txadmin_port(port).to_string()], dir.to_path_buf())
            } else {
                (exe, vec!["+exec".into(), "server.cfg".into()], dir.join("server-data"))
            }
        }
    })
}

/// Java (Temurin JRE) partagé par les serveurs Minecraft, un dossier par version.
pub async fn java(data: &Path, v: u8, log: &impl Fn(String)) -> Result<()> {
    if !JAVAS.contains(&v) {
        bail!("Java {v} non proposé");
    }
    if java_bin(data, v).await.is_ok() {
        return Ok(());
    }
    let os = if WIN { "windows" } else { "linux" };
    let dir = data.join("runtime").join(format!("java{v}"));
    let file = data.join("runtime").join("java.archive");
    tokio::fs::create_dir_all(&dir).await?;
    log(format!("Téléchargement de Java {v}…"));
    download(&format!("https://api.adoptium.net/v3/binary/latest/{v}/ga/{os}/x64/jre/hotspot/normal/eclipse"), &file).await?;
    extract(&file, &dir).await?;
    tokio::fs::remove_file(&file).await.ok();
    java_bin(data, v).await.map(|_| ())
}

async fn java_bin(data: &Path, v: u8) -> Result<PathBuf> {
    let mut rd = tokio::fs::read_dir(data.join("runtime").join(format!("java{v}"))).await?;
    while let Some(e) = rd.next_entry().await? {
        let bin = e.path().join("bin").join(if WIN { "java.exe" } else { "java" });
        if bin.exists() {
            return Ok(bin);
        }
    }
    bail!("Java introuvable")
}

/// MariaDB (LTS 11.4) partagée par les serveurs FiveM, écoute 127.0.0.1:3306, root sans mot de passe.
async fn mariadb(data: &Path, log: &impl Fn(String)) -> Result<()> {
    let rt = data.join("runtime");
    if mariadb_bin(data, "mariadbd").await.is_err() {
        let v: Value = get_json("https://downloads.mariadb.org/rest-api/mariadb/11.4/latest/").await?;
        let end = if WIN { "-winx64.zip" } else { "-linux-systemd-x86_64.tar.gz" };
        let url = v["releases"]
            .as_object()
            .and_then(|r| r.values().next())
            .and_then(|r| r["files"].as_array())
            .and_then(|f| f.iter().find(|f| f["file_name"].as_str().is_some_and(|n| n.ends_with(end))))
            .and_then(|f| f["file_download_url"].as_str())
            .ok_or_else(|| anyhow!("pas d'URL MariaDB"))?
            .replace("http://", "https://");
        let file = rt.join("mariadb.archive");
        tokio::fs::create_dir_all(rt.join("mariadb")).await?;
        log("Téléchargement de MariaDB…".into());
        download(&url, &file).await?;
        extract(&file, &rt.join("mariadb")).await?;
        tokio::fs::remove_file(&file).await.ok();
    }
    let datadir = rt.join("mariadb-data");
    if !datadir.join("mysql").exists() {
        log("Création de la base de données…".into());
        let base = mariadb_bin(data, "mariadbd").await?.parent().and_then(|b| b.parent()).ok_or_else(|| anyhow!("dossier MariaDB"))?.to_path_buf();
        let mut c = if WIN {
            tokio::process::Command::new(base.join("bin").join("mariadb-install-db.exe"))
        } else {
            let mut c = tokio::process::Command::new(base.join("scripts").join("mariadb-install-db"));
            c.arg("--no-defaults").arg(format!("--basedir={}", base.display())).arg("--auth-root-authentication-method=normal").arg("--user=root");
            c
        };
        let out = c.arg(format!("--datadir={}", datadir.display())).output().await?;
        if !out.status.success() {
            bail!("mariadb-install-db : {}", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or(""));
        }
    }
    mariadb_start(data).await
}

/// Lance MariaDB si elle est installée et pas déjà en route. Elle reste allumée après l'arrêt des serveurs.
pub async fn mariadb_start(data: &Path) -> Result<()> {
    let up = || async { tokio::net::TcpStream::connect("127.0.0.1:3306").await.is_ok() };
    if up().await {
        return Ok(());
    }
    let bin = mariadb_bin(data, "mariadbd").await?;
    let datadir = data.join("runtime").join("mariadb-data");
    let mut c = tokio::process::Command::new(&bin);
    c.arg("--no-defaults").arg(format!("--datadir={}", datadir.display())).arg("--port=3306").arg("--bind-address=127.0.0.1");
    if !WIN {
        let base = bin.parent().and_then(|b| b.parent()).unwrap_or(Path::new("."));
        c.arg(format!("--basedir={}", base.display())).arg(format!("--socket={}", datadir.join("mysql.sock").display())).arg("--user=root");
    }
    c.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).kill_on_drop(false);
    #[cfg(windows)]
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    #[cfg(unix)]
    c.process_group(0);
    c.spawn()?;
    for _ in 0..60 {
        if up().await {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    bail!("MariaDB ne répond pas sur le port 3306")
}

async fn mariadb_bin(data: &Path, name: &str) -> Result<PathBuf> {
    let mut rd = tokio::fs::read_dir(data.join("runtime").join("mariadb")).await?;
    while let Some(e) = rd.next_entry().await? {
        let bin = e.path().join("bin").join(if WIN { format!("{name}.exe") } else { name.to_string() });
        if bin.exists() {
            return Ok(bin);
        }
    }
    bail!("MariaDB introuvable")
}

/// Crée la base `fivem` et y importe les fichiers .sql du framework.
async fn import_sql(data: &Path, res: &Path, log: &impl Fn(String)) -> Result<()> {
    let client = mariadb_bin(data, "mariadb").await?;
    let run = |sql: Option<PathBuf>| {
        let client = client.clone();
        async move {
            let mut c = tokio::process::Command::new(&client);
            c.args(["-h", "127.0.0.1", "-P", "3306", "-u", "root"]);
            match &sql {
                Some(f) => { c.arg("fivem").stdin(std::fs::File::open(f)?); }
                None => { c.args(["-e", "CREATE DATABASE IF NOT EXISTS fivem CHARACTER SET utf8mb4"]); }
            }
            let out = c.output().await?;
            if !out.status.success() {
                bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
            }
            anyhow::Ok(())
        }
    };
    run(None).await?;
    let mut files = vec![];
    let mut todo = vec![res.to_path_buf()];
    while let Some(d) = todo.pop() {
        let mut rd = tokio::fs::read_dir(&d).await?;
        while let Some(e) = rd.next_entry().await? {
            let p = e.path();
            if e.file_type().await?.is_dir() {
                todo.push(p);
            } else if p.extension().is_some_and(|x| x == "sql") {
                files.push(p);
            }
        }
    }
    files.sort();
    log(format!("Import de {} fichier(s) SQL dans la base fivem…", files.len()));
    for f in files {
        // un fichier en échec (table déjà là, dépendance absente) n'empêche pas les autres
        if let Err(e) = run(Some(f.clone())).await {
            log(format!("{} : {e:#}", f.file_name().unwrap_or_default().to_string_lossy()));
        }
    }
    Ok(())
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

/// Plugins par page dans la recherche.
const PAGE: u32 = 20;

/// Recherche de plugins Paper sur Modrinth, compatibles avec la version de Minecraft si connue.
/// `sort` : relevance, downloads, follows, newest, updated. `page` commence à 0. `cat` : catégorie Modrinth ou vide.
pub async fn plugin_search(q: &str, mc: Option<&str>, sort: &str, page: u32, cat: &str) -> Result<Value> {
    let sort = if ["downloads", "follows", "newest", "updated"].contains(&sort) { sort } else { "relevance" };
    let mut facets = vec![r#"["project_type:plugin"]"#.to_string(), r#"["categories:paper"]"#.to_string()];
    if !cat.is_empty() && cat.len() < 40 && cat.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
        facets.push(serde_json::json!([format!("categories:{cat}")]).to_string());
    }
    if let Some(mc) = mc {
        facets.push(serde_json::json!([format!("versions:{mc}")]).to_string());
    }
    let facets = format!("[{}]", facets.join(","));
    let (limit, offset) = (PAGE.to_string(), (page.min(500) * PAGE).to_string());
    let r: Value = client().get("https://api.modrinth.com/v2/search")
        .query(&[("query", q), ("facets", &facets), ("index", sort), ("limit", &limit), ("offset", &offset)])
        .send().await?.error_for_status()?.json().await?;
    let hits: Vec<Value> = r["hits"].as_array().cloned().unwrap_or_default().into_iter().map(|h| serde_json::json!({
        "id": h["project_id"], "title": h["title"], "description": h["description"],
        "author": h["author"], "downloads": h["downloads"], "icon": h["icon_url"],
        "follows": h["follows"], "updated": h["date_modified"], "categories": h["display_categories"],
    })).collect();
    Ok(serde_json::json!({ "hits": hits, "total": r["total_hits"], "per_page": PAGE }))
}

/// Télécharge la dernière version compatible d'un plugin Modrinth dans `dir`. Renvoie le nom du fichier.
pub async fn plugin_install(project: &str, mc: Option<&str>, dir: &Path) -> Result<String> {
    if project.is_empty() || !project.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        bail!("plugin inconnu");
    }
    let mut q = vec![("loaders", r#"["paper"]"#.to_string())];
    if let Some(mc) = mc {
        q.push(("game_versions", serde_json::json!([mc]).to_string()));
    }
    let v: Value = client().get(format!("https://api.modrinth.com/v2/project/{project}/version"))
        .query(&q).send().await?.error_for_status()?.json().await?;
    let files = v[0]["files"].as_array().filter(|f| !f.is_empty())
        .ok_or_else(|| anyhow!("aucune version de ce plugin pour Minecraft {}", mc.unwrap_or("?")))?;
    let f = files.iter().find(|f| f["primary"] == true).unwrap_or(&files[0]);
    let name = f["filename"].as_str().filter(|n| plugin_file_ok(n)).ok_or_else(|| anyhow!("fichier du plugin invalide"))?;
    let url = f["url"].as_str().ok_or_else(|| anyhow!("pas d'URL de téléchargement"))?;
    tokio::fs::create_dir_all(dir).await?;
    download(url, &dir.join(name)).await?;
    Ok(name.to_string())
}

/// Nom de fichier de plugin sans chemin : `x.jar` ou `x.jar.disabled`.
pub fn plugin_file_ok(n: &str) -> bool {
    !n.starts_with('.') && !n.contains(['/', '\\', ':']) && !n.contains("..") && (n.ends_with(".jar") || n.ends_with(".jar.disabled"))
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

    #[test]
    fn noms_de_plugins() {
        use super::plugin_file_ok as ok;
        assert!(ok("LuckPerms-5.4.jar") && ok("a.jar.disabled"));
        assert!(!ok("../a.jar") && !ok("a/b.jar") && !ok("a\\b.jar") && !ok("a.txt") && !ok(".jar"));
    }

    #[test]
    fn java_par_version() {
        use super::java_for;
        assert_eq!((java_for("1.8.8"), java_for("1.16.5"), java_for("1.18.2"), java_for("1.20.4")), (8, 8, 17, 17));
        assert_eq!((java_for("1.20.6"), java_for("1.21.11"), java_for("26.1.2")), (21, 21, 25));
    }
}

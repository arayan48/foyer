//! Foyer agent : installe, lance et surveille des serveurs de jeu.
//! API HTTP + WebSocket protégée par un jeton, à garder sur le réseau local.
mod backups;
mod files;
mod games;
mod journal;
mod limits;
mod orphans;
mod players;
mod software;
mod trash;

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Path as UrlPath, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use games::Game;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Disks, Pid, ProcessesToUpdate, System};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{broadcast, Mutex};

const LOG_LINES: usize = 1000;

#[derive(Clone, Serialize, Deserialize)]
struct Server {
    id: String,
    name: String,
    game: Game,
    port: u16,
    ram_mb: u32,
    /// démarre quand l'agent démarre (donc au boot)
    #[serde(default)]
    auto_start: bool,
    /// relance après un crash, 3 essais max par fenêtre de 10 min
    #[serde(default)]
    auto_restart: bool,
    /// part max du processeur de la machine en %, 0 = pas de limite
    #[serde(default)]
    cpu_limit: u8,
    /// cœurs autorisés, vide = tous
    #[serde(default)]
    cores: Vec<u16>,
    /// bloque la mémoire du processus autour de ram_mb
    #[serde(default)]
    ram_cap: bool,
    /// version installée (« Vanilla 26.3 », « Paper 1.21.11 », « Build 38435 »)
    #[serde(default)]
    version: Option<String>,
    /// version de Java (Minecraft), 25 pour les anciens serveurs
    #[serde(default = "java_default")]
    java: u8,
    /// FiveM lancé via txAdmin, sinon directement avec server.cfg
    #[serde(default = "games::yes")]
    txadmin: bool,
    /// FiveM sans txAdmin : vanilla, esx ou qbcore
    #[serde(default)]
    framework: Option<String>,
}

fn java_default() -> u8 {
    25
}

impl Server {
    fn ports(&self) -> Vec<(u16, bool, bool)> {
        self.game.ports(self.port, self.txadmin)
    }
}

#[derive(Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Installing,
    Stopped,
    Starting,
    Running,
    Stopping,
    /// sauvegarde ou restauration, serveur arrêté
    Busy,
    Error,
}

struct Live {
    status: Status,
    stdin: Option<ChildStdin>,
    pid: Option<u32>,
    log: VecDeque<String>,
    tx: broadcast::Sender<String>,
    cpu: f32,
    mem: u64,
    crashes: VecDeque<Instant>,
    /// joueurs présents (Minecraft)
    players: std::collections::BTreeSet<String>,
    /// taille du dossier en octets, None avant le premier calcul
    disk: Option<u64>,
    /// sauvegarde ou restauration en cours, serveur allumé ou non
    backing: bool,
    backup_error: Option<String>,
}

impl Live {
    fn new(status: Status) -> Self {
        Live { status, stdin: None, pid: None, log: VecDeque::new(), tx: broadcast::channel(256).0, cpu: 0.0, mem: 0, crashes: VecDeque::new(), players: Default::default(), disk: None, backing: false, backup_error: None }
    }
    fn push(&mut self, line: String) {
        if self.log.len() >= LOG_LINES {
            self.log.pop_front();
        }
        self.log.push_back(line.clone());
        let _ = self.tx.send(line);
    }
}

struct App {
    data: PathBuf,
    token: String,
    servers: Mutex<Vec<Server>>,
    live: Mutex<HashMap<String, Live>>,
    /// mauvais jetons récents par adresse
    failures: Mutex<HashMap<IpAddr, VecDeque<Instant>>>,
    host: Mutex<HostStats>,
}

#[derive(Clone, Default, Serialize)]
struct HostStats {
    name: String,
    os: String,
    cpu: f32,
    mem_used: u64,
    mem_total: u64,
    disk_free: u64,
    disk_total: u64,
    cores: usize,
    threads: usize,
    cpu_model: String,
    arch: String,
    kernel: String,
    uptime: u64,
    agent: String,
}

type S = State<Arc<App>>;

#[tokio::main]
async fn main() -> Result<()> {
    let r = run().await;
    if let Err(e) = &r {
        journal::agent(&format!("arrêt sur erreur : {e:#}"));
    }
    r
}

async fn run() -> Result<()> {
    let data = std::env::var("FOYER_DATA").map(PathBuf::from).unwrap_or_else(|_| {
        if cfg!(windows) { PathBuf::from(r"C:\Foyer") } else { PathBuf::from("/var/lib/foyer") }
    });
    tokio::fs::create_dir_all(data.join("servers")).await?;
    journal::init(&data);
    let token = match tokio::fs::read_to_string(data.join("token")).await {
        Ok(t) => t.trim().to_string(),
        Err(_) => {
            let t: String = (0..32).map(|_| format!("{:x}", rand::random::<u8>() % 16)).collect();
            tokio::fs::write(data.join("token"), &t).await?;
            t
        }
    };
    let servers: Vec<Server> = tokio::fs::read(data.join("servers.json")).await.ok()
        .and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    let live = servers.iter().map(|s| (s.id.clone(), Live::new(Status::Stopped))).collect();
    let app = Arc::new(App { data: data.clone(), token, servers: Mutex::new(servers), live: Mutex::new(live), failures: Default::default(), host: Default::default() });

    orphans::stop(&data).await;
    backups::recover(&data).await;
    let _ = tokio::fs::remove_dir_all(data.join("tmp")).await; // envois interrompus
    tokio::spawn(stats_loop(app.clone()));
    tokio::spawn(disk_loop(app.clone()));
    tokio::spawn(trash::purge_loop(app.clone()));
    let auto: Vec<String> = app.servers.lock().await.iter().filter(|s| s.auto_start).map(|s| s.id.clone()).collect();
    for id in auto {
        if let Err(e) = start(&app, &id).await {
            journal::agent(&format!("démarrage auto de {id} : {e:#}"));
        }
    }

    let api = Router::new()
        .route("/api/system", get(system))
        .route("/api/shutdown", post(shutdown))
        .route("/api/catalog", get(catalog))
        .route("/api/catalog/{software}", get(software_versions))
        .route("/api/servers", get(list).post(create))
        .route("/api/servers/{id}", axum::routing::delete(remove).patch(update))
        .route("/api/servers/{id}/{action}", post(action))
        .route("/api/servers/{id}/software", post(software_set))
        .route("/api/servers/{id}/console", get(console))
        .route("/api/servers/{id}/properties", get(props_get).put(props_put))
        .route("/api/servers/{id}/plugins", get(plugins).post(plugin_add))
        .route("/api/servers/{id}/plugins/search", get(plugin_search))
        .route("/api/servers/{id}/plugins/{file}", axum::routing::delete(plugin_del).patch(plugin_toggle))
        .route("/api/servers/{id}/files", get(files::list).post(files::mkdir).patch(files::rename).delete(files::delete))
        .route("/api/servers/{id}/files/content", get(files::read).put(files::write))
        .route("/api/servers/{id}/backups", get(backups::list).post(backups::add))
        .route("/api/servers/{id}/backups/{backup}", post(backups::restore).delete(backups::delete))
        .route("/api/servers/{id}/players", get(players::list))
        .route("/api/servers/{id}/players/{name}/{action}", post(players::act))
        .route("/api/servers/{id}/rules/{rule}/{on}", post(players::rule))
        .route("/api/journal", get(journal::list))
        .route("/api/trash", get(trash::list))
        .route("/api/trash/{id}", post(trash::restore).delete(trash::purge))
        .layer(middleware::from_fn_with_state(app.clone(), journal::layer))
        .layer(middleware::from_fn_with_state(app.clone(), auth))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(app);

    let addr = std::env::var("FOYER_ADDR").unwrap_or_else(|_| "0.0.0.0:7878".into());
    journal::agent(&format!("Foyer agent {} sur https://{addr}  données : {}", env!("CARGO_PKG_VERSION"), data.display()));
    journal::agent(&format!("Jeton : fichier {}", data.join("token").display()));
    let (tls, fp) = tls(&data).await?;
    journal::agent(&format!("Empreinte du certificat : {fp}"));
    axum_server::bind_rustls(addr.parse()?, tls).serve(api.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

/// Certificat auto-signé créé au premier lancement ; le client épingle son empreinte (pas d'autorité externe).
async fn tls(data: &std::path::Path) -> Result<(axum_server::tls_rustls::RustlsConfig, String)> {
    use rustls::pki_types::{pem::PemObject, CertificateDer};
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (cert, key) = (data.join("cert.pem"), data.join("key.pem"));
    if !cert.exists() || !key.exists() {
        let ck = rcgen::generate_simple_self_signed(vec!["foyer".into()])?;
        tokio::fs::write(&key, ck.key_pair.serialize_pem()).await?;
        tokio::fs::write(&cert, ck.cert.pem()).await?;
    }
    let der = CertificateDer::from_pem_file(&cert)?;
    let fp = fingerprint(&der);
    tokio::fs::write(data.join("fingerprint"), &fp).await?;
    Ok((axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key).await?, fp))
}

fn fingerprint(der: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, der).as_ref().iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":")
}

async fn auth(State(app): S, ConnectInfo(addr): ConnectInfo<SocketAddr>, req: Request, next: Next) -> Response {
    // jeton seulement en en-tête : jamais dans l'URL, qui finit dans les journaux
    let given = req.headers().get("authorization").and_then(|h| h.to_str().ok()).and_then(|h| h.strip_prefix("Bearer ")).unwrap_or_default().to_string();
    if req.method() == axum::http::Method::OPTIONS {
        return next.run(req).await;
    }
    let ip = addr.ip();
    // la machine elle-même n'est jamais bloquée : l'installeur doit pouvoir arrêter l'agent
    let guarded = !ip.is_loopback();
    if guarded && is_blocked(&*app.failures.lock().await, ip, Instant::now()) {
        return (StatusCode::TOO_MANY_REQUESTS, "Trop de mauvais jetons, réessaie dans 10 min").into_response();
    }
    if eq_ct(given.as_bytes(), app.token.as_bytes()) {
        return next.run(req).await;
    }
    // sans jeton du tout : lecture de l'empreinte par le client, pas un essai
    if guarded && !given.is_empty() && note_failure(&mut *app.failures.lock().await, ip, Instant::now()) {
        journal::agent(&format!("{ip} bloquée 10 min : {MAX_FAILURES} mauvais jetons"));
    }
    StatusCode::UNAUTHORIZED.into_response()
}

const MAX_FAILURES: usize = 10;
const BLOCK: Duration = Duration::from_secs(600);

fn is_blocked(failures: &HashMap<IpAddr, VecDeque<Instant>>, ip: IpAddr, now: Instant) -> bool {
    failures.get(&ip).is_some_and(|f| f.len() >= MAX_FAILURES && f.back().is_some_and(|t| now.duration_since(*t) < BLOCK))
}

/// Note un mauvais jeton ; vrai si l'adresse vient d'être bloquée.
fn note_failure(failures: &mut HashMap<IpAddr, VecDeque<Instant>>, ip: IpAddr, now: Instant) -> bool {
    failures.retain(|_, f| f.back().is_some_and(|t| now.duration_since(*t) < BLOCK));
    let f = failures.entry(ip).or_default();
    // une adresse encore dans la table : on oublie ses essais trop vieux
    while f.front().is_some_and(|t| now.duration_since(*t) >= BLOCK) {
        f.pop_front();
    }
    f.push_back(now);
    f.len() == MAX_FAILURES
}

fn eq_ct(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Arrêt propre de tous les jeux puis de l'agent (utilisé par l'installeur avant une mise à jour).
async fn shutdown(State(app): S) -> StatusCode {
    let ids: Vec<String> = app.servers.lock().await.iter().map(|s| s.id.clone()).collect();
    futures_util::future::join_all(ids.iter().map(|id| stop(&app, id))).await;
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(300)).await;
        std::process::exit(0);
    });
    StatusCode::NO_CONTENT
}

async fn system(State(app): S) -> Json<HostStats> {
    Json(app.host.lock().await.clone())
}

#[derive(Serialize)]
struct ServerView {
    #[serde(flatten)]
    server: Server,
    status: Status,
    cpu: f32,
    mem: u64,
    disk: Option<u64>,
    /// joueurs présents, Minecraft seulement
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<std::collections::BTreeSet<String>>,
    admin_port: Option<u16>,
}

async fn list(State(app): S) -> Json<Vec<ServerView>> {
    let servers = app.servers.lock().await.clone();
    let live = app.live.lock().await;
    Json(servers.into_iter().map(|s| {
        let l = live.get(&s.id);
        ServerView {
            status: l.map(|l| l.status).unwrap_or(Status::Stopped),
            cpu: l.map(|l| l.cpu).unwrap_or(0.0),
            mem: l.map(|l| l.mem).unwrap_or(0),
            disk: l.and_then(|l| l.disk),
            players: l.filter(|_| s.game == Game::Minecraft).map(|l| l.players.clone()),
            admin_port: (s.game == Game::Fivem && s.txadmin).then(|| games::txadmin_port(s.port)),
            server: s,
        }
    }).collect())
}

#[derive(Deserialize)]
struct NewServer {
    name: String,
    game: Game,
    port: Option<u16>,
    ram_mb: Option<u32>,
    #[serde(default)]
    eula: bool,
    #[serde(flatten)]
    setup: games::Setup,
}

async fn catalog() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    games::catalog().await.map(Json).map_err(|e| (StatusCode::BAD_GATEWAY, format!("{e:#}")))
}

async fn software_versions(UrlPath(sw): UrlPath<software::Software>) -> Result<Json<Vec<serde_json::Value>>, Err> {
    games::minecraft_versions(sw).await.map(Json).map_err(|e| (StatusCode::BAD_GATEWAY, format!("{e:#}")))
}

#[derive(Deserialize)]
struct SetSoftware { software: software::Software, version: String }

/// Change le logiciel ou la version de Minecraft d'un serveur arrêté. Mondes et réglages restent.
async fn software_set(State(app): S, UrlPath(id): UrlPath<String>, Json(b): Json<SetSoftware>) -> Result<StatusCode, Err> {
    mc_plugins(&app, &id).await?; // refuse serveur inconnu ou autre jeu
    let java = games::java_for(&b.version);
    {
        let mut live = app.live.lock().await;
        let l = live.get_mut(&id).filter(|l| !l.backing && matches!(l.status, Status::Stopped | Status::Error)).ok_or((StatusCode::CONFLICT, "Arrête le serveur d'abord".into()))?;
        l.status = Status::Installing;
    }
    tokio::spawn(async move {
        let dir = app.data.join("servers").join(&id);
        let log = logger(&app, &id);
        let r = async {
            let mut before = None;
            if dir.exists() {
                log("Sauvegarde avant le changement…".into());
                before = Some(backups::create(&app, &id, &format!("Avant {} {}", b.software.label(), b.version), true, false).await?);
            }
            let r = software::install(b.software, &app.data, &dir, Some(&b.version), Some(java), &log).await;
            let (Err(e), Some(before)) = (&r, before) else { return r };
            log("L'installation a échoué, retour à l'état d'avant…".into());
            let done = match backups::put_back(&app, &id, &before, false).await {
                Ok(()) => "tout a été remis comme avant".to_string(),
                Err(x) => format!("retour arrière impossible ({x:#}), restaure la sauvegarde à la main"),
            };
            Err(anyhow::anyhow!("{e:#} ; {done}"))
        }
        .await;
        if let Ok(v) = &r {
            log(format!("{v} installé."));
            let mut servers = app.servers.lock().await;
            if let Some(x) = servers.iter_mut().find(|x| x.id == id) {
                x.version = Some(v.clone());
                x.java = java;
            }
            save(&app, &servers).await;
        }
        if let Some(l) = app.live.lock().await.get_mut(&id) {
            l.status = Status::Stopped;
            if let Err(e) = r {
                l.status = Status::Error;
                l.push(format!("[foyer] Erreur : {e:#}"));
                journal::agent(&format!("changement de logiciel de {id} : {e:#}"));
            }
        }
    });
    Ok(StatusCode::NO_CONTENT)
}

/// Écrit dans la console d'un serveur depuis une fonction non asynchrone.
fn logger(app: &Arc<App>, id: &str) -> impl Fn(String) + use<> {
    let (app, id) = (app.clone(), id.to_string());
    move |m| {
        let (app, id) = (app.clone(), id.clone());
        tokio::spawn(async move { if let Some(l) = app.live.lock().await.get_mut(&id) { l.push(format!("[foyer] {m}")) } });
    }
}

async fn create(State(app): S, Json(n): Json<NewServer>) -> Result<Json<Server>, (StatusCode, String)> {
    let bad = |m: &str| Err((StatusCode::BAD_REQUEST, m.to_string()));
    let name = n.name.trim().to_string();
    if name.is_empty() || name.len() > 60 {
        return bad("Nom vide ou trop long");
    }
    if n.game == Game::Minecraft && !n.eula {
        return bad("Il faut accepter l'EULA de Minecraft");
    }
    let port = n.port.unwrap_or(n.game.default_port());
    let ram_mb = n.ram_mb.unwrap_or(4096).clamp(512, 65536);
    let mut servers = app.servers.lock().await;
    let used: Vec<u16> = servers.iter().flat_map(|s| s.ports()).map(|p| p.0).collect();
    if n.game.ports(port, n.setup.txadmin).iter().any(|p| used.contains(&p.0)) {
        return bad("Port déjà utilisé par un autre serveur");
    }
    let id: String = (0..8).map(|_| format!("{:x}", rand::random::<u8>() % 16)).collect();
    if n.setup.java.is_some_and(|j| !games::JAVAS.contains(&j)) {
        return bad("Version de Java non proposée");
    }
    if let Err(e) = n.setup.properties(port, "") {
        return bad(&e.to_string());
    }
    let fivem = n.game == Game::Fivem;
    let txadmin = !fivem || n.setup.txadmin;
    let framework = (fivem && !txadmin).then(|| n.setup.framework.clone().unwrap_or_else(|| "vanilla".into()));
    if framework.as_deref().is_some_and(|f| !["vanilla", "esx", "qbcore"].contains(&f)) {
        return bad("Framework inconnu");
    }
    let java = n.setup.java.unwrap_or_else(|| n.setup.version.as_deref().map_or(25, games::java_for));
    let s = Server { id: id.clone(), name, game: n.game, port, ram_mb, auto_start: false, auto_restart: true, cpu_limit: 0, cores: vec![], ram_cap: false, version: None, java, txadmin, framework };
    servers.push(s.clone());
    save(&app, &servers).await;
    drop(servers);
    app.live.lock().await.insert(id.clone(), Live::new(Status::Installing));

    let app2 = app.clone();
    let s2 = s.clone();
    tokio::spawn(async move {
        let dir = app2.data.join("servers").join(&s2.id);
        let log = logger(&app2, &s2.id);
        let setup = games::Setup { txadmin: s2.txadmin, framework: s2.framework.clone(), ..n.setup };
        let auto_java = s2.game == Game::Minecraft && setup.java.is_none();
        let r = games::install(s2.game, &app2.data, &dir, s2.port, &s2.name, &setup, &log).await;
        if let Ok(v) = &r {
            firewall(&s2, true).await;
            let mut servers = app2.servers.lock().await;
            if let Some(x) = servers.iter_mut().find(|x| x.id == s2.id) {
                x.version = Some(v.clone());
                // sans version demandée, le bon Java n'est connu qu'une fois le logiciel installé
                if let (true, Some(mc)) = (auto_java, software::parse(Some(v)).1) {
                    x.java = games::java_for(&mc);
                }
            }
            save(&app2, &servers).await;
        }
        let mut live = app2.live.lock().await;
        if let Some(l) = live.get_mut(&s2.id) {
            match r {
                Ok(_) => l.status = Status::Stopped,
                Err(e) => {
                    l.status = Status::Error;
                    l.push(format!("[foyer] Erreur : {e:#}"));
                    journal::agent(&format!("installation de {} : {e:#}", s2.id));
                }
            }
        }
    });
    Ok(Json(s))
}

#[derive(Deserialize)]
struct Patch {
    name: Option<String>,
    port: Option<u16>,
    ram_mb: Option<u32>,
    auto_start: Option<bool>,
    auto_restart: Option<bool>,
    cpu_limit: Option<u8>,
    cores: Option<Vec<u16>>,
    ram_cap: Option<bool>,
    java: Option<u8>,
}

/// Modifie un serveur. Port et RAM servent au prochain démarrage.
async fn update(State(app): S, UrlPath(id): UrlPath<String>, Json(p): Json<Patch>) -> Result<Json<Server>, (StatusCode, String)> {
    let bad = |m: &str| Err((StatusCode::BAD_REQUEST, m.to_string()));
    let mut servers = app.servers.lock().await;
    let Some(i) = servers.iter().position(|s| s.id == id) else { return Err((StatusCode::NOT_FOUND, "serveur inconnu".into())) };
    let old = servers[i].clone();
    let mut s = old.clone();
    if let Some(name) = p.name {
        let name = name.trim().to_string();
        if name.is_empty() || name.len() > 60 {
            return bad("Nom vide ou trop long");
        }
        s.name = name;
    }
    if let Some(ram) = p.ram_mb {
        s.ram_mb = ram.clamp(512, 65536);
    }
    s.auto_start = p.auto_start.unwrap_or(s.auto_start);
    s.auto_restart = p.auto_restart.unwrap_or(s.auto_restart);
    if let Some(c) = p.cpu_limit {
        if c != 0 && !(5..=100).contains(&c) {
            return bad("Limite processeur entre 5 et 100 %");
        }
        s.cpu_limit = c;
    }
    if let Some(mut cores) = p.cores {
        cores.sort_unstable();
        cores.dedup();
        if cores.iter().any(|&c| c as usize >= limits::cores().min(64)) {
            return bad("Cœur inexistant sur cette machine");
        }
        s.cores = cores;
    }
    s.ram_cap = p.ram_cap.unwrap_or(s.ram_cap);
    if let Some(j) = p.java {
        if !games::JAVAS.contains(&j) {
            return bad("Version de Java non proposée");
        }
        s.java = j;
    }
    if let Some(port) = p.port.filter(|&p| p != old.port) {
        if s.game != Game::Minecraft {
            return bad("Le port FiveM se règle dans txAdmin");
        }
        if port < 1024 {
            return bad("Port trop bas (minimum 1024)");
        }
        let used: Vec<u16> = servers.iter().filter(|o| o.id != id).flat_map(|o| o.ports()).map(|p| p.0).collect();
        if s.game.ports(port, s.txadmin).iter().any(|p| used.contains(&p.0)) {
            return bad("Port déjà utilisé par un autre serveur");
        }
        let st = app.live.lock().await.get(&id).map(|l| l.status);
        if !matches!(st, Some(Status::Stopped | Status::Error)) {
            return bad("Arrête le serveur avant de changer le port");
        }
        let props = app.data.join("servers").join(&id).join("server.properties");
        let text = games::read_props(&props).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("server.properties : {e}")))?;
        if let Err(e) = tokio::fs::write(&props, games::set_props(&text, &[("server-port", &port.to_string()), ("query.port", &port.to_string())])).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("server.properties : {e}")));
        }
        s.port = port;
    }
    servers[i] = s.clone();
    save(&app, &servers).await;
    drop(servers);
    if s.port != old.port {
        firewall(&old, false).await;
        firewall(&s, true).await;
    }
    Ok(Json(s))
}

/// Arrête le serveur et le met à la corbeille.
async fn remove(State(app): S, UrlPath(id): UrlPath<String>) -> Result<StatusCode, Err> {
    if app.live.lock().await.get(&id).is_some_and(|l| l.backing || l.status == Status::Installing) {
        return Err((StatusCode::CONFLICT, "Attends la fin de l'opération en cours".into()));
    }
    stop(&app, &id).await;
    let mut servers = app.servers.lock().await;
    let Some(i) = servers.iter().position(|s| s.id == id) else { return Err((StatusCode::NOT_FOUND, "serveur inconnu".into())) };
    if app.live.lock().await.get(&id).is_some_and(|l| l.pid.is_some()) {
        return Err((StatusCode::CONFLICT, "Le serveur ne s'arrête pas".into()));
    }
    trash::put(&app, &servers[i]).await?;
    let s = servers.remove(i);
    save(&app, &servers).await;
    drop(servers);
    app.live.lock().await.remove(&id);
    firewall(&s, false).await;
    #[cfg(target_os = "linux")]
    limits::linux::remove(&id);
    Ok(StatusCode::NO_CONTENT)
}

type Err = (StatusCode, String);

/// Dossier plugins ou mods, logiciel et version de Minecraft d'un serveur Minecraft.
async fn mc_plugins(app: &App, id: &str) -> Result<(PathBuf, software::Software, Option<String>), Err> {
    let s = app.servers.lock().await.iter().find(|s| s.id == id).cloned().ok_or((StatusCode::NOT_FOUND, "serveur inconnu".into()))?;
    if s.game != Game::Minecraft {
        return Err((StatusCode::BAD_REQUEST, "Réservé à Minecraft".into()));
    }
    let (sw, mc) = software::parse(s.version.as_deref());
    Ok((app.data.join("servers").join(id).join(sw.addons().map_or("plugins", |a| a.0)), sw, mc))
}

fn no_addons() -> Err {
    (StatusCode::BAD_REQUEST, "Vanilla n'accepte ni plugins ni mods".into())
}

/// Chemin de `server.properties` d'un serveur Minecraft.
async fn mc_props(app: &App, id: &str) -> Result<PathBuf, Err> {
    let (dir, ..) = mc_plugins(app, id).await?;
    Ok(dir.with_file_name("server.properties"))
}

async fn props_get(State(app): S, UrlPath(id): UrlPath<String>) -> Result<Json<std::collections::BTreeMap<String, String>>, Err> {
    let text = games::read_props(&mc_props(&app, &id).await?).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("server.properties : {e}")))?;
    Ok(Json(games::parse_props(&text)))
}

/// Écrit les clés reçues ; le port reste géré par PATCH (pare-feu, conflits).
async fn props_put(State(app): S, UrlPath(id): UrlPath<String>, Json(kv): Json<std::collections::BTreeMap<String, String>>) -> Result<StatusCode, Err> {
    let path = mc_props(&app, &id).await?;
    let bad = |m: String| Err((StatusCode::BAD_REQUEST, m));
    for (k, v) in &kv {
        if k.is_empty() || k.contains(['=', ':', '#', '!', ' ', '\n', '\r']) || v.contains('\r') {
            return bad(format!("Valeur invalide : {k}"));
        }
        if k == "server-port" || k == "query.port" {
            return bad("Change le port dans les paramètres du serveur".into());
        }
    }
    let text = games::read_props(&path).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("server.properties : {e}")))?;
    let pairs: Vec<(&str, &str)> = kv.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    tokio::fs::write(&path, games::set_props(&text, &pairs)).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("server.properties : {e}")))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct Plugin { file: String, enabled: bool, size: u64 }

async fn plugins(State(app): S, UrlPath(id): UrlPath<String>) -> Result<Json<Vec<Plugin>>, Err> {
    let (dir, ..) = mc_plugins(&app, &id).await?;
    let mut out = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(e)) = rd.next_entry().await {
            let file = e.file_name().to_string_lossy().into_owned();
            if games::plugin_file_ok(&file) {
                let size = e.metadata().await.map(|m| m.len()).unwrap_or(0);
                out.push(Plugin { enabled: file.ends_with(".jar"), file, size });
            }
        }
    }
    out.sort_by_key(|p| p.file.to_lowercase());
    Ok(Json(out))
}

#[derive(Deserialize)]
struct Q { #[serde(default)] q: String, #[serde(default)] sort: String, #[serde(default)] page: u32, #[serde(default)] cat: String }

async fn plugin_search(State(app): S, UrlPath(id): UrlPath<String>, axum::extract::Query(q): axum::extract::Query<Q>) -> Result<Json<serde_json::Value>, Err> {
    let (_, sw, mc) = mc_plugins(&app, &id).await?;
    let (_, kind, loader) = sw.addons().ok_or_else(no_addons)?;
    games::plugin_search(kind, loader, &q.q, mc.as_deref(), &q.sort, q.page, &q.cat).await.map(Json).map_err(|e| (StatusCode::BAD_GATEWAY, format!("Modrinth : {e:#}")))
}

#[derive(Deserialize)]
struct AddPlugin { project: String }

async fn plugin_add(State(app): S, UrlPath(id): UrlPath<String>, Json(b): Json<AddPlugin>) -> Result<Json<String>, Err> {
    let (dir, sw, mc) = mc_plugins(&app, &id).await?;
    let (_, _, loader) = sw.addons().ok_or_else(no_addons)?;
    games::plugin_install(loader, &b.project, mc.as_deref(), &dir).await.map(Json).map_err(|e| (StatusCode::BAD_GATEWAY, format!("{e:#}")))
}

#[derive(Deserialize)]
struct Toggle { enabled: bool }

async fn plugin_toggle(State(app): S, UrlPath((id, file)): UrlPath<(String, String)>, Json(t): Json<Toggle>) -> Result<StatusCode, Err> {
    let (dir, ..) = mc_plugins(&app, &id).await?;
    if !games::plugin_file_ok(&file) {
        return Err((StatusCode::BAD_REQUEST, "fichier invalide".into()));
    }
    let base = file.trim_end_matches(".disabled");
    let to = if t.enabled { base.to_string() } else { format!("{base}.disabled") };
    if to != file {
        tokio::fs::rename(dir.join(&file), dir.join(&to)).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn plugin_del(State(app): S, UrlPath((id, file)): UrlPath<(String, String)>) -> Result<StatusCode, Err> {
    let (dir, ..) = mc_plugins(&app, &id).await?;
    if !games::plugin_file_ok(&file) {
        return Err((StatusCode::BAD_REQUEST, "fichier invalide".into()));
    }
    tokio::fs::remove_file(dir.join(&file)).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn action(State(app): S, UrlPath((id, action)): UrlPath<(String, String)>) -> Result<StatusCode, (StatusCode, String)> {
    let r = match action.as_str() {
        "start" => start(&app, &id).await,
        "stop" => { stop(&app, &id).await; Ok(()) }
        "restart" => { stop(&app, &id).await; start(&app, &id).await }
        _ => return Err((StatusCode::NOT_FOUND, "action inconnue".into())),
    };
    r.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::CONFLICT, format!("{e:#}")))
}

fn start(app: &Arc<App>, id: &str) -> futures_util::future::BoxFuture<'static, Result<()>> {
    // boxé : start est rappelé depuis sa propre tâche de surveillance (redémarrage auto)
    let (app, id) = (app.clone(), id.to_string());
    Box::pin(async move {
        // réservé tout de suite : une restauration ne peut pas s'intercaler
        match app.live.lock().await.get_mut(&id) {
            Some(l) if !l.backing && matches!(l.status, Status::Stopped | Status::Error) => l.status = Status::Starting,
            Some(_) => anyhow::bail!("le serveur n'est pas arrêté"),
            None => anyhow::bail!("serveur inconnu"),
        }
        let r = start_inner(&app, &id).await;
        if r.is_err() {
            if let Some(l) = app.live.lock().await.get_mut(&id).filter(|l| l.pid.is_none()) { l.status = Status::Error }
        }
        r
    })
}

async fn start_inner(app: &Arc<App>, id: &str) -> Result<()> {
    let s = app.servers.lock().await.iter().find(|s| s.id == id).cloned().ok_or_else(|| anyhow::anyhow!("serveur inconnu"))?;
    let dir = app.data.join("servers").join(id);
    if s.game == Game::Fivem && app.data.join("runtime").join("mariadb-data").exists() {
        if let Err(e) = games::mariadb_start(&app.data).await {
            if let Some(l) = app.live.lock().await.get_mut(id) { l.push(format!("[foyer] MariaDB : {e:#}")) }
        }
    }
    if s.game == Game::Minecraft {
        // installe la version choisie si elle manque (changée dans les réglages)
        let log = logger(app, id);
        games::java(&app.data, s.java, &log).await?;
    }
    let (exe, args, cwd) = games::command(s.game, &app.data, &dir, s.port, s.ram_mb, s.java, s.txadmin).await?;
    let mut cmd = Command::new(&exe);
    cmd.args(&args).current_dir(&cwd).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(false);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    #[cfg(unix)]
    cmd.process_group(0);
    let lim = limits::Limits { cpu_percent: s.cpu_limit, cores: s.cores.clone(), mem_mb: s.ram_cap.then(|| s.game.mem_cap(s.ram_mb)) };
    let mut warn = None;
    #[cfg(target_os = "linux")]
    let _procs = match limits::linux::prepare(id, &lim) {
        Ok(f) => {
            let f2 = f.try_clone()?;
            // inscrit le processus dans son cgroup avant exec : ses enfants suivent
            unsafe {
                cmd.pre_exec(move || {
                    use std::io::Write;
                    (&f2).write_all(b"0")
                });
            }
            Some(f)
        }
        Err(e) if lim.is_empty() => { let _ = e; None }
        Err(e) => { warn = Some(e.to_string()); None }
    };
    let mut child: Child = cmd.spawn()?;
    let pid = child.id();
    #[cfg(windows)]
    if !lim.is_empty() {
        if let Some(h) = child.raw_handle() {
            if let Err(e) = limits::windows::apply(h, &lim) {
                warn = Some(e.to_string());
            }
        }
    }
    {
        let mut live = app.live.lock().await;
        let l = live.get_mut(id).unwrap();
        l.status = Status::Starting;
        l.stdin = child.stdin.take();
        l.pid = pid;
        l.push(format!("[foyer] Démarrage : {}", exe.display()));
        if let Some(w) = &warn {
            l.push(format!("[foyer] Limites non appliquées : {w}"));
        } else if !lim.is_empty() {
            l.push(format!("[foyer] Limites : processeur {} %, cœurs {}, mémoire {}",
                if lim.cpu_percent == 0 { 100 } else { lim.cpu_percent },
                if lim.cores.is_empty() { "tous".into() } else { format!("{:?}", lim.cores) },
                lim.mem_mb.map_or("libre".into(), |m| format!("{m} Mo"))));
        }
    }
    for out in [child.stdout.take().map(|o| Box::new(o) as Box<dyn tokio::io::AsyncRead + Unpin + Send>),
                child.stderr.take().map(|o| Box::new(o) as Box<dyn tokio::io::AsyncRead + Unpin + Send>)].into_iter().flatten() {
        let app = app.clone();
        let id = id.to_string();
        let marker = s.game.ready_marker(s.txadmin);
        let mc = s.game == Game::Minecraft;
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            let mut unpacking = false;
            while let Ok(Some(mut line)) = lines.next_line().await {
                // Vanilla au premier lancement : une ligne par bibliothèque
                if mc && line.starts_with("Unpacking ") && line.contains(" to ") {
                    if unpacking {
                        continue;
                    }
                    unpacking = true;
                    line = "[foyer] Préparation des fichiers…".into();
                }
                let mut live = app.live.lock().await;
                if let Some(l) = live.get_mut(&id) {
                    if l.status == Status::Starting && line.contains(marker) {
                        l.status = Status::Running;
                    }
                    if mc {
                        players::track(&mut l.players, &line);
                    }
                    l.push(line);
                }
            }
        });
    }
    let app = app.clone();
    let id = id.to_string();
    tokio::spawn(async move {
        let code = child.wait().await.ok().and_then(|c| c.code());
        journal::agent(&format!("serveur {id} terminé (code {code:?})"));
        let auto = app.servers.lock().await.iter().find(|s| s.id == id).is_some_and(|s| s.auto_restart);
        let retry = {
            let mut live = app.live.lock().await;
            let Some(l) = live.get_mut(&id) else { return };
            let crashed = l.status != Status::Stopping;
            l.status = if crashed { Status::Error } else { Status::Stopped };
            l.stdin = None;
            l.pid = None;
            l.cpu = 0.0;
            l.mem = 0;
            l.players.clear();
            l.push(format!("[foyer] Processus terminé (code {code:?})"));
            if !(crashed && auto) {
                false
            } else if should_retry(&mut l.crashes, Instant::now()) {
                l.push(format!("[foyer] Crash : redémarrage automatique dans 5 s (essai {}/{MAX_CRASHES})", l.crashes.len()));
                true
            } else {
                l.push(format!("[foyer] {MAX_CRASHES} crashs en 10 min : redémarrage automatique abandonné"));
                false
            }
        };
        if retry {
            tokio::time::sleep(Duration::from_secs(5)).await;
            // démarré ou supprimé à la main entre-temps : on ne touche à rien
            if app.live.lock().await.get(&id).map(|l| l.status) != Some(Status::Error) {
                return;
            }
            if let Err(e) = start(&app, &id).await {
                journal::agent(&format!("redémarrage de {id} : {e:#}"));
                if let Some(l) = app.live.lock().await.get_mut(&id) { l.push(format!("[foyer] Redémarrage raté : {e:#}")) }
            }
        }
    });
    Ok(())
}

const MAX_CRASHES: usize = 3;
const CRASH_WINDOW: Duration = Duration::from_secs(600);

/// Note un crash ; vrai s'il en reste moins de MAX_CRASHES dans la fenêtre.
fn should_retry(crashes: &mut VecDeque<Instant>, now: Instant) -> bool {
    while crashes.front().is_some_and(|t| now.duration_since(*t) > CRASH_WINDOW) {
        crashes.pop_front();
    }
    if crashes.len() >= MAX_CRASHES {
        return false;
    }
    crashes.push_back(now);
    true
}

async fn stop(app: &Arc<App>, id: &str) {
    let stop_cmd = app.servers.lock().await.iter().find(|s| s.id == id).and_then(|s| s.game.stop_command(s.txadmin));
    let pid = {
        let mut live = app.live.lock().await;
        let Some(l) = live.get_mut(id) else { return };
        let Some(pid) = l.pid else { return };
        l.status = Status::Stopping;
        l.push("[foyer] Arrêt…".into());
        if let (Some(cmd), Some(stdin)) = (stop_cmd, l.stdin.as_mut()) {
            let _ = stdin.write_all(format!("{cmd}\n").as_bytes()).await;
        }
        pid
    };
    // attend l'arrêt propre 30 s, sinon tue l'arbre de processus
    for _ in 0..60 {
        if app.live.lock().await.get(id).map_or(true, |l| l.pid.is_none()) {
            return;
        }
        if stop_cmd.is_none() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    kill_tree(pid).await;
    for _ in 0..20 {
        if app.live.lock().await.get(id).map_or(true, |l| l.pid.is_none()) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

async fn kill_tree(pid: u32) {
    let mut c = if cfg!(windows) {
        let mut c = Command::new("taskkill");
        c.args(["/T", "/F", "/PID", &pid.to_string()]);
        c
    } else {
        let mut c = Command::new("kill");
        c.args(["-KILL", "--", &format!("-{pid}")]);
        c
    };
    let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).status().await;
}

async fn console(State(app): S, UrlPath(id): UrlPath<String>, ConnectInfo(addr): ConnectInfo<SocketAddr>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| console_ws(app, id, addr.ip().to_string(), socket))
}

async fn console_ws(app: Arc<App>, id: String, ip: String, mut socket: WebSocket) {
    let (backlog, mut rx) = {
        let live = app.live.lock().await;
        let Some(l) = live.get(&id) else { return };
        (l.log.iter().cloned().collect::<Vec<_>>(), l.tx.subscribe())
    };
    for line in backlog {
        if socket.send(Message::Text(line.into())).await.is_err() {
            return;
        }
    }
    loop {
        tokio::select! {
            line = rx.recv() => match line {
                Ok(line) => if socket.send(Message::Text(line.into())).await.is_err() { return },
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => return,
            },
            msg = socket.recv() => match msg {
                Some(Ok(Message::Text(cmd))) => {
                    journal::add(&app, &ip, &format!("CONSOLE {id} {cmd}"), None).await;
                    let mut live = app.live.lock().await;
                    if let Some(l) = live.get_mut(&id) {
                        l.push(format!("> {cmd}"));
                        if let Some(stdin) = l.stdin.as_mut() {
                            let _ = stdin.write_all(format!("{cmd}\n").as_bytes()).await;
                        }
                    }
                }
                Some(Ok(_)) => {}
                _ => return,
            }
        }
    }
}

async fn stats_loop(app: Arc<App>) {
    let mut sys = System::new();
    let mut disks = Disks::new_with_refreshed_list();
    sys.refresh_cpu_all(); // remplit le nom du processeur
    loop {
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        disks.refresh(true);
        let ncpu = sys.cpus().len().max(1) as f32;
        let data = app.data.clone();
        // disque qui contient le dossier de données
        let disk = disks.iter().filter(|d| data.starts_with(d.mount_point())).max_by_key(|d| d.mount_point().as_os_str().len());
        *app.host.lock().await = HostStats {
            name: System::host_name().unwrap_or_default(),
            os: System::long_os_version().unwrap_or_default(),
            cpu: sys.global_cpu_usage(),
            mem_used: sys.used_memory(),
            mem_total: sys.total_memory(),
            disk_free: disk.map_or(0, |d| d.available_space()),
            disk_total: disk.map_or(0, |d| d.total_space()),
            cores: sys.physical_core_count().unwrap_or(0),
            threads: sys.cpus().len(),
            cpu_model: sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default(),
            arch: System::cpu_arch(),
            kernel: System::kernel_version().unwrap_or_default(),
            uptime: System::uptime(),
            agent: env!("CARGO_PKG_VERSION").into(),
        };
        let mut live = app.live.lock().await;
        for l in live.values_mut() {
            let Some(pid) = l.pid else { continue };
            let root = Pid::from_u32(pid);
            // processus + enfants directs (txAdmin lance FXServer en enfant)
            let (cpu, mem) = sys.processes().values()
                .filter(|p| p.thread_kind().is_none() && (p.pid() == root || p.parent() == Some(root)))
                .fold((0.0, 0), |(c, m), p| (c + p.cpu_usage(), m + p.memory()));
            l.cpu = cpu / ncpu;
            l.mem = mem;
        }
        drop(live);
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Taille des dossiers, toutes les 5 min : parcourir un gros monde coûte trop pour `stats_loop`.
async fn disk_loop(app: Arc<App>) {
    loop {
        let ids: Vec<String> = app.servers.lock().await.iter().map(|s| s.id.clone()).collect();
        for id in ids {
            let dir = app.data.join("servers").join(&id);
            let size = tokio::task::spawn_blocking(move || dir_size(&dir)).await.ok();
            if let Some(l) = app.live.lock().await.get_mut(&id) {
                l.disk = size;
            }
        }
        tokio::time::sleep(Duration::from_secs(300)).await;
    }
}

/// Sans suivre les liens symboliques (DirEntry::metadata ne les suit pas).
fn dir_size(dir: &std::path::Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
    rd.flatten().map(|e| match e.metadata() {
        Ok(m) if m.is_dir() => dir_size(&e.path()),
        Ok(m) if m.is_file() => m.len(),
        _ => 0,
    }).sum()
}

async fn save(app: &App, servers: &[Server]) {
    let tmp = app.data.join("servers.json.tmp");
    if tokio::fs::write(&tmp, serde_json::to_vec_pretty(servers).unwrap()).await.is_ok() {
        let _ = tokio::fs::rename(&tmp, app.data.join("servers.json")).await;
    }
}

/// Ouvre ou ferme les ports dans le pare-feu Windows (réseau local seulement).
async fn firewall(s: &Server, open: bool) {
    if !cfg!(windows) {
        return; // shortcut: pare-feu Linux non géré (ufw/nftables varient), à ouvrir à la main
    }
    for (port, tcp, udp) in s.ports() {
        for (proto, on) in [("TCP", tcp), ("UDP", udp)] {
            if !on {
                continue;
            }
            let name = format!("name=Foyer {} {proto} {port}", s.id);
            let mut c = Command::new("netsh");
            c.args(["advfirewall", "firewall"]);
            if open {
                c.args(["add", "rule", &name, "dir=in", "action=allow", &format!("protocol={proto}"), &format!("localport={port}"), "remoteip=localsubnet"]);
            } else {
                c.args(["delete", "rule", &name]);
            }
            let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).status().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_format() {
        let fp = super::fingerprint(b"abc");
        assert_eq!(fp.len(), 32 * 3 - 1);
        assert!(fp.starts_with("BA:78:16:BF"));
    }

    #[test]
    fn blocage_mauvais_jetons() {
        let (mut f, t) = (HashMap::new(), Instant::now());
        let (a, b): (IpAddr, IpAddr) = ("10.0.0.1".parse().unwrap(), "10.0.0.2".parse().unwrap());
        for _ in 0..MAX_FAILURES - 1 {
            assert!(!note_failure(&mut f, a, t));
        }
        assert!(!is_blocked(&f, a, t));
        assert!(note_failure(&mut f, a, t));
        assert!(is_blocked(&f, a, t) && !is_blocked(&f, b, t));
        assert!(!is_blocked(&f, a, t + BLOCK));
        assert!(!note_failure(&mut f, a, t + BLOCK));
        assert_eq!(f[&a].len(), 1);
    }

    #[test]
    fn retry_limit() {
        let mut c = VecDeque::new();
        let t = Instant::now();
        assert!(should_retry(&mut c, t));
        assert!(should_retry(&mut c, t));
        assert!(should_retry(&mut c, t));
        assert!(!should_retry(&mut c, t + Duration::from_secs(60)));
        // après 10 min les vieux crashs ne comptent plus
        assert!(should_retry(&mut c, t + Duration::from_secs(601)));
    }

    #[test]
    fn taille_de_dossier() {
        let d = std::env::temp_dir().join(format!("foyer-d-{:x}", rand::random::<u64>()));
        std::fs::create_dir_all(d.join("a/b")).unwrap();
        std::fs::write(d.join("x"), [0; 10]).unwrap();
        std::fs::write(d.join("a/b/y"), [0; 5]).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&d, d.join("a/boucle")).unwrap();
        assert_eq!(dir_size(&d), 15);
        std::fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn old_servers_json_loads() {
        let s: Server = serde_json::from_str(r#"{"id":"a","name":"x","game":"minecraft","port":25565,"ram_mb":1024}"#).unwrap();
        assert!(!s.auto_start && !s.auto_restart);
    }
}

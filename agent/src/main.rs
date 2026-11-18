//! Foyer agent : installe, lance et surveille des serveurs de jeu.
//! API HTTP + WebSocket protégée par un jeton, à garder sur le réseau local.
mod games;

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path as UrlPath, Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use games::Game;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
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
}

#[derive(Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Installing,
    Stopped,
    Starting,
    Running,
    Stopping,
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
}

impl Live {
    fn new(status: Status) -> Self {
        Live { status, stdin: None, pid: None, log: VecDeque::new(), tx: broadcast::channel(256).0, cpu: 0.0, mem: 0 }
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
}

type S = State<Arc<App>>;

#[tokio::main]
async fn main() -> Result<()> {
    let data = std::env::var("FOYER_DATA").map(PathBuf::from).unwrap_or_else(|_| {
        if cfg!(windows) { PathBuf::from(r"C:\Foyer") } else { PathBuf::from("/var/lib/foyer") }
    });
    tokio::fs::create_dir_all(data.join("servers")).await?;
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
    let app = Arc::new(App { data: data.clone(), token, servers: Mutex::new(servers), live: Mutex::new(live), host: Default::default() });

    tokio::spawn(stats_loop(app.clone()));

    let api = Router::new()
        .route("/api/system", get(system))
        .route("/api/shutdown", post(shutdown))
        .route("/api/servers", get(list).post(create))
        .route("/api/servers/{id}", axum::routing::delete(remove))
        .route("/api/servers/{id}/{action}", post(action))
        .route("/api/servers/{id}/console", get(console))
        .layer(middleware::from_fn_with_state(app.clone(), auth))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(app);

    let addr = std::env::var("FOYER_ADDR").unwrap_or_else(|_| "0.0.0.0:7878".into());
    println!("Foyer agent sur http://{addr}  données : {}", data.display());
    println!("Jeton : fichier {}", data.join("token").display());
    let l = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(l, api).await?;
    Ok(())
}

#[derive(Deserialize)]
struct TokenQ {
    token: Option<String>,
}

async fn auth(State(app): S, Query(q): Query<TokenQ>, req: Request, next: Next) -> Response {
    let header = req.headers().get("authorization").and_then(|h| h.to_str().ok()).and_then(|h| h.strip_prefix("Bearer "));
    // le WebSocket du navigateur ne peut pas envoyer d'en-tête : jeton aussi accepté en paramètre
    let given = header.map(str::to_string).or(q.token).unwrap_or_default();
    if req.method() == axum::http::Method::OPTIONS || eq_ct(given.as_bytes(), app.token.as_bytes()) {
        next.run(req).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
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
            admin_port: (s.game == Game::Fivem).then(|| games::txadmin_port(s.port)),
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
    let used: Vec<u16> = servers.iter().flat_map(|s| s.game.ports(s.port)).map(|p| p.0).collect();
    if n.game.ports(port).iter().any(|p| used.contains(&p.0)) {
        return bad("Port déjà utilisé par un autre serveur");
    }
    let id: String = (0..8).map(|_| format!("{:x}", rand::random::<u8>() % 16)).collect();
    let s = Server { id: id.clone(), name, game: n.game, port, ram_mb };
    servers.push(s.clone());
    save(&app, &servers).await;
    drop(servers);
    app.live.lock().await.insert(id.clone(), Live::new(Status::Installing));

    let app2 = app.clone();
    let s2 = s.clone();
    tokio::spawn(async move {
        let dir = app2.data.join("servers").join(&s2.id);
        let log = |m: String| {
            let app3 = app2.clone();
            let id = s2.id.clone();
            tokio::spawn(async move { if let Some(l) = app3.live.lock().await.get_mut(&id) { l.push(format!("[foyer] {m}")) } });
        };
        let r = games::install(s2.game, &app2.data, &dir, s2.port, &log).await;
        if r.is_ok() {
            firewall(&s2, true).await;
        }
        let mut live = app2.live.lock().await;
        if let Some(l) = live.get_mut(&s2.id) {
            match r {
                Ok(()) => l.status = Status::Stopped,
                Err(e) => {
                    l.status = Status::Error;
                    l.push(format!("[foyer] Erreur : {e:#}"));
                }
            }
        }
    });
    Ok(Json(s))
}

async fn remove(State(app): S, UrlPath(id): UrlPath<String>) -> StatusCode {
    stop(&app, &id).await;
    let mut servers = app.servers.lock().await;
    let Some(i) = servers.iter().position(|s| s.id == id) else { return StatusCode::NOT_FOUND };
    let s = servers.remove(i);
    save(&app, &servers).await;
    drop(servers);
    app.live.lock().await.remove(&id);
    firewall(&s, false).await;
    // shortcut: suppression définitive du dossier, pas de corbeille ; ajouter une sauvegarde avant si besoin
    let _ = tokio::fs::remove_dir_all(app.data.join("servers").join(&id)).await;
    StatusCode::NO_CONTENT
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

async fn start(app: &Arc<App>, id: &str) -> Result<()> {
    let s = app.servers.lock().await.iter().find(|s| s.id == id).cloned().ok_or_else(|| anyhow::anyhow!("serveur inconnu"))?;
    {
        let live = app.live.lock().await;
        let st = live.get(id).map(|l| l.status);
        if !matches!(st, Some(Status::Stopped | Status::Error)) {
            anyhow::bail!("le serveur n'est pas arrêté");
        }
    }
    let dir = app.data.join("servers").join(id);
    let (exe, args) = games::command(s.game, &app.data, &dir, s.port, s.ram_mb).await?;
    let mut cmd = Command::new(&exe);
    cmd.args(&args).current_dir(&dir).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(false);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    #[cfg(unix)]
    cmd.process_group(0);
    let mut child: Child = cmd.spawn()?;
    let pid = child.id();
    {
        let mut live = app.live.lock().await;
        let l = live.get_mut(id).unwrap();
        l.status = Status::Starting;
        l.stdin = child.stdin.take();
        l.pid = pid;
        l.push(format!("[foyer] Démarrage : {}", exe.display()));
    }
    for out in [child.stdout.take().map(|o| Box::new(o) as Box<dyn tokio::io::AsyncRead + Unpin + Send>),
                child.stderr.take().map(|o| Box::new(o) as Box<dyn tokio::io::AsyncRead + Unpin + Send>)].into_iter().flatten() {
        let app = app.clone();
        let id = id.to_string();
        let marker = s.game.ready_marker();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut live = app.live.lock().await;
                if let Some(l) = live.get_mut(&id) {
                    if l.status == Status::Starting && line.contains(marker) {
                        l.status = Status::Running;
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
        let mut live = app.live.lock().await;
        if let Some(l) = live.get_mut(&id) {
            let crashed = l.status != Status::Stopping;
            l.status = if crashed { Status::Error } else { Status::Stopped };
            l.stdin = None;
            l.pid = None;
            l.cpu = 0.0;
            l.mem = 0;
            l.push(format!("[foyer] Processus terminé (code {code:?})"));
        }
    });
    Ok(())
}

async fn stop(app: &Arc<App>, id: &str) {
    let game = app.servers.lock().await.iter().find(|s| s.id == id).map(|s| s.game);
    let pid = {
        let mut live = app.live.lock().await;
        let Some(l) = live.get_mut(id) else { return };
        let Some(pid) = l.pid else { return };
        l.status = Status::Stopping;
        l.push("[foyer] Arrêt…".into());
        if let (Some(cmd), Some(stdin)) = (game.and_then(|g| g.stop_command()), l.stdin.as_mut()) {
            let _ = stdin.write_all(format!("{cmd}\n").as_bytes()).await;
        }
        pid
    };
    // attend l'arrêt propre 30 s, sinon tue l'arbre de processus
    for _ in 0..60 {
        if app.live.lock().await.get(id).map_or(true, |l| l.pid.is_none()) {
            return;
        }
        if game.and_then(|g| g.stop_command()).is_none() {
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

async fn console(State(app): S, UrlPath(id): UrlPath<String>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| console_ws(app, id, socket))
}

async fn console_ws(app: Arc<App>, id: String, mut socket: WebSocket) {
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
    for (port, tcp, udp) in s.game.ports(s.port) {
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

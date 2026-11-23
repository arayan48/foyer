//! Accès depuis Internet par playit.gg : liaison du compte, un tunnel par serveur,
//! et le tunnel lui-même qui tourne dans l'agent (crates playit-agent, BSD-2).
use crate::{games::Game, App, Server, S};
use axum::extract::Path as UrlPath;
use axum::http::StatusCode;
use axum::Json;
use playit_agent_core::network::origin_lookup::OriginLookup;
use playit_agent_core::network::tcp::tcp_settings::TcpSettings;
use playit_agent_core::network::udp::udp_settings::UdpSettings;
use playit_agent_core::playit_agent::{PlayitAgent, PlayitAgentSettings};
use playit_api_client::api::*;
use playit_api_client::PlayitApi;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const API: &str = "https://api.playit.gg";

type Err = (StatusCode, String);

#[derive(Default, Serialize, Deserialize)]
struct Conf {
    secret_key: String,
    /// id du serveur Foyer → id du tunnel playit
    #[serde(default)]
    tunnels: BTreeMap<String, uuid::Uuid>,
}

/// un seul lecteur-écrivain de playit.json à la fois
static FILE: Mutex<()> = Mutex::const_new(());
/// liaison en cours : (code, état)
static LINK: Mutex<Option<(String, &'static str)>> = Mutex::const_new(None);
/// arrête le tunnel en cours (après déliaison)
static CANCEL: std::sync::Mutex<Option<Box<dyn Fn() + Send>>> = std::sync::Mutex::new(None);

async fn load(data: &Path) -> Option<Conf> {
    let b = tokio::fs::read(data.join("playit.json")).await.ok()?;
    serde_json::from_slice::<Conf>(&b).ok().filter(|c| !c.secret_key.is_empty())
}

async fn store(data: &Path, c: &Conf) -> Result<(), Err> {
    tokio::fs::write(data.join("playit.json"), serde_json::to_vec_pretty(c).unwrap()).await.map_err(fail)
}

fn fail(e: impl std::fmt::Debug) -> Err {
    (StatusCode::BAD_GATEWAY, format!("playit.gg : {e:?}"))
}

fn not_linked() -> Err {
    (StatusCode::CONFLICT, "Aucun compte playit.gg relié".into())
}

fn api(c: &Conf) -> PlayitApi {
    PlayitApi::create(API.into(), Some(c.secret_key.clone()))
}

/// Tâche de fond : fait tourner le tunnel tant qu'un compte est relié.
pub async fn run_loop(app: Arc<App>) {
    let lookup = Arc::new(OriginLookup::default());
    let mut wait = 3;
    loop {
        let Some(c) = load(&app.data).await else {
            tokio::time::sleep(Duration::from_secs(3)).await;
            continue;
        };
        let api = api(&c);
        if let Ok(d) = api.v1_agents_rundata().await {
            lookup.update_from_run_data(&d).await;
        }
        let settings = PlayitAgentSettings { api_url: API.into(), secret_key: c.secret_key.clone(), tcp_settings: TcpSettings::default(), udp_settings: UdpSettings::default() };
        let agent = match PlayitAgent::new(settings, lookup.clone()).await {
            Ok(a) => a,
            Err(e) => {
                crate::journal::agent(&format!("tunnel playit.gg : {e:?}"));
                tokio::time::sleep(Duration::from_secs(wait)).await;
                wait = (wait * 2).min(300);
                continue;
            }
        };
        wait = 3;
        let token = agent.cancellation_token();
        *CANCEL.lock().unwrap() = Some(Box::new(move || token.cancel()));
        // les tunnels créés ensuite sont pris en compte sans relancer
        let (l, a) = (lookup.clone(), api.clone());
        let refresh = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(20)).await;
                if let Ok(d) = a.v1_agents_rundata().await {
                    l.update_from_run_data(&d).await;
                }
            }
        });
        agent.run().await;
        refresh.abort();
        *CANCEL.lock().unwrap() = None;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

#[derive(Serialize)]
pub struct Account {
    linked: bool,
    /// « ready », « guest », « email-not-verified »… (null si playit.gg ne répond pas)
    status: Option<serde_json::Value>,
}

pub async fn account(axum::extract::State(app): S) -> Json<Account> {
    let Some(c) = load(&app.data).await else { return Json(Account { linked: false, status: None }) };
    let status = api(&c).agents_rundata().await.ok().and_then(|d| serde_json::to_value(d.account_status).ok());
    Json(Account { linked: true, status })
}

#[derive(Serialize)]
pub struct Link {
    url: String,
    state: &'static str,
}

/// Lance la liaison : l'appli ouvre l'adresse, l'agent attend l'accord en fond.
pub async fn link_start(axum::extract::State(app): S) -> Json<Link> {
    let code: String = (0..5).map(|_| format!("{:02x}", rand::random::<u8>())).collect();
    *LINK.lock().await = Some((code.clone(), "waiting"));
    let url = format!("https://playit.gg/claim/{code}");
    tokio::spawn(async move {
        let state = claim(&app.data, &code).await;
        let mut l = LINK.lock().await;
        if l.as_ref().is_some_and(|(c, _)| *c == code) {
            *l = Some((code, state));
        }
    });
    Json(Link { url, state: "waiting" })
}

async fn claim(data: &Path, code: &str) -> &'static str {
    let api = PlayitApi::create(API.into(), None);
    let version = format!("Foyer {}", env!("CARGO_PKG_VERSION"));
    for _ in 0..600 {
        let r = api.claim_setup(ReqClaimSetup { code: code.into(), agent_type: ClaimAgentType::SelfManaged, version: version.clone() }).await;
        match r {
            Ok(ClaimSetupResponse::UserAccepted) => break,
            Ok(ClaimSetupResponse::UserRejected) => return "rejected",
            Ok(_) => {}
            Err(ApiError::Fail(_)) => return "expired",
            Err(_) => {} // réseau : on réessaie
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    for _ in 0..30 {
        if let Ok(k) = api.claim_exchange(ReqClaimExchange { code: code.into() }).await {
            let _f = FILE.lock().await;
            let c = Conf { secret_key: k.secret_key, tunnels: Default::default() };
            return if store(data, &c).await.is_ok() { "done" } else { "error" };
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    "expired"
}

pub async fn link_state() -> Json<Option<&'static str>> {
    Json(LINK.lock().await.as_ref().map(|(_, s)| *s))
}

/// Délie : supprime les tunnels créés par Foyer, oublie la clé, arrête le tunnel.
pub async fn unlink(axum::extract::State(app): S) -> StatusCode {
    let _f = FILE.lock().await;
    if let Some(c) = load(&app.data).await {
        let api = api(&c);
        for id in c.tunnels.values() {
            let _ = api.tunnels_delete(ReqTunnelsDelete { tunnel_id: *id }).await;
        }
    }
    let _ = tokio::fs::remove_file(app.data.join("playit.json")).await;
    if let Some(cancel) = CANCEL.lock().unwrap().as_ref() {
        cancel();
    }
    *LINK.lock().await = None;
    StatusCode::NO_CONTENT
}

#[derive(Serialize)]
pub struct Tunnel {
    /// adresse à donner aux joueurs, null tant que playit.gg la prépare
    address: Option<String>,
    disabled: Option<String>,
}

pub async fn get(axum::extract::State(app): S, UrlPath(id): UrlPath<String>) -> Result<Json<Option<Tunnel>>, Err> {
    let Some(c) = load(&app.data).await else { return Ok(Json(None)) };
    let Some(tid) = c.tunnels.get(&id) else { return Ok(Json(None)) };
    let d = api(&c).v1_agents_rundata().await.map_err(fail)?;
    Ok(Json(Some(match d.tunnels.iter().find(|t| t.id == *tid) {
        Some(t) => Tunnel { address: Some(t.display_address.clone()), disabled: t.disabled_reason.as_ref().map(|r| r.to_string()) },
        None => Tunnel { address: None, disabled: None },
    })))
}

pub async fn add(axum::extract::State(app): S, UrlPath(id): UrlPath<String>) -> Result<StatusCode, Err> {
    let s = app.servers.lock().await.iter().find(|s| s.id == id).cloned().ok_or((StatusCode::NOT_FOUND, "serveur inconnu".into()))?;
    let _f = FILE.lock().await;
    let mut c = load(&app.data).await.ok_or_else(not_linked)?;
    if c.tunnels.contains_key(&id) {
        return Ok(StatusCode::NO_CONTENT);
    }
    let tid = create(&c, &s).await?;
    c.tunnels.insert(id, tid);
    store(&app.data, &c).await?;
    Ok(StatusCode::CREATED)
}

async fn create(c: &Conf, s: &Server) -> Result<uuid::Uuid, Err> {
    let api = api(c);
    let agent_id = api.v1_agents_rundata().await.map_err(fail)?.agent_id;
    let (tunnel_type, port_type) = match s.game {
        Game::Minecraft => (Some(TunnelType::MinecraftJava), PortType::Tcp),
        Game::Fivem => (None, PortType::Both),
    };
    let req = ReqTunnelsCreate {
        name: Some(format!("Foyer · {}", s.name)),
        tunnel_type,
        port_type,
        port_count: 1,
        origin: TunnelOriginCreate::Agent(AssignedAgentCreate { agent_id, local_ip: [127, 0, 0, 1].into(), local_port: Some(s.port) }),
        enabled: true,
        alloc: None,
        firewall_id: None,
        proxy_protocol: None,
    };
    match api.tunnels_create(req).await {
        Ok(o) => Ok(o.id),
        Err(ApiError::Fail(e)) => Err((StatusCode::CONFLICT, format!("playit.gg refuse le tunnel : {e:?}"))),
        Err(e) => Err(fail(e)),
    }
}

pub async fn remove(axum::extract::State(app): S, UrlPath(id): UrlPath<String>) -> Result<StatusCode, Err> {
    forget(&app.data, &id).await.map(|_| StatusCode::NO_CONTENT)
}

/// Supprime le tunnel d'un serveur s'il en a un (serveur supprimé).
pub async fn forget(data: &Path, id: &str) -> Result<(), Err> {
    let _f = FILE.lock().await;
    let Some(mut c) = load(data).await else { return Ok(()) };
    let Some(tid) = c.tunnels.remove(id) else { return Ok(()) };
    match api(&c).tunnels_delete(ReqTunnelsDelete { tunnel_id: tid }).await {
        Ok(()) | Err(ApiError::Fail(DeleteError::TunnelNotFound)) => store(data, &c).await,
        Err(e) => Err(fail(e)),
    }
}

/// Port changé : recrée le tunnel sur le nouveau port.
pub async fn moved(data: &Path, s: &Server) -> Result<(), Err> {
    let had = load(data).await.is_some_and(|c| c.tunnels.contains_key(&s.id));
    if !had {
        return Ok(());
    }
    forget(data, &s.id).await?;
    let _f = FILE.lock().await;
    let mut c = load(data).await.ok_or_else(not_linked)?;
    let tid = create(&c, s).await?;
    c.tunnels.insert(s.id.clone(), tid);
    store(data, &c).await
}

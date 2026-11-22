//! Liaison avec l'agent en HTTPS. Le certificat de l'agent est auto-signé : on n'accepte que celui
//! dont l'empreinte a été vérifiée par l'utilisateur à la connexion (comme SSH).
use futures_util::{SinkExt, StreamExt};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};

const MISMATCH: &str = "EMPREINTE";

pub fn fingerprint(der: &[u8]) -> String {
  ring::digest::digest(&ring::digest::SHA256, der).as_ref().iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":")
}

#[derive(Debug)]
struct Pin {
  /// None : simple lecture de l'empreinte, aucune donnée envoyée derrière
  want: Option<String>,
  seen: Mutex<Option<String>>,
  provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for Pin {
  fn verify_server_cert(&self, end: &CertificateDer, _: &[CertificateDer], _: &ServerName, _: &[u8], _: UnixTime) -> Result<ServerCertVerified, rustls::Error> {
    let got = fingerprint(end);
    let ok = self.want.as_ref().is_none_or(|w| w.eq_ignore_ascii_case(&got));
    *self.seen.lock().unwrap() = Some(got);
    if ok { Ok(ServerCertVerified::assertion()) } else { Err(rustls::Error::General(MISMATCH.into())) }
  }
  fn verify_tls12_signature(&self, m: &[u8], c: &CertificateDer, d: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
    rustls::crypto::verify_tls12_signature(m, c, d, &self.provider.signature_verification_algorithms)
  }
  fn verify_tls13_signature(&self, m: &[u8], c: &CertificateDer, d: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
    rustls::crypto::verify_tls13_signature(m, c, d, &self.provider.signature_verification_algorithms)
  }
  fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
    self.provider.signature_verification_algorithms.supported_schemes()
  }
}

fn tls(want: Option<String>) -> (Arc<ClientConfig>, Arc<Pin>) {
  let provider = Arc::new(rustls::crypto::ring::default_provider());
  let pin = Arc::new(Pin { want, seen: Mutex::new(None), provider: provider.clone() });
  let cfg = ClientConfig::builder_with_provider(provider)
    .with_safe_default_protocol_versions()
    .expect("TLS")
    .dangerous()
    .with_custom_certificate_verifier(pin.clone())
    .with_no_client_auth();
  (Arc::new(cfg), pin)
}

fn https(url: &str) -> Result<(), String> {
  if url.starts_with("https://") { Ok(()) } else { Err("Adresse non chiffrée refusée".into()) }
}

fn err(e: impl std::fmt::Debug) -> String {
  let s = format!("{e:?}");
  if s.contains(MISMATCH) { MISMATCH.into() } else { "Agent injoignable".into() }
}

/// Un client par empreinte : garde la connexion ouverte entre deux appels.
fn client(fp: &str) -> reqwest::Client {
  static C: OnceLock<Mutex<HashMap<String, reqwest::Client>>> = OnceLock::new();
  let mut m = C.get_or_init(Default::default).lock().unwrap();
  m.entry(fp.to_string())
    .or_insert_with(|| {
      let (cfg, _) = tls(Some(fp.to_string()));
      reqwest::Client::builder().use_preconfigured_tls((*cfg).clone()).timeout(Duration::from_secs(30)).build().expect("client")
    })
    .clone()
}

/// Lit l'empreinte du certificat présenté, sans envoyer le jeton.
#[tauri::command]
pub async fn agent_probe(url: String) -> Result<String, String> {
  https(&url)?;
  let (cfg, pin) = tls(None);
  let c = reqwest::Client::builder().use_preconfigured_tls((*cfg).clone()).timeout(Duration::from_secs(8)).build().map_err(err)?;
  let _ = c.get(format!("{url}/api/system")).send().await; // 401 attendu, seule l'empreinte compte
  pin.seen.lock().unwrap().clone().ok_or_else(|| "Agent injoignable".into())
}

#[tauri::command]
pub async fn agent_call(url: String, fp: String, token: String, method: String, path: String, body: Option<String>) -> Result<(u16, String), String> {
  https(&url)?;
  let m = reqwest::Method::from_bytes(method.as_bytes()).map_err(err)?;
  // arrêter un serveur peut prendre 35 s
  let limit = Duration::from_secs(if m == reqwest::Method::GET { 30 } else { 60 });
  let mut r = client(&fp).request(m, format!("{url}{path}")).bearer_auth(token).timeout(limit);
  if let Some(b) = body {
    r = r.header("Content-Type", "application/json").body(b);
  }
  let res = r.send().await.map_err(err)?;
  Ok((res.status().as_u16(), res.text().await.map_err(err)?))
}

/// Un gros fichier dépasse les 30 s des appels ordinaires.
const TRANSFER: Duration = Duration::from_secs(3600);

async fn check(res: reqwest::Response) -> Result<reqwest::Response, String> {
  if res.status().is_success() {
    return Ok(res);
  }
  let st = res.status();
  let text = res.text().await.unwrap_or_default();
  Err(if text.is_empty() { format!("Erreur {}", st.as_u16()) } else { text })
}

/// Corps brut : le contenu du fichier ; l'adresse et le jeton arrivent en en-têtes.
#[tauri::command]
pub async fn file_upload(request: tauri::ipc::Request<'_>) -> Result<(), String> {
  let h = |k: &str| request.headers().get(k).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
  let (url, fp, token, path) = (h("url"), h("fp"), h("token"), h("path"));
  https(&url)?;
  let tauri::ipc::InvokeBody::Raw(body) = request.body() else { return Err("Fichier illisible".into()) };
  let res = client(&fp).put(format!("{url}{path}")).bearer_auth(token).timeout(TRANSFER).body(body.clone()).send().await.map_err(err)?;
  check(res).await.map(|_| ())
}

/// Nom de fichier seul, sans chemin : il vient de l'agent.
fn plain_name(n: &str) -> bool {
  !n.is_empty() && n != "." && n != ".." && !n.contains(['/', '\\', ':', '\0'])
}

/// Crée « nom », sinon « nom (1) »… sans jamais écraser.
fn create_free(dir: &std::path::Path, name: &str) -> Result<(std::fs::File, std::path::PathBuf), String> {
  let (stem, ext) = name.rsplit_once('.').filter(|(s, _)| !s.is_empty()).map_or((name, String::new()), |(s, e)| (s, format!(".{e}")));
  for i in 0..1000 {
    let p = dir.join(if i == 0 { name.to_string() } else { format!("{stem} ({i}){ext}") });
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&p) {
      Ok(f) => return Ok((f, p)),
      Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
      Err(e) => return Err(err(e)),
    }
  }
  Err("Trop de fichiers du même nom".into())
}

/// Télécharge vers le dossier Téléchargements et renvoie le chemin.
#[tauri::command]
pub async fn file_download(app: AppHandle, url: String, fp: String, token: String, path: String, name: String) -> Result<String, String> {
  use std::io::Write;
  https(&url)?;
  if !plain_name(&name) {
    return Err("Nom de fichier invalide".into());
  }
  let dir = app.path().download_dir().map_err(err)?;
  let mut res = check(client(&fp).get(format!("{url}{path}")).bearer_auth(token).timeout(TRANSFER).send().await.map_err(err)?).await?;
  let (mut f, dest) = create_free(&dir, &name)?;
  let done = async {
    while let Some(c) = res.chunk().await.map_err(err)? {
      f.write_all(&c).map_err(err)?;
    }
    f.flush().map_err(err)
  }
  .await;
  drop(f);
  if let Err(e) = done {
    let _ = std::fs::remove_file(&dest);
    return Err(e);
  }
  Ok(dest.display().to_string())
}

fn consoles() -> &'static Mutex<HashMap<String, mpsc::UnboundedSender<String>>> {
  static S: OnceLock<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>> = OnceLock::new();
  S.get_or_init(Default::default)
}

#[derive(Clone, serde::Serialize)]
struct Line {
  key: String,
  /// None : connexion fermée
  line: Option<String>,
}

/// Jeton en en-tête, jamais dans l'URL.
async fn connect_ws(url: &str, fp: &str, token: &str, id: &str) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, String> {
  https(url)?;
  let mut req = format!("wss{}/api/servers/{id}/console", &url[5..]).into_client_request().map_err(err)?;
  req.headers_mut().insert("Authorization", format!("Bearer {token}").parse().map_err(err)?);
  let (cfg, _) = tls(Some(fp.to_string()));
  let (ws, _) = tokio_tungstenite::connect_async_tls_with_config(req, None, false, Some(tokio_tungstenite::Connector::Rustls(cfg)))
    .await
    .map_err(err)?;
  Ok(ws)
}

/// Ouvre la console d'un serveur ; les lignes arrivent par l'évènement « console ».
#[tauri::command]
pub async fn console_open(app: AppHandle, key: String, url: String, fp: String, token: String, id: String) -> Result<(), String> {
  let ws = connect_ws(&url, &fp, &token, &id).await?;
  let (tx, mut rx) = mpsc::unbounded_channel::<String>();
  consoles().lock().unwrap().insert(key.clone(), tx);
  tauri::async_runtime::spawn(async move {
    let (mut w, mut r) = ws.split();
    loop {
      tokio::select! {
        m = r.next() => match m {
          Some(Ok(Message::Text(t))) => { let _ = app.emit("console", Line { key: key.clone(), line: Some(t.to_string()) }); }
          Some(Ok(_)) => {}
          _ => break,
        },
        c = rx.recv() => match c {
          Some(c) => { if w.send(Message::text(c)).await.is_err() { break } }
          None => { let _ = w.close().await; break }
        },
      }
    }
    consoles().lock().unwrap().remove(&key);
    let _ = app.emit("console", Line { key, line: None });
  });
  Ok(())
}

#[tauri::command]
pub fn console_send(key: String, text: String) {
  if let Some(tx) = consoles().lock().unwrap().get(&key) {
    let _ = tx.send(text);
  }
}

#[tauri::command]
pub fn console_close(key: String) {
  consoles().lock().unwrap().remove(&key);
}

#[cfg(test)]
mod tests {
  #[test]
  fn fingerprint_format() {
    assert!(super::fingerprint(b"abc").starts_with("BA:78:16:BF"));
  }

  #[test]
  fn telechargement_sans_ecraser() {
    for n in ["", "..", "a/b", "a\\b", "C:x"] {
      assert!(!super::plain_name(n), "{n}");
    }
    let d = std::env::temp_dir().join(format!("foyer-dl-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    assert_eq!(super::create_free(&d, "a.txt").unwrap().1, d.join("a.txt"));
    assert_eq!(super::create_free(&d, "a.txt").unwrap().1, d.join("a (1).txt"));
    std::fs::remove_dir_all(d).unwrap();
  }
}

/// Contre un vrai agent : FOYER_TEST_URL=https://127.0.0.1:7999 FOYER_TEST_TOKEN=… cargo test
#[cfg(test)]
mod live {
  #[test]
  fn probe_call_and_mismatch() {
    let (Ok(url), Ok(token)) = (std::env::var("FOYER_TEST_URL"), std::env::var("FOYER_TEST_TOKEN")) else { return };
    tauri::async_runtime::block_on(async {
      let fp = super::agent_probe(url.clone()).await.unwrap();
      let (st, _) = super::agent_call(url.clone(), fp.clone(), token.clone(), "GET".into(), "/api/system".into(), None).await.unwrap();
      assert_eq!(st, 200);
      let bad = "00".repeat(32);
      assert_eq!(super::agent_call(url.clone(), bad, token.clone(), "GET".into(), "/api/system".into(), None).await.unwrap_err(), super::MISMATCH);
      assert!(super::agent_probe(url.replace("https", "http")).await.is_err());
      // console : jeton en en-tête accepté, mauvais jeton refusé
      assert!(super::connect_ws(&url, &fp, &token, "x").await.is_ok());
      assert!(super::connect_ws(&url, &fp, "mauvais", "x").await.is_err());
    });
  }
}

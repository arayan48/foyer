//! Pont vers un agent joint par son identifiant (iroh) : un port local sur 127.0.0.1 dont chaque
//! connexion TCP devient un flux chiffré vers l'agent. Le reste du code y parle en HTTPS comme à
//! n'importe quel agent : certificat épinglé et jeton inchangés.
use iroh::endpoint::{presets, Connection};
use iroh::{Endpoint, EndpointId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex as AsyncMutex, OnceCell};

const ALPN: &[u8] = b"foyer/1";

async fn endpoint() -> Result<&'static Endpoint, String> {
  static EP: OnceCell<Endpoint> = OnceCell::const_new();
  EP.get_or_try_init(|| async { Endpoint::bind(presets::N0).await.map_err(|e| format!("Accès à distance indisponible : {e}")) }).await
}

/// Adresse locale `https://127.0.0.1:<port>` qui mène à l'agent `id` ; un seul pont par identifiant.
pub async fn bridge(id: &str) -> Result<String, String> {
  static PORTS: Mutex<Option<HashMap<String, u16>>> = Mutex::new(None);
  if let Some(p) = PORTS.lock().unwrap().get_or_insert_default().get(id) {
    return Ok(format!("https://127.0.0.1:{p}"));
  }
  let target: EndpointId = id.trim().parse().map_err(|_| "Identifiant invalide".to_string())?;
  let ep = endpoint().await?;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.map_err(|e| e.to_string())?;
  let port = listener.local_addr().map_err(|e| e.to_string())?.port();
  // deux appels simultanés : on garde le premier pont, l'autre écouteur est abandonné
  let kept = *PORTS.lock().unwrap().get_or_insert_default().entry(id.to_string()).or_insert(port);
  if kept != port {
    return Ok(format!("https://127.0.0.1:{kept}"));
  }
  let conn: Arc<AsyncMutex<Option<Connection>>> = Default::default();
  tauri::async_runtime::spawn(async move {
    while let Ok((tcp, _)) = listener.accept().await {
      let conn = conn.clone();
      tauri::async_runtime::spawn(async move {
        // une connexion iroh réutilisée, rouverte si elle est tombée
        let c = {
          let mut g = conn.lock().await;
          match g.as_ref().filter(|c| c.close_reason().is_none()) {
            Some(c) => c.clone(),
            None => match ep.connect(target, ALPN).await {
              Ok(c) => g.insert(c).clone(),
              Err(_) => return, // agent injoignable : la requête HTTPS échoue côté appelant
            },
          }
        };
        let Ok((mut send, mut recv)) = c.open_bi().await else { return };
        let (mut r, mut w) = tcp.into_split();
        let up = async {
          tokio::io::copy(&mut r, &mut send).await?;
          send.finish().map_err(std::io::Error::other)
        };
        let down = async {
          tokio::io::copy(&mut recv, &mut w).await?;
          w.shutdown().await
        };
        let _ = tokio::try_join!(up, down);
      });
    }
  });
  Ok(format!("https://127.0.0.1:{port}"))
}

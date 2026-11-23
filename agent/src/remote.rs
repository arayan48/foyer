//! Accès à distance par identifiant (iroh) : l'appli joint l'agent par sa clé publique, même derrière
//! une box ou chez un ami, sans ouvrir de port. Connexion directe si possible, sinon par un relais
//! qui ne voit que du chiffré. Chaque flux est relayé tel quel vers l'API locale : HTTPS, certificat
//! épinglé et jeton s'appliquent comme sur le réseau local.
use anyhow::Result;
use axum::Json;
use iroh::endpoint::{presets, Connection};
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::{Endpoint, SecretKey};
use std::path::Path;
use std::sync::OnceLock;
use tokio::io::AsyncWriteExt;

pub const ALPN: &[u8] = b"foyer/1";

static ID: OnceLock<String> = OnceLock::new();

/// Démarre l'écoute ; la clé (donc l'identifiant) est gardée dans `remote.key`.
pub async fn start(data: &Path, port: u16) -> Result<Router> {
    let file = data.join("remote.key");
    let key = match tokio::fs::read(&file).await.ok().and_then(|b| <[u8; 32]>::try_from(b).ok()) {
        Some(b) => SecretKey::from_bytes(&b),
        None => {
            let k = SecretKey::generate();
            tokio::fs::write(&file, k.to_bytes()).await?;
            k
        }
    };
    let ep = Endpoint::builder(presets::N0).secret_key(key).alpns(vec![ALPN.to_vec()]).bind().await?;
    let id = ep.id().to_string();
    tokio::fs::write(data.join("remote-id"), &id).await?;
    let _ = ID.set(id);
    Ok(Router::builder(ep).accept(ALPN, Bridge(port)).spawn())
}

/// GET /api/remote : identifiant à donner à l'appli (null si l'écoute n'a pas démarré).
pub async fn get() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "id": ID.get() }))
}

#[derive(Debug, Clone)]
struct Bridge(u16);

impl ProtocolHandler for Bridge {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        // un flux par connexion TCP côté appli
        while let Ok((mut send, mut recv)) = conn.accept_bi().await {
            let port = self.0;
            tokio::spawn(async move {
                // shortcut: vu de l'agent, le flux vient de 127.0.0.1, donc pas de blocage après 10 mauvais
                // jetons ; le jeton fait 128 bits. À revoir si on veut un blocage par identifiant distant.
                let Ok(tcp) = tokio::net::TcpStream::connect(("127.0.0.1", port)).await else { return };
                let (mut r, mut w) = tcp.into_split();
                let up = async {
                    tokio::io::copy(&mut recv, &mut w).await?;
                    w.shutdown().await
                };
                let down = async {
                    tokio::io::copy(&mut r, &mut send).await?;
                    send.finish().map_err(std::io::Error::other)
                };
                let _ = tokio::try_join!(up, down);
            });
        }
        Ok(())
    }
}

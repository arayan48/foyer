//! Joueurs Minecraft : présents, listes (op, whitelist, bannis) et commandes.
use crate::{mc_plugins, App, Err, Status, S};
use axum::extract::{Path as UrlPath, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// Nom Minecraft, avec le point initial que Floodgate met aux joueurs Bedrock.
fn name_ok(n: &str) -> bool {
    let n = n.strip_prefix('.').unwrap_or(n);
    (1..=16).contains(&n.len()) && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Suit les arrivées et départs dans la console ; un message de chat (« <Bob> … ») ne compte pas.
// shortcut: un plugin qui change ou masque ces messages fausse la liste, passer au Query/RCON si ça gêne
pub fn track(players: &mut BTreeSet<String>, line: &str) {
    let Some((_, msg)) = line.split_once("]: ") else { return };
    if let Some(n) = msg.trim_end().strip_suffix(" joined the game").filter(|n| name_ok(n)) {
        players.insert(n.into());
    } else if let Some(n) = msg.trim_end().strip_suffix(" left the game") {
        players.remove(n);
    }
}

fn command(action: &str, name: &str) -> Option<String> {
    let c = match action {
        "kick" | "ban" | "pardon" | "op" | "deop" => action,
        "whitelist_add" => "whitelist add",
        "whitelist_remove" => "whitelist remove",
        _ => return None,
    };
    name_ok(name).then(|| format!("{c} {name}"))
}

async fn names(file: &Path) -> Vec<String> {
    let v: Vec<Value> = tokio::fs::read(file).await.ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    v.iter().filter_map(|p| p["name"].as_str().map(str::to_string)).collect()
}

pub async fn list(State(app): S, UrlPath(id): UrlPath<String>) -> Result<Json<Value>, Err> {
    let (plugins, ..) = mc_plugins(&app, &id).await?;
    let online = app.live.lock().await.get(&id).map(|l| l.players.clone()).unwrap_or_default();
    Ok(Json(json!({
        "online": online,
        "ops": names(&plugins.with_file_name("ops.json")).await,
        "whitelist": names(&plugins.with_file_name("whitelist.json")).await,
        "banned": names(&plugins.with_file_name("banned-players.json")).await,
    })))
}

pub async fn act(State(app): S, UrlPath((id, name, action)): UrlPath<(String, String, String)>) -> Result<StatusCode, Err> {
    mc_plugins(&app, &id).await?;
    let cmd = command(&action, &name).ok_or((StatusCode::BAD_REQUEST, "Joueur ou action invalide".into()))?;
    send(&app, &id, &cmd).await
}

/// Réglages de `server.properties` devenus des règles de jeu en 1.21.9, renommées en 1.21.11.
fn rule_command(mc: &str, rule: &str, on: bool) -> Option<String> {
    let v = crate::software::key(mc);
    let name = match rule {
        _ if v < vec![1, 21, 9] => return None,
        "pvp" => "pvp",
        "command-blocks" if v < vec![1, 21, 11] => "commandBlocksEnabled",
        "command-blocks" => "command_blocks_work",
        _ => return None,
    };
    Some(format!("gamerule {name} {on}"))
}

pub async fn rule(State(app): S, UrlPath((id, rule, on)): UrlPath<(String, String, bool)>) -> Result<StatusCode, Err> {
    let servers = app.servers.lock().await;
    let label = servers.iter().find(|s| s.id == id && s.game == crate::games::Game::Minecraft).and_then(|s| s.version.clone());
    drop(servers);
    let mc = crate::software::parse(label.as_deref()).1.unwrap_or_default();
    let cmd = rule_command(&mc, &rule, on).ok_or((StatusCode::BAD_REQUEST, "Règle indisponible pour ce serveur".into()))?;
    send(&app, &id, &cmd).await
}

async fn send(app: &App, id: &str, cmd: &str) -> Result<StatusCode, Err> {
    let mut live = app.live.lock().await;
    let off = || (StatusCode::CONFLICT, "Démarre le serveur d'abord".to_string());
    let l = live.get_mut(id).filter(|l| l.status == Status::Running).ok_or_else(off)?;
    l.push(format!("> {cmd}"));
    let stdin = l.stdin.as_mut().ok_or_else(off)?;
    stdin.write_all(format!("{cmd}\n").as_bytes()).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suivi_des_joueurs() {
        let mut p = BTreeSet::new();
        track(&mut p, "[20:00:00 INFO]: Bob joined the game");
        track(&mut p, "[20:00:00] [Server thread/INFO]: .Eve_1 joined the game");
        track(&mut p, "[20:00:00 INFO]: <Bob> Max joined the game");
        track(&mut p, "[20:00:00 INFO]: <Bob> [a]: Max joined the game");
        track(&mut p, "[20:00:00 INFO]: <Max> Bob left the game");
        assert_eq!(p.iter().collect::<Vec<_>>(), [".Eve_1", "Bob"]);
        track(&mut p, "[20:00:00 INFO]: Bob left the game");
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn regles_de_jeu() {
        assert_eq!(rule_command("1.21.10", "command-blocks", true).unwrap(), "gamerule commandBlocksEnabled true");
        assert_eq!(rule_command("26.3", "command-blocks", false).unwrap(), "gamerule command_blocks_work false");
        assert_eq!(rule_command("1.21.11", "pvp", false).unwrap(), "gamerule pvp false");
        assert!(rule_command("1.21.8", "pvp", true).is_none() && rule_command("", "pvp", true).is_none() && rule_command("26.3", "stop", true).is_none());
    }

    #[test]
    fn commandes_sures() {
        assert_eq!(command("whitelist_add", "Bob").unwrap(), "whitelist add Bob");
        for (a, n) in [("stop", "Bob"), ("op", "Bob\nstop"), ("op", "@a"), ("op", ""), ("op", "a b"), ("ban", "abcdefghijklmnopq")] {
            assert!(command(a, n).is_none(), "{a} {n}");
        }
    }
}

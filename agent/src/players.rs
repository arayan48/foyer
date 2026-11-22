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
fn rule_command(mc: &str, rule: &str, on: Option<bool>) -> Option<String> {
    let v = crate::software::key(mc);
    let name = match rule {
        _ if v < vec![1, 21, 9] => return None,
        "pvp" => "pvp",
        "command-blocks" if v < vec![1, 21, 11] => "commandBlocksEnabled",
        "command-blocks" => "command_blocks_work",
        _ => return None,
    };
    Some(on.map_or(format!("gamerule {name}"), |on| format!("gamerule {name} {on}")))
}

/// Règle et valeur annoncées dans la console ; un message de chat (« <Bob> … ») ne compte pas.
pub fn rule_line(line: &str) -> Option<(&'static str, bool)> {
    let (_, msg) = line.split_once("]: ")?;
    let mut msg = msg.trim_end();
    // changée par un joueur : « [Bob: Gamerule … ] »
    if let Some(m) = msg.strip_prefix('[').and_then(|m| m.strip_suffix(']')) {
        let (who, m) = m.split_once(": ")?;
        msg = Some(m).filter(|_| name_ok(who))?;
    }
    let (name, value) = msg.strip_prefix("Gamerule ")?.split_once(" is ")?;
    let value = value.strip_prefix("now set to: ").or(value.strip_prefix("currently set to: "))?;
    let rule = match name.strip_prefix("minecraft:").unwrap_or(name) {
        "pvp" => "pvp",
        "commandBlocksEnabled" | "command_blocks_work" => "command-blocks",
        _ => return None,
    };
    Some((rule, value.parse().ok()?))
}

const RULES: [&str; 2] = ["pvp", "command-blocks"];
type Rules = std::collections::BTreeMap<String, bool>;
static RULES_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn rules_file(app: &App, id: &str) -> std::path::PathBuf {
    app.data.join("servers").join(id).join("foyer-rules.json")
}

async fn read_rules(file: &Path) -> Rules {
    tokio::fs::read(file).await.ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// Retient la valeur d'une règle, pour l'afficher et la remettre au démarrage.
pub async fn save_rule(app: &App, id: &str, rule: &str, on: bool) -> std::io::Result<()> {
    let _guard = RULES_LOCK.lock().await;
    let file = rules_file(app, id);
    let mut r = read_rules(&file).await;
    if r.insert(rule.into(), on) == Some(on) {
        return Ok(());
    }
    tokio::fs::write(&file, serde_json::to_vec(&r)?).await
}

async fn rules_mc(app: &App, id: &str) -> String {
    let servers = app.servers.lock().await;
    let label = servers.iter().find(|s| s.id == id && s.game == crate::games::Game::Minecraft).and_then(|s| s.version.clone());
    crate::software::parse(label.as_deref()).1.unwrap_or_default()
}

/// Serveur prêt : remet les règles choisies, demande la valeur des autres.
pub async fn apply_rules(app: &App, id: &str) {
    let mc = rules_mc(app, id).await;
    let saved = read_rules(&rules_file(app, id)).await;
    for rule in RULES {
        let Some(cmd) = rule_command(&mc, rule, saved.get(rule).copied()) else { return };
        let _ = send(app, id, &cmd).await;
    }
}

/// Valeurs connues ; les deux règles sont actives par défaut dans Minecraft.
pub async fn rules(State(app): S, UrlPath(id): UrlPath<String>) -> Result<Json<Rules>, Err> {
    mc_plugins(&app, &id).await?;
    let mut r: Rules = RULES.iter().map(|k| (k.to_string(), true)).collect();
    r.extend(read_rules(&rules_file(&app, &id)).await);
    Ok(Json(r))
}

/// Tout de suite si le serveur tourne, sinon au prochain démarrage.
pub async fn rule(State(app): S, UrlPath((id, rule, on)): UrlPath<(String, String, bool)>) -> Result<StatusCode, Err> {
    let cmd = rule_command(&rules_mc(&app, &id).await, &rule, Some(on)).ok_or((StatusCode::BAD_REQUEST, "Règle indisponible pour ce serveur".into()))?;
    save_rule(&app, &id, &rule, on).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match send(&app, &id, &cmd).await {
        Err((StatusCode::CONFLICT, _)) => Ok(StatusCode::NO_CONTENT),
        r => r,
    }
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
        assert_eq!(rule_command("1.21.10", "command-blocks", Some(true)).unwrap(), "gamerule commandBlocksEnabled true");
        assert_eq!(rule_command("26.3", "command-blocks", Some(false)).unwrap(), "gamerule command_blocks_work false");
        assert_eq!(rule_command("1.21.11", "pvp", None).unwrap(), "gamerule pvp");
        assert!(rule_command("1.21.8", "pvp", None).is_none() && rule_command("", "pvp", None).is_none() && rule_command("26.3", "stop", None).is_none());
    }

    #[test]
    fn regles_dans_la_console() {
        assert_eq!(rule_line("[20:00:00 INFO]: Gamerule pvp is now set to: false"), Some(("pvp", false)));
        assert_eq!(rule_line("[20:00:00] [Server thread/INFO]: Gamerule minecraft:command_blocks_work is currently set to: true"), Some(("command-blocks", true)));
        assert_eq!(rule_line("[20:00:00 INFO]: [Bob: Gamerule pvp is now set to: true]"), Some(("pvp", true)));
        for l in ["[20:00:00 INFO]: <Bob> Gamerule pvp is now set to: true", "[20:00:00 INFO]: <Bob> ]: Gamerule pvp is now set to: true", "[20:00:00 INFO]: [Bob] Gamerule pvp is now set to: true", "[20:00:00 INFO]: [Bob] a: Gamerule pvp is now set to: true]", "[20:00:00 INFO]: Gamerule keepInventory is now set to: true"] {
            assert!(rule_line(l).is_none(), "{l}");
        }
    }

    #[test]
    fn commandes_sures() {
        assert_eq!(command("whitelist_add", "Bob").unwrap(), "whitelist add Bob");
        for (a, n) in [("stop", "Bob"), ("op", "Bob\nstop"), ("op", "@a"), ("op", ""), ("op", "a b"), ("ban", "abcdefghijklmnopq")] {
            assert!(command(a, n).is_none(), "{a} {n}");
        }
    }
}

# Architecture de Foyer

_État : v0.1.0 (2026-11-18)._

## Vue d'ensemble

```
 PC de Rayan                         Machine serveur (.240, Windows Server 2025)
┌──────────────────┐   HTTP + WS    ┌──────────────────────────────────────────┐
│  Foyer Client    │ ─────────────▶ │  foyer-agent.exe  (port 7878, SYSTEM)    │
│  (Tauri+Svelte)  │  jeton Bearer  │   ├─ API REST /api/...                   │
└──────────────────┘                │   ├─ WebSocket console                   │
                                    │   ├─ stats (sysinfo, toutes les 2 s)     │
 Sur le serveur (option) :          │   └─ lance les jeux en processus enfants │
┌──────────────────┐   127.0.0.1    │        ├─ java -jar server.jar (Paper)   │
│  Foyer Serveur   │ ─────────────▶ │        └─ FXServer.exe (+ txAdmin)       │
│  (même appli)    │  jeton lu dans │                                          │
└──────────────────┘  C:\Foyer\token└──────────────────────────────────────────┘
```

Deux programmes :
- **Agent** (`agent/`, Rust) : service sans fenêtre sur la machine qui héberge les jeux. Fait tout le travail.
- **Appli** (`app/`, Tauri 2 + Svelte 5) : interface. Se connecte à un agent par adresse + jeton, ou tout seul à l'agent local.

Deux installeurs construits à partir de la même appli :
| Installeur | Contenu | Config |
|---|---|---|
| `Foyer-Serveur.exe` | appli + agent (sidecar), tâche planifiée au boot, règle pare-feu | `tauri.conf.json` + `tauri.server.json` + `installer.nsh` |
| `Foyer-Client.exe` | appli seule, sans droits admin | `tauri.conf.json` |

## Agent (`agent/src/`)

| Fichier | Rôle |
|---|---|
| `main.rs` | serveur HTTP (axum 0.8), auth, état, cycle de vie des processus, stats, pare-feu, persistance |
| `games.rs` | recettes de jeux : installation (téléchargements, extraction), commande de lancement, ports, commande d'arrêt, marqueur « prêt » |
| `build.rs` | icône Windows de l'exe |

### Données (`FOYER_DATA`, défaut `C:\Foyer` ou `/var/lib/foyer`)
```
token                 jeton (32 hex), créé au premier lancement
servers.json          liste des serveurs (id, name, game, port, ram_mb)
runtime/java25/       Java Temurin 25 partagé (Minecraft)
servers/<id>/         fichiers d'un serveur (server.jar, monde, artifacts FiveM…)
```

### API
Toutes les routes exigent `Authorization: Bearer <jeton>` (ou `?token=` pour le WebSocket).

| Méthode | Route | Effet |
|---|---|---|
| GET | `/api/system` | machine : nom, OS, CPU %, RAM, disque |
| GET | `/api/servers` | serveurs + statut, cpu, mem, admin_port |
| POST | `/api/servers` | crée et installe `{name, game, port?, ram_mb?, eula}` |
| DELETE | `/api/servers/{id}` | arrête, retire le pare-feu, **efface le dossier** |
| POST | `/api/servers/{id}/start\|stop\|restart` | cycle de vie |
| GET (WS) | `/api/servers/{id}/console` | 1000 dernières lignes puis direct ; texte envoyé = commande stdin |
| POST | `/api/shutdown` | arrêt propre de tous les jeux puis de l'agent (avant mise à jour) |

### Statuts
`installing → stopped → starting → running → stopping → stopped`, `error` si install ratée ou processus mort sans demande d'arrêt.
« running » = ligne de log contenant le marqueur du jeu (`Done (` pour Minecraft, `txAdmin` pour FiveM).

### Arrêt
Commande stdin (`stop` pour Minecraft) puis attente 30 s, sinon arbre tué (`taskkill /T /F` ou `kill -- -pgid`).

### Jeux
| Jeu | Source | Lancement | Ports |
|---|---|---|---|
| Minecraft | Paper via `fill.papermc.io/v3` (dernière stable) + Java 25 (Adoptium) | `java -Xms -Xmx -jar server.jar nogui` | TCP port (défaut 25565) |
| FiveM | build « recommended » via `changelogs-live.fivem.net` | `FXServer.exe +set txAdminPort <port+10000>` | TCP+UDP port (30120), TCP txAdmin (40120) |

Extraction avec `tar` du système (bsdtar sous Windows lit zip, tar.gz, tar.xz).

## Appli (`app/`)

| Fichier | Rôle |
|---|---|
| `src/App.svelte` | coquille : barre latérale (machine), navigation liste / création / détail, rafraîchissement 2 s, toasts |
| `src/lib/api.ts` | types, appels HTTP, URL console, textes des jeux et statuts, formatage |
| `src/lib/Connect.svelte` | connexion (adresse + jeton) ; auto-connexion à l'agent local via la commande Tauri `local_token` |
| `src/lib/ServerList.svelte` | liste des serveurs, état, CPU/RAM, actions |
| `src/lib/NewServer.svelte` | création : choix du jeu, nom, RAM, EULA, port libre auto |
| `src/lib/ServerView.svelte` | détail : infos, console WebSocket, commandes, suppression confirmée |
| `src/lib/Update.svelte` | bandeau de mise à jour (plugin updater) |
| `src/lib/GameMark.svelte` | pastille de jeu (icône Lucide + couleur) |
| `src/app.css` | jetons de design (OKLCH, sombre, accent rose braise `oklch(0.72 0.18 357)`), boutons, champs |
| `src-tauri/src/lib.rs` | plugins (updater, process), commande `local_token` |
| `src-tauri/installer.nsh` | hooks NSIS du serveur : tâche planifiée, pare-feu, question interface, arrêt propre avant mise à jour |
| `mock-agent.mjs` | faux agent pour travailler l'interface (`node mock-agent.mjs`, jeton `test`) |

## Build et publication

- Machine de build : **claude-box** (Ubuntu, `/srv/foyer`), Rust + `cargo-xwin` (cible MSVC) + mingw (agent en cible GNU) + NSIS.
- Le Mac n'a pas Rust : on n'y compile pas.
- `./release.sh <version> "<notes>"` : met la version partout, compile agent + 2 installeurs, signe, écrit `client.json` / `serveur.json`, commit, push, crée la release GitHub, copie les exe dans `~/Downloads`.

## Mises à jour

- Plugin `tauri-plugin-updater`. L'appli lit `https://github.com/arayan48/foyer/releases/latest/download/{client|serveur}.json` au lancement puis toutes les 6 h.
- Installeurs signés (minisign). **Clé privée seulement sur claude-box** : `~/.tauri/foyer.key` (sans mot de passe). Clé publique dans `tauri.conf.json`.
- Mise à jour serveur : `PREINSTALL` appelle `/api/shutdown` (sauvegarde des mondes), tue l'agent, remplace les fichiers ; `POSTINSTALL` relance la tâche sans reposer de question.
- Limite connue : serveur installé sans interface = pas de mise à jour automatique.

## Sécurité (état actuel)

- Jeton comparé en temps constant. HTTP **non chiffré** : réseau local uniquement.
- Pare-feu Windows : 7878 et ports de jeu ouverts en `remoteip=localsubnet`.
- Agent et jeux tournent en **SYSTEM** (à changer, voir PLAN).
- Ne jamais rediriger 7878 sur la Freebox.

# Architecture de Foyer

_État : v0.1.0 (2026-11-18)._

## Vue d'ensemble

```
 PC de Rayan                         Machine serveur (.240, Windows Server 2025)
┌──────────────────┐  HTTPS + WSS   ┌──────────────────────────────────────────┐
│  Foyer Client    │ ─────────────▶ │  foyer-agent.exe  (port 7878, SYSTEM)    │
│  (Tauri+Svelte)  │  jeton Bearer  │   ├─ API REST /api/...                   │
└──────────────────┘                │   ├─ WebSocket console                   │
                                    │   ├─ stats (sysinfo, toutes les 2 s)     │
 Sur le serveur (option) :          │   └─ lance les jeux en processus enfants │
┌──────────────────┐   127.0.0.1    │        ├─ java -jar server.jar           │
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
| `main.rs` | serveur HTTPS (axum 0.8 + axum-server/rustls, certificat auto-signé via rcgen), auth, état, cycle de vie des processus, stats, pare-feu, persistance |
| `games.rs` | recettes de jeux : installation (téléchargements, extraction), commande de lancement, ports, commande d'arrêt, marqueur « prêt » |
| `files.rs` | fichiers d'un serveur, chemins bloqués dans `servers/<id>` (liens symboliques compris) |
| `backups.rs` | archives `backups/<id>/<date>.tar.gz`, restauration, reprise après coupure |
| `orphans.rs` | au démarrage de l'agent, arrête les jeux laissés par un agent coupé net |
| `software.rs` | logiciels Minecraft (Vanilla, Paper, Purpur, Fabric, Forge, NeoForge) : versions et installation |
| `players.rs` | joueurs Minecraft : présents, listes, commandes |
| `journal.rs` | journal des actions (`actions.log`, 1 Mo puis `actions.log.1`) |
| `trash.rs` | corbeille des serveurs supprimés |
| `build.rs` | icône Windows de l'exe |

### Données (`FOYER_DATA`, défaut `C:\Foyer` ou `/var/lib/foyer`)
```
token                 jeton (32 hex), créé au premier lancement
cert.pem, key.pem      certificat auto-signé de l'agent, créé au premier lancement
fingerprint           empreinte SHA-256 du certificat (à comparer à la connexion)
servers.json          liste des serveurs (id, name, game, port, ram_mb, auto_start, auto_restart)
runtime/java25/       Java Temurin 25 partagé (Minecraft)
servers/<id>/         fichiers d'un serveur (server.jar, monde, artifacts FiveM…)
trash/<id>/           serveur supprimé : server.json + files/
actions.log           journal des actions
agent.log             journal technique de l'agent (heures UTC, 1 Mo puis agent.log.1)
tmp/                  envois de fichiers en cours, vidé au démarrage
```

### API
Toutes les routes exigent `Authorization: Bearer <jeton>`, WebSocket compris (jamais dans l'URL).

| Méthode | Route | Effet |
|---|---|---|
| GET | `/api/system` | machine : nom, OS, CPU %, RAM, disque |
| GET | `/api/servers` | serveurs + statut, cpu, mem, disk (octets, calculé toutes les 5 min, `null` avant), admin_port, `uptime_s` (prêt depuis) ; Minecraft : `players`, `max_players` |
| POST | `/api/servers` | crée et installe `{name, game, port?, ram_mb?, eula}` ; Minecraft : `software?`, `version?`, `seed?`, `level_type?`, `hardcore?`, `online_mode?` |
| PATCH | `/api/servers/{id}` | modifie `{name?, port?, ram_mb?, auto_start?, auto_restart?}` ; port : Minecraft seulement, serveur arrêté (réécrit `server.properties`, pare-feu) |
| GET | `/api/catalog/{logiciel}` | versions de Minecraft d'un logiciel, la plus récente d'abord |
| GET / POST | `/api/servers/{id}/backups` | liste `{backups, running, error}` / lance une sauvegarde (à chaud pour Minecraft) |
| POST / DELETE | `/api/servers/{id}/backups/{date}` | restaure (serveur arrêté, état actuel sauvegardé avant) / efface |
| POST | `/api/servers/{id}/software` | Minecraft, serveur arrêté : installe `{software, version}`, garde mondes et réglages |
| GET / PUT | `/api/servers/{id}/properties` | Minecraft : lit / écrit les clés de `server.properties` (`{clé: valeur}`) ; PUT ne touche qu'aux clés envoyées, refuse le port (passer par PATCH) |
| DELETE | `/api/servers/{id}` | arrête, retire le pare-feu, met le dossier à la corbeille ; 409 pendant l'installation |
| GET | `/api/servers/{id}/files?path=` | liste un dossier `[{name, dir, size, modified}]` ; chemins relatifs avec `/` |
| POST / PATCH / DELETE | `/api/servers/{id}/files?path=` | crée un dossier / renomme ou déplace `{to}` sans écraser / efface (définitif) |
| GET / PUT | `/api/servers/{id}/files/content?path=` | télécharge / écrit un fichier (corps brut, sans limite de taille, remplacé une fois tout reçu) |
| GET | `/api/servers/{id}/players` | Minecraft : `{online, ops, whitelist, banned}` (noms) ; présents suivis dans la console |
| POST | `/api/servers/{id}/players/{nom}/{action}` | `kick`, `ban`, `pardon`, `op`, `deop`, `whitelist_add`, `whitelist_remove` ; serveur démarré, sinon 409 |
| POST | `/api/servers/{id}/rules/{pvp\|command-blocks}/{true\|false}` | Minecraft 1.21.9 et plus : règle de jeu, retenue dans `foyer-rules.json` et remise à chaque démarrage |
| GET | `/api/servers/{id}/rules` | valeurs connues de ces règles, suivies dans la console |
| GET | `/api/journal` | 200 dernières actions `{at, ip, action, status}` : requêtes qui changent quelque chose et commandes console |
| GET | `/api/trash` | serveurs supprimés + `deleted_at` |
| POST / DELETE | `/api/trash/{id}` | restaure (409 si le port est pris) / efface définitivement ; effacement auto après 30 jours |
| POST | `/api/servers/{id}/start\|stop\|restart` | cycle de vie |
| GET (WS) | `/api/servers/{id}/console` | 1000 dernières lignes puis direct ; texte envoyé = commande stdin |
| POST | `/api/shutdown` | arrêt propre de tous les jeux puis de l'agent (avant mise à jour) |

### Statuts
`installing → stopped → starting → running → stopping → stopped`, `error` si install ratée ou processus mort sans demande d'arrêt.
Crash + `auto_restart` : relance après 5 s, 3 essais max par fenêtre de 10 min. `auto_start` : lancé au démarrage de l'agent (donc au boot).
« running » = ligne de log contenant le marqueur du jeu (`Done (` pour Minecraft, `txAdmin` pour FiveM).

### Arrêt
Commande stdin (`stop` pour Minecraft) puis attente 30 s, sinon arbre tué (`taskkill /T /F` ou `kill -- -pgid`).

### Jeux
| Jeu | Source | Lancement | Ports |
|---|---|---|---|
| Minecraft | logiciel choisi à la création (Fabric : avec Fabric API), changeable ensuite depuis l'onglet Version ; Java conseillé (Adoptium) | `java -Xms -Xmx -jar server.jar nogui`, ou les arguments de `foyer-launch.txt` (Forge, NeoForge, vieux Vanilla) | TCP port (défaut 25565) |
| FiveM | build « recommended » via `changelogs-live.fivem.net` | `FXServer.exe +set txAdminPort <port+10000>` | TCP+UDP port (30120), TCP txAdmin (40120) |

Extraction avec `tar` du système (bsdtar sous Windows lit zip, tar.gz, tar.xz).

## Appli (`app/`)

| Fichier | Rôle |
|---|---|
| `src/App.svelte` | coquille : barre latérale (machine), navigation liste / création / détail, rafraîchissement 2 s, toasts |
| `src/lib/api.ts` | types, appels à l'agent (via Rust dans l'appli, `fetch` dans un navigateur), console, textes des jeux et statuts, formatage |
| `src/lib/Connect.svelte` | connexion (adresse + jeton) ; vérification de l'empreinte ; auto-connexion à l'agent local via la commande Tauri `local_agent` |
| `src/lib/ServerList.svelte` | liste des serveurs, état, CPU/RAM, actions |
| `src/lib/NewServer.svelte` | création : choix du jeu, logiciel et version, monde, nom, RAM, EULA, port libre auto |
| `src/lib/ServerView.svelte` | détail : infos, console WebSocket, commandes, suppression confirmée |
| `src/lib/ServerView.svelte` → onglets | Console, Général, Réseau, Version, Propriétés (Minecraft) ou FiveM (txAdmin, clé Cfx.re, server.cfg), Joueurs, Fichiers, Sauvegardes. Ce qui n'a pas encore d'API est affiché désactivé avec « Bientôt » |
| `src/lib/ServerFiles.svelte` | fichiers : parcourir, éditer (texte, 1 Mo max), envoyer (500 Mo max), télécharger dans Téléchargements |
| `src/lib/ServerPlayers.svelte` | joueurs Minecraft : connectés, opérateurs, liste blanche, bannis |
| `src/lib/Trash.svelte`, `src/lib/Journal.svelte` | corbeille et journal des actions |
| `src/lib/SaveBar.svelte` | barre « Enregistrer / Annuler » qui apparaît quand une page est modifiée |
| `src/lib/Update.svelte` | bandeau de mise à jour (plugin updater) |
| `src/lib/GameMark.svelte` | pastille de jeu : bloc d'herbe dessiné pour Minecraft, logo FiveM (simple-icons, CC0) |
| `src/app.css` | jetons de design (OKLCH, sombre, accent rose braise `oklch(0.72 0.18 357)`), boutons, champs |
| `src-tauri/src/lib.rs` | plugins (updater, process), commande `local_agent`
| `src-tauri/src/agent.rs` | HTTPS + WSS vers l'agent avec certificat épinglé (rustls + ring, pas d'autorité externe) | |
| `src-tauri/installer.nsh` | hooks NSIS du serveur : tâche planifiée, pare-feu, question interface, arrêt propre avant mise à jour |
| `mock-agent.mjs` | faux agent pour travailler l'interface (`node mock-agent.mjs`, jeton `test`) |

## Build et publication

- Machine de build : **claude-box** (Ubuntu, `/srv/foyer`), Rust + `cargo-xwin` (cible MSVC) + mingw (agent en cible GNU) + NSIS.
- Le Mac n'a pas Rust : l'appli Mac (universelle, .dmg) est construite par GitHub Actions (`.github/workflows/mac.yml`), lancé par `release.sh`.
- `./release.sh <version> "<notes>"` : met la version partout, compile agent + 2 installeurs, signe, écrit `client.json` / `serveur.json`, commit, push, crée la release GitHub, copie les exe dans `~/Downloads`.

## Mises à jour

- Plugin `tauri-plugin-updater`. L'appli lit `https://github.com/arayan48/foyer/releases/latest/download/{client|serveur}.json` au lancement puis toutes les 6 h, télécharge en fond, puis bouton « Redémarrer » : installateur sans fenêtre (`installMode: "quiet"`) et relance.
- Installeurs signés (minisign). **Clé privée seulement sur claude-box** : `~/.tauri/foyer.key` (sans mot de passe). Clé publique dans `tauri.conf.json`.
- Mise à jour serveur : `PREINSTALL` appelle `/api/shutdown` (sauvegarde des mondes), tue l'agent, remplace les fichiers ; `POSTINSTALL` relance la tâche sans reposer de question.
- Limite connue : serveur installé sans interface = pas de mise à jour automatique.

## Sécurité (état actuel)

- 10 mauvais jetons en 10 min : adresse bloquée 10 min (429), sauf la machine elle-même.
- Jeton comparé en temps constant. HTTPS avec certificat auto-signé épinglé (empreinte confirmée par l'utilisateur, comme SSH). Ne pas exposer 7878 sur Internet pour autant.
- Pare-feu Windows : 7878 et ports de jeu ouverts en `remoteip=localsubnet`.
- Agent et jeux tournent en **SYSTEM** (à changer, voir PLAN).
- Ne jamais rediriger 7878 sur la Freebox.

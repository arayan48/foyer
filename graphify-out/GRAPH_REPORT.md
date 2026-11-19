# Graph Report - foyer  (2026-11-19)

## Corpus Check
- 55 files · ~30,593 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 9 file(s) not represented in the graph (top: (none) 6, .nsh 2, .css 1)

## Summary
- 702 nodes · 1111 edges · 52 communities (38 shown, 14 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 15 edges (avg confidence: 0.83)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f5710fce`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- agent/src/main.rs
- App.svelte
- package.json
- Config Tauri
- Recettes de jeux
- Config TS node
- Config TS app
- Concept agent et jeux
- Build, signature, MAJ
- SettingsView.svelte
- Sécurité et accès
- Permissions Tauri
- Installeurs
- Faux agent de test
- lib.rs
- Config TS racine
- Sauvegardes (plan)
- Serveur de test
- Design
- Paquet foyer
- Paquet agent
- README
- ServerNetwork.svelte
- agent.rs
- create
- App
- tls
- Status
- api.ts
- lucide-svelte
- Connect.svelte
- ServerConsole.svelte
- NewServer.svelte
- ServerPlugins.svelte
- limits.rs
- S
- Dashboard.svelte
- ServerVersion.svelte
- README.es.md
- README.fr.md
- ServerGeneral.svelte
- settings.ts
- API REST /api + WebSocket console
- SubPages.svelte
- @tauri-apps/api

## God Nodes (most connected - your core abstractions)
1. `lucide-svelte` - 18 edges
2. `call()` - 17 edges
3. `S` - 15 edges
4. `compilerOptions` - 15 edges
5. `App` - 12 edges
6. `create()` - 12 edges
7. `Server` - 12 edges
8. `install()` - 11 edges
9. `update()` - 11 edges
10. `Conn` - 11 edges

## Surprising Connections (you probably didn't know these)
- `Jeux sous compte Windows dédié` --conceptually_related_to--> `Agent et jeux en SYSTEM`  [INFERRED]
  ROADMAP.md → docs/ARCHITECTURE.md
- `index.html (point d'entrée appli)` --implements--> `Appli Tauri 2 + Svelte 5`  [INFERRED]
  app/index.html → docs/ARCHITECTURE.md
- `FOYER_DATA / FOYER_ADDR` --shares_data_with--> `Auth jeton Bearer`  [INFERRED]
  README.md → docs/ARCHITECTURE.md
- `Foyer (CLAUDE.md)` --references--> `foyer-agent (Rust, axum)`  [EXTRACTED]
  CLAUDE.md → docs/ARCHITECTURE.md
- `Agent Rust (port 7878)` --references--> `foyer-agent (Rust, axum)`  [EXTRACTED]
  README.md → docs/ARCHITECTURE.md

## Import Cycles
- 1-file cycle: `agent/src/limits.rs -> agent/src/limits.rs`

## Hyperedges (group relationships)
- **Chaîne de release et mises à jour** — release, docs_architecture_minisign, docs_architecture_updater, docs_architecture_claude_box [EXTRACTED 1.00]
- **Sécurité agent** — docs_architecture_jeton_bearer, readme_reseau_local, roadmap_https_appairage, roadmap_compte_dedie [INFERRED 0.85]

## Communities (52 total, 14 thin omitted)

### Community 0 - "agent/src/main.rs"
Cohesion: 0.05
Nodes (12): AddPlugin, auth(), console(), console_ws(), CRASH_WINDOW, eq_ct(), kill_tree(), LOG_LINES (+4 more)

### Community 1 - "App.svelte"
Cohesion: 0.13
Nodes (20): act(), address, conn, connect(), current, loading, logout(), refresh() (+12 more)

### Community 2 - "package.json"
Cohesion: 0.05
Nodes (46): dependencies, @fontsource/unbounded@^5.3.0, @fontsource-variable/figtree@^5.3.0, lucide-svelte@^1.0.1, @tauri-apps/api@^2.12.5, @tauri-apps/plugin-opener@^2.7.3, @tauri-apps/plugin-process@^2.4.0, @tauri-apps/plugin-updater@^2.14.1 (+38 more)

### Community 3 - "Config Tauri"
Cohesion: 0.06
Nodes (30): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+22 more)

### Community 4 - "Recettes de jeux"
Cohesion: 0.10
Nodes (29): catalog(), client(), command(), DEFAULT_DB, download(), extract(), fetch_tree(), Game (+21 more)

### Community 5 - "Config TS node"
Cohesion: 0.12
Nodes (16): compilerOptions, allowImportingTsExtensions, erasableSyntaxOnly, lib, module, moduleDetection, noEmit, noFallthroughCasesInSwitch (+8 more)

### Community 6 - "Config TS app"
Cohesion: 0.14
Nodes (13): compilerOptions, allowArbitraryExtensions, allowJs, checkJs, module, moduleDetection, noEmit, target (+5 more)

### Community 7 - "Concept agent et jeux"
Cohesion: 0.15
Nodes (10): Foyer (CLAUDE.md), foyer-agent (Rust, axum), FiveM FXServer + txAdmin, games.rs recettes de jeux, main.rs serveur HTTP, Minecraft Paper + Java 25, Cycle de statuts serveur, v0.2 Paramètres et limites (+2 more)

### Community 8 - "Build, signature, MAJ"
Cohesion: 0.18
Nodes (6): claude-box machine de build, Signature minisign (clé sur claude-box), tauri-plugin-updater via GitHub Releases, v0.9 Industrialisation, release.sh script, CI GitHub Actions

### Community 9 - "SettingsView.svelte"
Cohesion: 0.12
Nodes (13): @tauri-apps/plugin-autostart@^2.7.0, { size = 16 }, autostart, { conn, sys, onlogout }, mac, newVersion, page, pages (+5 more)

### Community 10 - "Sécurité et accès"
Cohesion: 0.29
Nodes (6): Agent et jeux en SYSTEM, v0.6 Sécurité, v0.7 Accès extérieur, Jeux sous compte Windows dédié, HTTPS épinglé + appairage 6 chiffres, Intégration playit.gg

### Community 11 - "Permissions Tauri"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "Installeurs"
Cohesion: 0.40
Nodes (4): POST /api/shutdown, Installeur Foyer-Client.exe, Installeur Foyer-Serveur.exe, installer.nsh hooks NSIS

### Community 13 - "Faux agent de test"
Cohesion: 0.40
Nodes (3): plugins, servers, sys

### Community 14 - "lib.rs"
Cohesion: 0.13
Nodes (3): CLOSE_TO_TRAY, run(), show()

### Community 29 - "ServerNetwork.svelte"
Cohesion: 0.14
Nodes (11): { dirty, busy, error, onsave, onreset }, busy, { conn, server, address }, copied, error, idle, page, pages (+3 more)

### Community 30 - "agent.rs"
Cohesion: 0.07
Nodes (15): agent_call(), agent_probe(), client(), connect_ws(), console_close(), console_open(), console_send(), consoles() (+7 more)

### Community 31 - "create"
Cohesion: 0.44
Nodes (7): create(), firewall(), NewServer, Patch, save(), Server, update()

### Community 32 - "App"
Cohesion: 0.17
Nodes (9): App, HostStats, Live, main(), should_retry(), start(), start_inner(), stats_loop() (+1 more)

### Community 33 - "tls"
Cohesion: 0.50
Nodes (3): fingerprint(), fingerprint_format(), tls()

### Community 34 - "Status"
Cohesion: 0.22
Nodes (9): list(), ServerView, Status, Error, Installing, Running, Starting, Stopped (+1 more)

### Community 35 - "api.ts"
Cohesion: 0.18
Nodes (12): bytes(), Game, GAMES, Server, STATUS, { game, size = 40 }, { servers, address, loading, onopen, onaction, onnew }, canRun (+4 more)

### Community 36 - "lucide-svelte"
Cohesion: 0.18
Nodes (9): { open = $bindable(false), title, text, action = 'Supprimer', onconfirm }, page, pages, { server, address }, showKey, txadmin, page, pages (+1 more)

### Community 37 - "Connect.svelte"
Cohesion: 0.17
Nodes (13): inTauri, normalizeUrl(), probe(), busy, confirm(), error, { onconnect }, pending (+5 more)

### Community 38 - "ServerConsole.svelte"
Cohesion: 0.09
Nodes (14): Console, cmd, { conn, server, address }, connected, copied, cpuPct, filter, full (+6 more)

### Community 39 - "NewServer.svelte"
Cohesion: 0.06
Nodes (29): autoName, busy, canNext, catalog, catalogError, channel, { conn, servers, oncreated, oncancel }, current (+21 more)

### Community 40 - "ServerPlugins.svelte"
Cohesion: 0.07
Nodes (28): base(), f, askDelete, busy, { conn, server }, error, Hit, install() (+20 more)

### Community 41 - "limits.rs"
Cohesion: 0.10
Nodes (20): apply(), AssignProcessToJobObject(), Basic, CloseHandle(), CPU_RATE, CpuRate, CreateJobObjectW(), Extended (+12 more)

### Community 42 - "S"
Cohesion: 0.29
Nodes (13): action(), catalog(), Err, mc_plugins(), plugin_add(), plugin_del(), plugin_search(), plugin_toggle() (+5 more)

### Community 43 - "Dashboard.svelte"
Cohesion: 0.16
Nodes (11): alerts, diskPct, down, hist, live(), memPct, off(), ramParts (+3 more)

### Community 44 - "ServerVersion.svelte"
Cohesion: 0.17
Nodes (12): Catalog, Conn, patchServer(), save(), save(), { conn, server }, java, javaError (+4 more)

### Community 45 - "README.es.md"
Cohesion: 0.15
Nodes (12): Arquitectura, Capturas de pantalla, Compilar desde el código, Configuración, Contribuir, Descripción, Funciones, Hoja de ruta (+4 more)

### Community 46 - "README.fr.md"
Cohesion: 0.15
Nodes (12): Architecture, Captures d'écran, Compiler depuis les sources, Configuration, Contribuer, Feuille de route, Fonctionnalités, Installation (+4 more)

### Community 47 - "ServerGeneral.svelte"
Cohesion: 0.17
Nodes (10): host, busy, confirmDelete, { conn, server, ondelete }, dirty, error, page, pages (+2 more)

### Community 48 - "settings.ts"
Cohesion: 0.24
Nodes (8): applySettings(), applyTheme(), DEFAULTS, loadSettings(), Settings, sysLight, toggleTray(), app

### Community 49 - "API REST /api + WebSocket console"
Cohesion: 0.29
Nodes (7): index.html (point d'entrée appli), Test UI via mock-agent, API REST /api + WebSocket console, Appli Tauri 2 + Svelte 5, Auth jeton Bearer, Application Tauri, FOYER_DATA / FOYER_ADDR

### Community 50 - "SubPages.svelte"
Cohesion: 0.33
Nodes (3): { icon: Icon, title, text, version }, Page, { pages, page = $bindable(), label, onselect, children }

### Community 51 - "@tauri-apps/api"
Cohesion: 1.00
Nodes (3): openConsole(), tauri(), @tauri-apps/api

## Knowledge Gaps
- **29 isolated node(s):** `foyer-agent`, `Minecraft`, `Fivem`, `Installing`, `Stopped` (+24 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 383 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **14 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `lucide-svelte` connect `lucide-svelte` to `App.svelte`, `package.json`, `api.ts`, `Connect.svelte`, `ServerConsole.svelte`, `NewServer.svelte`, `ServerPlugins.svelte`, `SettingsView.svelte`, `Dashboard.svelte`, `ServerVersion.svelte`, `ServerGeneral.svelte`, `ServerNetwork.svelte`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **What connects `foyer-agent`, `Minecraft`, `Fivem` to the rest of the system?**
  _29 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `agent/src/main.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05391120507399577 - nodes in this community are weakly interconnected._
- **Why does `Conn` connect `ServerVersion.svelte` to `App.svelte`, `api.ts`, `Connect.svelte`, `ServerConsole.svelte`, `NewServer.svelte`, `ServerPlugins.svelte`, `SettingsView.svelte`, `ServerGeneral.svelte`, `ServerNetwork.svelte`?**
  _High betweenness centrality (0.008) - this node is a cross-community bridge._
- **Should `App.svelte` be split into smaller, more focused modules?**
  _Cohesion score 0.13043478260869565 - nodes in this community are weakly interconnected._
- **Why does `Server` connect `api.ts` to `App.svelte`, `lucide-svelte`, `ServerConsole.svelte`, `NewServer.svelte`, `ServerPlugins.svelte`, `Dashboard.svelte`, `ServerVersion.svelte`, `ServerGeneral.svelte`, `ServerNetwork.svelte`?**
  _High betweenness centrality (0.008) - this node is a cross-community bridge._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.05333333333333334 - nodes in this community are weakly interconnected._
# Graph Report - foyer  (2026-11-18)

## Corpus Check
- 38 files · ~12,431 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 8 file(s) not represented in the graph (top: (none) 6, .nsh 1, .css 1)

## Summary
- 408 nodes · 585 edges · 35 communities (20 shown, 15 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 9 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ccaa6575`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Agent HTTP et processus
- App.svelte
- Dépendances npm
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
- NewServer.svelte
- ServerView.svelte
- ServerSettings.svelte
- Connect.svelte
- api.ts
- Status

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 15 edges
2. `App` - 11 edges
3. `Status` - 10 edges
4. `S` - 10 edges
5. `create()` - 10 edges
6. `lucide-svelte` - 10 edges
7. `call()` - 10 edges
8. `compilerOptions` - 10 edges
9. `Game` - 9 edges
10. `update()` - 9 edges

## Surprising Connections (you probably didn't know these)
- `Jeux sous compte Windows dédié` --conceptually_related_to--> `Agent et jeux en SYSTEM`  [INFERRED]
  ROADMAP.md → docs/ARCHITECTURE.md
- `Foyer (CLAUDE.md)` --references--> `foyer-agent (Rust, axum)`  [EXTRACTED]
  CLAUDE.md → docs/ARCHITECTURE.md
- `Agent Rust (port 7878)` --references--> `foyer-agent (Rust, axum)`  [EXTRACTED]
  README.md → docs/ARCHITECTURE.md
- `CI GitHub Actions` --conceptually_related_to--> `claude-box machine de build`  [INFERRED]
  ROADMAP.md → docs/ARCHITECTURE.md
- `Droits logos des jeux` --conceptually_related_to--> `Risque : clé de signature unique`  [INFERRED]
  ROADMAP.md → docs/PLAN.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Chaîne de release et mises à jour** — release, docs_architecture_minisign, docs_architecture_updater, docs_architecture_claude_box [EXTRACTED 1.00]
- **Sécurité agent** — docs_architecture_jeton_bearer, readme_reseau_local, roadmap_https_appairage, roadmap_compte_dedie [INFERRED 0.85]

## Communities (35 total, 15 thin omitted)

### Community 0 - "Agent HTTP et processus"
Cohesion: 0.05
Nodes (32): action(), App, auth(), console(), console_ws(), CRASH_WINDOW, create(), eq_ct() (+24 more)

### Community 1 - "App.svelte"
Cohesion: 0.16
Nodes (18): act(), address, conn, connect(), current, loading, logout(), refresh() (+10 more)

### Community 2 - "Dépendances npm"
Cohesion: 0.08
Nodes (31): index.html (point d'entrée appli), @fontsource-variable/inter@^5.3.0, devDependencies, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.6, @tsconfig/svelte@^5.0.8 (+23 more)

### Community 3 - "Config Tauri"
Cohesion: 0.07
Nodes (29): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+21 more)

### Community 4 - "Recettes de jeux"
Cohesion: 0.09
Nodes (14): client(), command(), download(), extract(), Game, Fivem, Minecraft, get_json() (+6 more)

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
Cohesion: 0.07
Nodes (30): dependencies, @tauri-apps/api@^2.12.5, @tauri-apps/plugin-autostart@^2.7.0, @tauri-apps/plugin-process@^2.4.0, @tauri-apps/plugin-updater@^2.14.1, scripts, build, check (+22 more)

### Community 10 - "Sécurité et accès"
Cohesion: 0.29
Nodes (6): Agent et jeux en SYSTEM, v0.6 Sécurité, v0.7 Accès extérieur, Jeux sous compte Windows dédié, HTTPS épinglé + appairage 6 chiffres, Intégration playit.gg

### Community 11 - "Permissions Tauri"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "Installeurs"
Cohesion: 0.40
Nodes (4): POST /api/shutdown, Installeur Foyer-Client.exe, Installeur Foyer-Serveur.exe, installer.nsh hooks NSIS

### Community 14 - "lib.rs"
Cohesion: 0.14
Nodes (3): CLOSE_TO_TRAY, run(), show()

### Community 29 - "NewServer.svelte"
Cohesion: 0.15
Nodes (12): lucide-svelte@^1.0.1, Game, { game, size = 40 }, busy, { conn, servers, oncreated, oncancel }, error, eula, game (+4 more)

### Community 30 - "ServerView.svelte"
Cohesion: 0.17
Nodes (9): consoleUrl(), canRun, cmd, confirmDelete, { conn, server, address, onback, onaction, ondelete }, connected, editing, lines (+1 more)

### Community 31 - "ServerSettings.svelte"
Cohesion: 0.17
Nodes (10): autoRestart, autoStart, busy, { conn, server, onclose }, error, init, name, port (+2 more)

### Community 32 - "Connect.svelte"
Cohesion: 0.22
Nodes (9): AuthError, Conn, normalizeUrl(), busy, error, { onconnect }, submit(), token (+1 more)

### Community 33 - "api.ts"
Cohesion: 0.39
Nodes (5): bytes(), GAMES, Server, STATUS, { servers, address, loading, onopen, onaction, onnew }

### Community 34 - "Status"
Cohesion: 0.29
Nodes (7): Status, Error, Installing, Running, Starting, Stopped, Stopping

## Knowledge Gaps
- **28 isolated node(s):** `foyer-agent`, `Minecraft`, `Fivem`, `Installing`, `Stopped` (+23 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 223 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `lucide-svelte` connect `NewServer.svelte` to `Connect.svelte`, `App.svelte`, `Dépendances npm`, `api.ts`, `SettingsView.svelte`, `ServerView.svelte`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **What connects `foyer-agent`, `Minecraft`, `Fivem` to the rest of the system?**
  _28 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Agent HTTP et processus` be split into smaller, more focused modules?**
  _Cohesion score 0.05297297297297297 - nodes in this community are weakly interconnected._
- **Should `Dépendances npm` be split into smaller, more focused modules?**
  _Cohesion score 0.0766488413547237 - nodes in this community are weakly interconnected._
- **Should `Config Tauri` be split into smaller, more focused modules?**
  _Cohesion score 0.06666666666666667 - nodes in this community are weakly interconnected._
- **Should `Recettes de jeux` be split into smaller, more focused modules?**
  _Cohesion score 0.09462365591397849 - nodes in this community are weakly interconnected._
- **Should `Config TS node` be split into smaller, more focused modules?**
  _Cohesion score 0.11764705882352941 - nodes in this community are weakly interconnected._
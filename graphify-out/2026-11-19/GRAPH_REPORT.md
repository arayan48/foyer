# Graph Report - foyer  (2026-11-19)

## Corpus Check
- 47 files · ~17,003 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 8 file(s) not represented in the graph (top: (none) 6, .nsh 1, .css 1)

## Summary
- 497 nodes · 758 edges · 35 communities (20 shown, 15 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 10 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ccaa6575`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- agent/src/main.rs
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
- ServerView.svelte
- agent.rs
- create
- App
- tls
- Status

## God Nodes (most connected - your core abstractions)
1. `lucide-svelte` - 16 edges
2. `compilerOptions` - 15 edges
3. `App` - 11 edges
4. `Server` - 11 edges
5. `call()` - 11 edges
6. `Status` - 10 edges
7. `S` - 10 edges
8. `create()` - 10 edges
9. `compilerOptions` - 10 edges
10. `Game` - 9 edges

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
- None detected.

## Hyperedges (group relationships)
- **Chaîne de release et mises à jour** — release, docs_architecture_minisign, docs_architecture_updater, docs_architecture_claude_box [EXTRACTED 1.00]
- **Sécurité agent** — docs_architecture_jeton_bearer, readme_reseau_local, roadmap_https_appairage, roadmap_compte_dedie [INFERRED 0.85]

## Communities (35 total, 15 thin omitted)

### Community 0 - "agent/src/main.rs"
Cohesion: 0.07
Nodes (7): auth(), console(), console_ws(), CRASH_WINDOW, eq_ct(), LOG_LINES, MAX_CRASHES

### Community 1 - "App.svelte"
Cohesion: 0.05
Nodes (53): act(), address, conn, connect(), current, loading, logout(), refresh() (+45 more)

### Community 2 - "Dépendances npm"
Cohesion: 0.07
Nodes (32): index.html (point d'entrée appli), @fontsource-variable/inter@^5.3.0, devDependencies, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.6, @tsconfig/svelte@^5.0.8 (+24 more)

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
Nodes (30): @tauri-apps/plugin-autostart@^2.7.0, @tauri-apps/plugin-process@^2.4.0, @tauri-apps/plugin-updater@^2.14.1, scripts, build, check, dev, preview (+22 more)

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

### Community 29 - "ServerView.svelte"
Cohesion: 0.05
Nodes (40): dependencies, lucide-svelte@^1.0.1, @tauri-apps/api@^2.12.5, Conn, patchServer(), Server, { dirty, busy, error, onsave, onreset }, cmd (+32 more)

### Community 30 - "agent.rs"
Cohesion: 0.07
Nodes (15): agent_call(), agent_probe(), client(), connect_ws(), console_close(), console_open(), console_send(), consoles() (+7 more)

### Community 31 - "create"
Cohesion: 0.23
Nodes (14): action(), create(), firewall(), list(), NewServer, Patch, remove(), S (+6 more)

### Community 32 - "App"
Cohesion: 0.24
Nodes (7): App, HostStats, kill_tree(), should_retry(), start(), start_inner(), stop()

### Community 33 - "tls"
Cohesion: 0.50
Nodes (3): fingerprint(), fingerprint_format(), tls()

### Community 34 - "Status"
Cohesion: 0.17
Nodes (10): Live, main(), stats_loop(), Status, Error, Installing, Running, Starting (+2 more)

## Knowledge Gaps
- **28 isolated node(s):** `foyer-agent`, `Minecraft`, `Fivem`, `Installing`, `Stopped` (+23 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 258 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `lucide-svelte` connect `ServerView.svelte` to `App.svelte`, `Dépendances npm`, `SettingsView.svelte`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **What connects `foyer-agent`, `Minecraft`, `Fivem` to the rest of the system?**
  _28 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `agent/src/main.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07058823529411765 - nodes in this community are weakly interconnected._
- **Why does `Status` connect `Status` to `agent/src/main.rs`, `create`?**
  _High betweenness centrality (0.008) - this node is a cross-community bridge._
- **Should `App.svelte` be split into smaller, more focused modules?**
  _Cohesion score 0.05480769230769231 - nodes in this community are weakly interconnected._
- **Should `Dépendances npm` be split into smaller, more focused modules?**
  _Cohesion score 0.06906906906906907 - nodes in this community are weakly interconnected._
- **Should `Config Tauri` be split into smaller, more focused modules?**
  _Cohesion score 0.06666666666666667 - nodes in this community are weakly interconnected._
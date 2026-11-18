# Graph Report - foyer  (2026-11-18)

## Corpus Check
- Corpus is ~10,247 words - fits in a single context window. You may not need a graph.

## Summary
- 348 nodes · 483 edges · 29 communities (13 shown, 16 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 8 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Agent HTTP et processus
- Interface Svelte
- Dépendances npm
- Config Tauri
- Recettes de jeux
- Config TS node
- Config TS app
- Concept agent et jeux
- Build, signature, MAJ
- API et appli
- Sécurité et accès
- Permissions Tauri
- Installeurs
- Faux agent de test
- Config TS racine
- Sauvegardes (plan)
- Serveur de test
- Design
- Paquet foyer
- Paquet agent
- README

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 15 edges
2. `Status` - 10 edges
3. `App` - 10 edges
4. `create()` - 10 edges
5. `compilerOptions` - 10 edges
6. `Game` - 9 edges
7. `S` - 9 edges
8. `lucide-svelte` - 9 edges
9. `call()` - 9 edges
10. `auth()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `Foyer (CLAUDE.md)` --references--> `foyer-agent (Rust, axum)`  [EXTRACTED]
  CLAUDE.md → docs/ARCHITECTURE.md
- `Agent Rust (port 7878)` --references--> `foyer-agent (Rust, axum)`  [EXTRACTED]
  README.md → docs/ARCHITECTURE.md
- `FOYER_DATA / FOYER_ADDR` --shares_data_with--> `Auth jeton Bearer`  [INFERRED]
  README.md → docs/ARCHITECTURE.md
- `index.html (point d'entrée appli)` --implements--> `Appli Tauri 2 + Svelte 5`  [INFERRED]
  app/index.html → docs/ARCHITECTURE.md
- `Jeux sous compte Windows dédié` --conceptually_related_to--> `Agent et jeux en SYSTEM`  [INFERRED]
  ROADMAP.md → docs/ARCHITECTURE.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Chaîne de release et mises à jour** — release, docs_architecture_minisign, docs_architecture_updater, docs_architecture_claude_box [EXTRACTED 1.00]
- **Sécurité agent** — docs_architecture_jeton_bearer, readme_reseau_local, roadmap_https_appairage, roadmap_compte_dedie [INFERRED 0.85]

## Communities (29 total, 16 thin omitted)

### Community 0 - "Agent HTTP et processus"
Cohesion: 0.05
Nodes (33): action(), App, auth(), console(), console_ws(), create(), eq_ct(), firewall() (+25 more)

### Community 1 - "Interface Svelte"
Cohesion: 0.06
Nodes (51): lucide-svelte@^1.0.1, act(), address, conn, connect(), current, loading, logout() (+43 more)

### Community 2 - "Dépendances npm"
Cohesion: 0.06
Nodes (39): dependencies, @fontsource-variable/inter@^5.3.0, @tauri-apps/api@^2.12.5, @tauri-apps/plugin-process@^2.4.0, @tauri-apps/plugin-updater@^2.14.1, devDependencies, svelte@^5.57.3, svelte-check@^4.7.7 (+31 more)

### Community 3 - "Config Tauri"
Cohesion: 0.07
Nodes (29): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+21 more)

### Community 4 - "Recettes de jeux"
Cohesion: 0.10
Nodes (12): client(), command(), download(), extract(), Game, Fivem, Minecraft, get_json() (+4 more)

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

### Community 9 - "API et appli"
Cohesion: 0.29
Nodes (7): index.html (point d'entrée appli), Test UI via mock-agent, API REST /api + WebSocket console, Appli Tauri 2 + Svelte 5, Auth jeton Bearer, Application Tauri, FOYER_DATA / FOYER_ADDR

### Community 10 - "Sécurité et accès"
Cohesion: 0.29
Nodes (6): Agent et jeux en SYSTEM, v0.6 Sécurité, v0.7 Accès extérieur, Jeux sous compte Windows dédié, HTTPS épinglé + appairage 6 chiffres, Intégration playit.gg

### Community 11 - "Permissions Tauri"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "Installeurs"
Cohesion: 0.40
Nodes (4): POST /api/shutdown, Installeur Foyer-Client.exe, Installeur Foyer-Serveur.exe, installer.nsh hooks NSIS

## Knowledge Gaps
- **28 isolated node(s):** `foyer-agent`, `Minecraft`, `Fivem`, `Installing`, `Stopped` (+23 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 187 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `lucide-svelte` connect `Interface Svelte` to `Dépendances npm`?**
  _High betweenness centrality (0.032) - this node is a cross-community bridge._
- **What connects `foyer-agent`, `Minecraft`, `Fivem` to the rest of the system?**
  _28 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Agent HTTP et processus` be split into smaller, more focused modules?**
  _Cohesion score 0.053923541247484906 - nodes in this community are weakly interconnected._
- **Why does `index.html (point d'entrée appli)` connect `API et appli` to `Dépendances npm`?**
  _High betweenness centrality (0.011) - this node is a cross-community bridge._
- **Should `Interface Svelte` be split into smaller, more focused modules?**
  _Cohesion score 0.06001984126984127 - nodes in this community are weakly interconnected._
- **Should `Dépendances npm` be split into smaller, more focused modules?**
  _Cohesion score 0.05893719806763285 - nodes in this community are weakly interconnected._
- **Should `Config Tauri` be split into smaller, more focused modules?**
  _Cohesion score 0.06666666666666667 - nodes in this community are weakly interconnected._
<p align="center">
  <img src="app/src-tauri/icons/128x128.png" width="96" alt="Foyer">
</p>

<h1 align="center">Foyer</h1>

<p align="center">
  <strong>Self-hosted game server manager for Windows and Linux.</strong><br>
  Install, run and monitor your game servers from a clean desktop app.
</p>

<p align="center">
  <a href="https://github.com/arayan48/foyer/releases/latest"><img src="https://img.shields.io/github/v/release/arayan48/foyer?style=flat-square" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/arayan48/foyer?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux-blue?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri-orange?style=flat-square" alt="Rust + Tauri">
</p>

---

> 🇫🇷 **En bref :** Foyer installe, lance et surveille vos serveurs de jeu (Minecraft, FiveM, d'autres jeux à venir) depuis une appli de bureau. Un agent tourne sur la machine serveur, l'appli s'y connecte en HTTPS chiffré. Téléchargez `Foyer-Serveur.exe` sur la machine qui héberge les jeux, `Foyer-Client.exe` sur votre PC. Le reste de ce document est en anglais.

---

## Overview

Foyer turns any Windows or Linux machine into a game server host, without config files or command lines. A lightweight Rust **agent** does the work on the host. A **desktop app** (Tauri + Svelte) gives you a dashboard, live consoles and settings for every server.

### Supported games

| Game | Status | Source |
|---|---|---|
| Minecraft (Paper) | ✅ Supported | PaperMC, latest stable + Java 25 (Adoptium) |
| FiveM (txAdmin) | ✅ Supported | Official "recommended" build |
| More games | 🗺️ Planned | Foyer uses a game recipe system built for new titles, see [ROADMAP.md](ROADMAP.md) |

## Features

- **One-click install**: downloads the game server and its runtime, ready to launch.
- **Full lifecycle**: start, stop, restart with graceful shutdown and process-tree cleanup.
- **Live console**: real-time logs and command input over WebSocket.
- **Monitoring**: per-server CPU and memory, host CPU, RAM and disk.
- **Resilience**: auto-start at boot, auto-restart on crash (3 attempts per 10 min).
- **Resource limits**: RAM and CPU caps per server.
- **Firewall handled**: ports opened and closed automatically.
- **Secure by default**: HTTPS with a pinned self-signed certificate, bearer token on every route.
- **Remote or local**: manage the host from another PC, or directly on the server itself.

## Architecture

```
  Your PC                               Host machine
 ┌──────────────────┐   HTTPS + WSS    ┌───────────────────────────────────┐
 │  Foyer app       │ ───────────────▶ │  foyer-agent (port 7878)          │
 │  Tauri + Svelte  │   Bearer token   │   ├─ REST API  /api/...           │
 └──────────────────┘   pinned cert    │   ├─ WebSocket console            │
                                       │   ├─ system & process stats       │
                                       │   └─ game servers (child procs)   │
                                       └───────────────────────────────────┘
```

| Component | Path | Stack |
|---|---|---|
| Agent | [`agent/`](agent) | Rust, axum, rustls |
| Desktop app | [`app/`](app) | Tauri 2, Svelte 5, TypeScript |

Full details: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Installation

Download from the [latest release](https://github.com/arayan48/foyer/releases/latest):

| Installer | Install on | Contents |
|---|---|---|
| `Foyer-Serveur.exe` | The machine hosting the games | App + agent, boot service, firewall rule |
| `Foyer-Client.exe` | Any PC you manage from | App only, no admin rights |

**First connection:** enter the host address and the token from `C:\Foyer\token`. Compare the fingerprint shown by the app with `C:\Foyer\fingerprint` on the host before accepting.

## Security

- All traffic is encrypted (TLS), the app pins the agent certificate.
- Every route, WebSocket included, requires the token in the `Authorization` header.
- **Never expose port 7878 to the internet.** For remote play or management, use a VPN or a tunnel such as playit.gg.

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| `FOYER_DATA` | `C:\Foyer` / `/var/lib/foyer` | Data directory (token, certificate, servers) |
| `FOYER_ADDR` | `0.0.0.0:7878` | Agent listen address |

## Building from source

**Agent**

```bash
cd agent
cargo build --release                                  # Linux
cargo build --release --target x86_64-pc-windows-gnu   # Windows (cross, mingw-w64)
```

**Desktop app**

```bash
cd app
npm install
npx tauri build                                                     # current platform
npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc # Windows from Linux
```

**UI development without a server**

```bash
cd app
node mock-agent.mjs   # fake agent, token: test
npx vite
```

## Roadmap

Next up: polished visuals and installer, file manager and backups, player management, mods and plugins, then more games. See [docs/PLAN.md](docs/PLAN.md) and [ROADMAP.md](ROADMAP.md).

## License

[MIT](LICENSE)

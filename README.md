# Foyer

Gestion de serveurs de jeu (Minecraft, FiveM) chez soi, sous Windows ou Linux.

- `agent/` : service Rust installé sur la machine qui héberge les jeux. Il télécharge, lance, arrête et surveille les serveurs, et expose une API HTTP + WebSocket protégée par un jeton (port 7878).
- `app/` : application de bureau Tauri (Rust + Svelte) qui se connecte à l'agent.

## Agent

```
cargo build --release                                   # Linux
cargo build --release --target x86_64-pc-windows-gnu    # Windows (depuis Linux, avec mingw-w64)
```

Variables : `FOYER_DATA` (défaut `C:\Foyer` ou `/var/lib/foyer`), `FOYER_ADDR` (défaut `0.0.0.0:7878`).
Le jeton est créé au premier lancement dans `<FOYER_DATA>/token`.

À garder sur le réseau local : le trafic n'est pas chiffré (HTTP). Pour jouer depuis l'extérieur, passer par un VPN ou playit.gg, jamais en ouvrant le port 7878 sur la box.

## Application

```
cd app
npm install
npm run dev                  # interface seule dans le navigateur
node mock-agent.mjs          # faux agent pour tester l'interface (jeton : test)
npx tauri build              # binaire de la plateforme courante
npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc   # .exe Windows depuis Linux
```

## Licence

MIT

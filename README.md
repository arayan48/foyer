# Foyer

Gestion de serveurs de jeu (Minecraft, FiveM) chez soi, sous Windows ou Linux.

- `agent/` : service Rust installé sur la machine qui héberge les jeux. Il télécharge, lance, arrête et surveille les serveurs, et expose une API HTTPS + WebSocket protégée par un jeton (port 7878). Certificat auto-signé créé au premier lancement ; l'appli épingle son empreinte (fichier `fingerprint`).
- `app/` : application de bureau Tauri (Rust + Svelte) qui se connecte à l'agent.

## Agent

```
cargo build --release                                   # Linux
cargo build --release --target x86_64-pc-windows-gnu    # Windows (depuis Linux, avec mingw-w64)
```

Variables : `FOYER_DATA` (défaut `C:\Foyer` ou `/var/lib/foyer`), `FOYER_ADDR` (défaut `0.0.0.0:7878`).
Le jeton est créé au premier lancement dans `<FOYER_DATA>/token`.

Le trafic est chiffré (HTTPS, certificat épinglé). À la première connexion, comparer l'empreinte affichée par l'appli avec le fichier `C:\Foyer\fingerprint` du serveur. Pour jouer depuis l'extérieur, passer par un VPN ou playit.gg, jamais en ouvrant le port 7878 sur la box.

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

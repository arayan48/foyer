# Foyer

Gestionnaire de serveurs de jeu (Minecraft, FiveM) : agent Rust + appli Tauri/Svelte. Dépôt public `arayan48/foyer`, licence MIT.

## À lire d'abord
- `docs/ARCHITECTURE.md` : comment c'est fait (API, fichiers, build, mises à jour)
- `docs/PLAN.md` : versions et prochaine étape
- `ROADMAP.md` : liste complète des idées, à cocher
- `docs/DECISIONS.md` : choix déjà faits, ne pas les rediscuter sans raison
- `graphify-out/GRAPH_REPORT.md` : carte du code (puis `graphify query "…"`)

## Règles
- Textes de l'appli, docs et commits en français.
- Aucune trace d'IA (commits, code, docs, release).
- Pas de Rust sur le Mac : compiler sur claude-box (`ssh -i ~/.ssh/claude_box -o IdentitiesOnly=yes rayan@192.168.1.244`, dossier `/srv/foyer`).
- Publier : `./release.sh <version> "<notes>"` ; les exe finissent aussi dans `~/Downloads` avec des noms clairs.
- Interface : charte graphique 1.0 (`../foyer-site/charte/`), jetons de `app/src/app.css`, accent terracotta `--primary`, Figtree pour l'interface et Unbounded pour les titres, états hover/focus/disabled partout.
- Tester l'interface sans serveur : `node app/mock-agent.mjs` (jeton `test`) + `npx vite` dans `app/`.
- Serveur de test réel : la .240 (`ssh -i ~/.ssh/arayan_claude -o IdentitiesOnly=yes Administrateur@192.168.1.240`) ; demander avant d'y installer ou changer quoi que ce soit.
- Après modifs, mettre à jour le graphe : `/graphify . --update`.

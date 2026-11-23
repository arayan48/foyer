# Décisions

| Date | Décision | Pourquoi |
|---|---|---|
| 2026-11-18 | Faire son propre panneau plutôt qu'AMP / HTN Panel | HTN Panel payant (8,99 $/mois) ; projet perso open source |
| 2026-11-18 | Appli de bureau (.exe), pas une appli web | Demande de Rayan |
| 2026-11-18 | Rust (agent) + Tauri 2 / Svelte 5 (appli) | Open source, Windows + Linux, binaires légers sans dépendance ; Rust préféré à C# pour le multi-plateforme |
| 2026-11-18 | Pas de conteneurs | Conteneurs Windows lourds, FiveM s'y prête mal ; isolation par dossier + limites Windows (Job Objects) |
| 2026-11-18 | Agent séparé de l'interface (télécommande / télé) | Gérer à distance ; serveur possible sans fenêtre |
| 2026-11-18 | Deux installeurs : Foyer Serveur / Foyer Client | Plus clair qu'un installeur unique avec questions |
| 2026-11-18 | Mises à jour via GitHub Releases, dépôt public `arayan48/foyer` | Gratuit, marche partout, open source |
| 2026-11-18 | Build sur claude-box (cross-compilation) | Pas de Rust sur le Mac ; claude-box est la machine de travail |
| 2026-11-18 | Licence MIT | Simple, permissive |
| 2026-11-18 | Nom « Foyer », accent rose braise | Héberger chez soi ; identité propre, pas copiée de HTN |
| 2026-11-19 | Sécurité sans service externe : certificat auto-signé épinglé, appairage local | Un service tiers (domaine, Let's Encrypt, relais) peut fermer ; l'agent doit rester sûr tout seul |
| 2026-11-23 | Tunnel playit.gg intégré à l'agent, pas de service `playit.exe` séparé | Un seul exe et une seule mise à jour ; Foyer crée et supprime lui-même les tunnels ; reste optionnel |
| 2026-11-19 | Peu de dépendances, seulement des bibliothèques connues et maintenues | Une dépendance ou un service peut mourir à tout moment ; tout ce qui est externe reste optionnel |

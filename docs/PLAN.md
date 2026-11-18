# Plan de Foyer par versions

La liste complète des idées est dans [../ROADMAP.md](../ROADMAP.md). Ici : l'ordre de travail, version par version.
Chaque version = une release via `./release.sh`, testée sur la .240 avant de passer à la suivante.

## Principes
1. Une version = un thème, livrable en quelques sessions.
2. Ne jamais casser une installation existante : les mises à jour doivent passer sans réinstaller.
3. Chaque nouvelle fonction de l'agent a son test.
4. L'interface reste cohérente : mêmes boutons, mêmes champs, même accent.

---

## v0.2 — Paramètres et limites  ← prochaine
Objectif : régler un serveur sans toucher aux fichiers.
- Page **Paramètres d'un serveur** (nom, port, RAM, **CPU %**, cœurs, démarrage auto, redémarrage si crash)
- Agent : limites dures CPU + RAM (Windows Job Objects ; Linux cgroups v2), affinité
- Agent : redémarrage auto si crash (3 essais / 10 min), démarrage auto au boot
- Éditeur `server.properties` en formulaire (Minecraft)
- Page **Paramètres de l'appli** (version, vérifier les mises à jour, rafraîchissement)
- API : `PATCH /api/servers/{id}`, `GET/PUT /api/servers/{id}/properties`
- Tests : limites appliquées, PATCH, redémarrage auto

## v0.3 — Look et installateur
Objectif : une appli et un installeur qui donnent envie.
- Visuels des jeux (bannières, icônes) après vérification des droits ; sinon illustrations Foyer
- Transitions de pages, apparition des listes, états de chargement soignés
- Installeur NSIS : bannière + image latérale au logo Foyer
- Icône dans la zone de notification (état des serveurs, ouvrir, quitter)
- Notifications Windows (crash, mise à jour dispo)

## v0.4 — Fichiers et sauvegardes
- Gestionnaire de fichiers (parcourir, éditer texte, envoyer, télécharger, supprimer)
- Sauvegardes manuelles + planifiées, restauration, rétention
- Suppression de serveur → archive au lieu d'effacement
- Sauvegarde auto avant mise à jour du jeu

## v0.5 — Joueurs et jeux
- Joueurs connectés, kick / ban / op / whitelist
- Mods / plugins Minecraft (Modrinth), ressources FiveM
- Choix de version et de type Minecraft (Paper, Vanilla, Fabric, Forge…)
- Recettes de jeux en fichiers TOML

## v0.6 — Sécurité
- HTTPS avec certificat épinglé + appairage par code à 6 chiffres
- Jeux sous un compte Windows dédié (plus SYSTEM)
- Comptes et rôles pour les amis, journal des actions

## v0.7 — Accès extérieur et multi-machines
- Intégration playit.gg (les potes jouent depuis chez eux sans ouvrir la box)
- Test de joignabilité des ports
- Plusieurs agents dans un même client, découverte sur le réseau

## v0.8 — Automatisation
- Tâches planifiées (redémarrage quotidien, commandes, annonces)
- Webhooks Discord
- Graphiques historiques CPU / RAM / joueurs

## v0.9 — Industrialisation
- CI GitHub Actions : tests + build + release (plus besoin de claude-box)
- Installateur au style de l'appli (bootstrapper Tauri)
- Paquets Linux, doc utilisateur, traduction anglaise

## v1.0
- Tout ce qui précède stable, testé sur Windows et Linux, doc complète.

---

## Risques à surveiller
- **Clé de signature** seulement sur claude-box : la sauvegarder (gestionnaire de mots de passe) avant v0.2.
- **SmartScreen** : alerte tant que l'installeur n'est pas signé (certificat payant).
- **Marques des jeux** : vérifier les droits avant d'intégrer des logos officiels.
- **Mises à jour du jeu** (Paper, FiveM) qui changent d'API ou d'arguments : déjà vu avec `--nogui` → `nogui`.

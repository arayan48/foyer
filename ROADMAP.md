# Foyer — feuille de route

_Créée le 2026-11-18, après la v0.1.0 (testée et fonctionnelle sur la .240)._
Cocher au fur et à mesure. Priorités : **P1** = prochaine version, **P2** = ensuite, **P3** = plus tard.

## 1. Ressources et performances
- [ ] **P1** Limite CPU par serveur (Windows : Job Objects `JOBOBJECT_CPU_RATE_CONTROL` ; Linux : cgroups v2 `cpu.max`)
- [ ] **P1** Limite RAM « dure » par serveur (Job Objects / `memory.max`), pas seulement `-Xmx` pour Minecraft
- [ ] **P1** Choix des cœurs (affinité CPU) par serveur
- [ ] **P2** Priorité processus (basse / normale / haute)
- [ ] **P2** Graphiques CPU / RAM / joueurs sur 1 h, 24 h, 7 j (historique stocké par l'agent)
- [ ] **P2** Taille du dossier de chaque serveur (agent fait), alerte disque presque plein
- [ ] **P3** Alerte si un serveur dépasse sa limite longtemps

## 2. Paramètres et options
- [x] **P1** Page **Paramètres de l'appli** (v0.1.2 : arrière-plan, démarrage auto, mises à jour, connexion ; reste : thème, langue, rafraîchissement) : thème, langue, fréquence de rafraîchissement, vérifier les mises à jour, version installée
- [x] **P1** Page **Paramètres d'un serveur** (v0.2 : tout sauf CPU) : nom, port, RAM, CPU, démarrage auto au boot, redémarrage auto si crash
- [x] **P1** Éditeur `server.properties` (Minecraft) en formulaire : difficulté, joueurs max, mode de jeu, whitelist, seed, MOTD
- [ ] **P1** Éditeur `server.cfg` (FiveM) : nom, slots, clé de licence (masquée)
- [ ] **P2** Choix de la version du jeu (Minecraft 1.20, 1.21, 26.x… ; build FiveM recommandé / dernier)
- [ ] **P2** Choix du type Minecraft : Paper, Vanilla, Fabric, Forge, NeoForge, Purpur
- [ ] **P2** Variables / arguments Java avancés (flags Aikar)
- [ ] **P2** Paramètres de l'agent depuis l'appli : port, dossier de données, régénérer le jeton
- [ ] **P3** Profils de réglages réutilisables (« Survie 10 joueurs », « RP 32 slots »)
- [ ] **P2** Fréquence de vérification des mises à jour réglable (aujourd'hui fixe : au lancement puis toutes les 6 h), avec bouton « Vérifier maintenant »

## 3. Interface et style
- [ ] **P1** Vrais visuels des jeux dans la liste et la création (bannière + icône par jeu) — voir note droits ci-dessous
- [ ] **P1** Supprimer un serveur depuis la liste (menu « … » sur chaque ligne), pas seulement en bas de sa page
- [ ] **P1** Écran d'accueil de chaque serveur plus riche : joueurs connectés, uptime, version
- [ ] **P1** Animations plus soignées (transitions de pages, apparition des cartes, états de chargement)
- [ ] **P2** Thème clair + choix de la couleur d'accent
- [ ] **P2** Palette de commandes (Ctrl+K) : démarrer, aller à un serveur, ouvrir la console
- [ ] **P2** Notifications Windows : serveur planté, mise à jour dispo, sauvegarde finie
- [x] **P2** Icône dans la zone de notification (près de l'horloge) avec état des serveurs
- [ ] **P3** Mode compact / mode « grand écran »
- [ ] **P3** Version Mac (.dmg) — à compiler sur un Mac

## 4. Installateur
- [ ] **P1** Logo Foyer partout dans l'installateur : bannière en haut, image latérale, icône (NSIS `headerImage`, `sidebarImage`)
- [ ] **P2** Installateur au style de l'appli : petit programme Tauri « bootstrapper » (même design sombre, choix serveur / interface en cartes, barre de progression), qui lance l'installation en silence
- [ ] **P2** Choix du dossier de données (autre disque que C:)
- [ ] **P2** Signature de code (supprimer l'alerte SmartScreen) — certificat payant, à décider
- [ ] **P3** Paquets Linux `.deb` / `.AppImage` dans chaque release

## 5. Fichiers et sauvegardes
- [ ] **P1** Gestionnaire de fichiers : parcourir, éditer, envoyer, télécharger, supprimer (agent fait, reste l'interface)
- [ ] **P1** Sauvegardes : manuelle + planifiée, restauration en un clic, rétention (garder les N dernières)
- [ ] **P2** Sauvegarde avant chaque mise à jour du jeu
- [ ] **P2** Corbeille : la suppression d'un serveur passe par une archive au lieu d'effacer direct (agent fait, 30 jours ; reste l'interface)
- [ ] **P3** Envoi des sauvegardes sur le HDD 300 Go, un NAS ou le cloud

## 6. Jeux
- [ ] **P1** Gestion des mods / plugins Minecraft (Modrinth, Hangar)
- [ ] **P1** Ressources FiveM (dossier `resources`, activer / désactiver)
- [ ] **P2** Recettes de jeux en fichiers (TOML) pour en ajouter sans recompiler
- [ ] **P2** Nouveaux jeux : RedM, Rust, Valheim, Palworld, Terraria, ARK, 7 Days to Die, Satisfactory (via SteamCMD)
- [ ] **P3** Catalogue communautaire de recettes

## 7. Réseau et accès
- [ ] **P1** Liste des joueurs connectés, kick / ban / op depuis l'appli (agent fait pour Minecraft, reste l'interface)
- [ ] **P2** Accès depuis l'extérieur pour les joueurs : intégration playit.gg (tunnel sans ouvrir la box)
- [ ] **P2** Vérification des ports (« ton serveur est-il joignable ? »)
- [ ] **P2** Plusieurs machines dans le même client (liste de serveurs Foyer)
- [ ] **P3** Découverte automatique des agents sur le réseau local

## 8. Sécurité
- [x] **P1** HTTPS entre client et agent (certificat auto-signé épinglé au premier appairage)
- [ ] **P1** Appairage par code à 6 chiffres au lieu de copier le jeton
- [x] **P1** Jeton de la console hors de l'URL (sous-protocole WebSocket ou ticket court)
- [ ] **P1** Jeton par appareil, révocable un par un
- [x] **P2** Blocage temporaire après plusieurs mauvais jetons
- [x] **P2** Corbeille : serveur supprimé archivé quelques jours avant effacement
- [ ] **P2** `SECURITY.md` (signaler une faille en privé) + `cargo audit` / `npm audit` dans GitHub Actions
- [ ] **P3** Doc : accès à distance via VPN (Tailscale, WireGuard) plutôt que port ouvert sur la box
- [ ] **P2** Comptes utilisateurs et rôles (admin, modérateur, lecture seule) pour les amis
- [ ] **P2** Journal des actions (agent fait : adresse IP, action, date ; reste l'interface)
- [ ] **P2** Faire tourner les jeux sous un compte Windows dédié, pas SYSTEM
- [ ] **P3** Sauvegarde de la clé de signature des mises à jour (aujourd'hui seulement sur claude-box)

## 9. Planification et automatisation
- [x] **P1** Redémarrage auto si crash (avec limite de tentatives)
- [ ] **P2** Tâches planifiées : redémarrage quotidien, sauvegarde, commande, message aux joueurs
- [ ] **P2** Démarrage auto des serveurs choisis au boot de la machine
- [ ] **P3** Webhooks Discord (serveur en ligne, crash, joueur connecté)

## 10. Qualité et projet
- [ ] **P1** Tests de l'agent (install, start, stop, limites) dans un CI GitHub Actions
- [ ] **P1** Build et release automatiques par GitHub Actions (plus besoin de claude-box)
- [x] **P2** Journal de l'agent dans un fichier (`C:\Foyer\agent.log`), rotation
- [ ] **P2** Page de documentation (installation, FAQ, dépannage)
- [ ] **P3** Traduction anglaise (projet open source)

## Notes
- **Logos des jeux** : Minecraft (Mojang/Microsoft), FiveM (Cfx.re/Rockstar) sont des marques. Dans un dépôt public, préférer des visuels propres à Foyer ou des images officielles « press kit » si leurs conditions l'autorisent. À vérifier jeu par jeu avant d'intégrer.
- **Publier une version** : `./release.sh <version> "<notes>"` (copie aussi les exe dans Téléchargements).

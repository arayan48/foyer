<script lang="ts">
  import { Earth, Swords, Users } from 'lucide-svelte'
</script>

<div class="card">
  <div class="card-head"><Earth size={18} /><div><h2>Monde <span class="soon">Bientôt</span></h2><p>Réglages de server.properties, sans ouvrir le fichier.</p></div></div>
  <label class="row"><div><strong>Message d'accueil</strong><small>motd, affiché dans la liste des serveurs du jeu.</small></div><input class="input" placeholder="Un serveur Minecraft" disabled /></label>
  <label class="row"><div><strong>Graine du monde</strong><small>level-seed, vide = au hasard. Pour un nouveau monde seulement.</small></div><input class="input" disabled /></label>
  <label class="row"><div><strong>Distance d'affichage</strong><small>view-distance, en tronçons. Plus haut = plus lourd.</small></div><input class="input" type="number" value="10" disabled /></label>
</div>

<div class="card">
  <div class="card-head"><Swords size={18} /><div><h2>Partie <span class="soon">Bientôt</span></h2><p>Comment on joue.</p></div></div>
  <label class="row"><div><strong>Mode de jeu</strong></div><select class="input" disabled><option>Survie</option><option>Créatif</option><option>Aventure</option><option>Spectateur</option></select></label>
  <label class="row"><div><strong>Difficulté</strong></div><select class="input" disabled><option>Normale</option><option>Paisible</option><option>Facile</option><option>Difficile</option></select></label>
  <label class="row"><div><strong>Combat entre joueurs</strong><small>pvp</small></div><input type="checkbox" class="switch" checked disabled /></label>
  <label class="row"><div><strong>Mode hardcore</strong><small>Une seule vie.</small></div><input type="checkbox" class="switch" disabled /></label>
</div>

<div class="card">
  <div class="card-head"><Users size={18} /><div><h2>Accès <span class="soon">Bientôt</span></h2><p>Qui peut entrer.</p></div></div>
  <label class="row"><div><strong>Joueurs maximum</strong><small>max-players</small></div><input class="input" type="number" value="20" disabled /></label>
  <label class="row"><div><strong>Liste blanche</strong><small>Seuls les joueurs ajoutés peuvent entrer.</small></div><input type="checkbox" class="switch" disabled /></label>
  <label class="row"><div><strong>Comptes officiels seulement</strong><small>online-mode, à laisser activé.</small></div><input type="checkbox" class="switch" checked disabled /></label>
</div>

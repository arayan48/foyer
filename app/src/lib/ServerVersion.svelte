<script lang="ts">
  import { Package, RefreshCw, History } from 'lucide-svelte'
  import type { Server } from './api'

  let { server }: { server: Server } = $props()
  const mc = server.game === 'minecraft'
</script>

{#if mc}
  <div class="card">
    <div class="card-head"><Package size={18} /><div><h2>Logiciel serveur</h2><p>Le programme qui fait tourner le monde.</p></div></div>
    <div class="row">
      <div><strong>Installé</strong><small>Paper, dernière version stable au moment de l'installation.</small></div>
      <div class="val">{server.version ?? 'Paper'}</div>
    </div>
    <label class="row">
      <div><strong>Type <span class="soon">Bientôt</span></strong><small>Paper pour les plugins, Fabric ou Forge pour les mods.</small></div>
      <select class="input" disabled><option>Paper</option><option>Vanilla</option><option>Fabric</option><option>Forge</option></select>
    </label>
    <label class="row">
      <div><strong>Version de Minecraft <span class="soon">Bientôt</span></strong><small>Les joueurs doivent avoir la même.</small></div>
      <select class="input" disabled><option>Dernière stable</option></select>
    </label>
  </div>
{:else}
  <div class="card">
    <div class="card-head"><Package size={18} /><div><h2>Artefacts FiveM</h2><p>Les fichiers du serveur FXServer, publiés par Cfx.re.</p></div></div>
    <div class="row">
      <div><strong>Build installé</strong><small>Build « recommended » au moment de l'installation.</small></div>
      <div class="val">{server.version ?? 'Recommended'}</div>
    </div>
    <label class="row">
      <div><strong>Canal <span class="soon">Bientôt</span></strong><small>« Recommended » est le plus sûr ; « Latest » a les nouveautés.</small></div>
      <select class="input" disabled><option>Recommended</option><option>Optional</option><option>Latest</option><option>Build précis…</option></select>
    </label>
  </div>
{/if}

<div class="card">
  <div class="card-head"><RefreshCw size={18} /><div><h2>Mises à jour <span class="soon">Bientôt</span></h2><p>Foyer vérifie s'il existe une version plus récente.</p></div></div>
  <div class="row">
    <div><strong>Mettre à jour maintenant</strong><small>Arrête le serveur, met à jour, puis le relance.</small></div>
    <div class="val"><button class="btn" disabled><RefreshCw size={14} /> Mettre à jour</button></div>
  </div>
  <label class="row">
    <div><strong>Sauvegarder avant chaque mise à jour</strong><small>Pour revenir en arrière si ça casse.</small></div>
    <input type="checkbox" class="switch" checked disabled />
  </label>
</div>

<div class="card">
  <div class="card-head"><History size={18} /><div><h2>Revenir à une version <span class="soon">Bientôt</span></h2><p>Réinstalle une version précédente à partir d'une sauvegarde.</p></div></div>
</div>

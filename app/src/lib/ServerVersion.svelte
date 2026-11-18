<script lang="ts">
  import { Package, RefreshCw, History } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import type { Server } from './api'

  let { server }: { server: Server } = $props()
  const mc = server.game === 'minecraft'
  const pages = [
    { id: 'software', label: mc ? 'Logiciel' : 'Artefacts', icon: Package },
    { id: 'updates', label: 'Mises à jour', icon: RefreshCw },
    { id: 'rollback', label: 'Retour arrière', icon: History },
  ]
  let page = $state('software')
</script>

<SubPages {pages} bind:page label="Réglages de version">
  {#if page === 'software'}
    {#if mc}
      <h2>Logiciel serveur</h2>
      <p class="lead">Le programme qui fait tourner le monde.</p>
      <div class="card">
        <div class="row">
          <div><strong>Installé</strong><small>Paper, dernière version stable au moment de l'installation.</small></div>
          <div class="val">{server.version ?? 'Paper'}</div>
        </div>
        <div class="row">
          <div><strong>Java</strong><small>Choisi à la création selon la version du jeu.</small></div>
          <div class="val">Java {server.java ?? 25}</div>
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
      <h2>Artefacts FiveM</h2>
      <p class="lead">Les fichiers du serveur FXServer, publiés par Cfx.re.</p>
      <div class="card">
        <div class="row">
          <div><strong>Build installé</strong><small>Build « recommended » au moment de l'installation.</small></div>
          <div class="val">{server.version ?? 'Recommended'}</div>
        </div>
        <div class="row">
          <div><strong>Mode</strong><small>{server.txadmin === false ? 'FXServer lancé directement avec server-data/server.cfg.' : 'Géré par txAdmin.'}</small></div>
          <div class="val">{server.txadmin === false ? ({ esx: 'ESX Legacy', qbcore: 'QBCore' } as Record<string, string>)[server.framework ?? ''] ?? 'Vanilla' : 'txAdmin'}</div>
        </div>
        <label class="row">
          <div><strong>Canal <span class="soon">Bientôt</span></strong><small>« Recommended » est le plus sûr ; « Latest » a les nouveautés.</small></div>
          <select class="input" disabled><option>Recommended</option><option>Optional</option><option>Latest</option><option>Build précis…</option></select>
        </label>
      </div>
    {/if}

  {:else if page === 'updates'}
    <h2>Mises à jour <span class="soon">Bientôt</span></h2>
    <p class="lead">Foyer vérifie s'il existe une version plus récente.</p>
    <div class="card">
      <div class="row">
        <div><strong>Mettre à jour maintenant</strong><small>Arrête le serveur, met à jour, puis le relance.</small></div>
        <div class="val"><button class="btn" disabled><RefreshCw size={14} /> Mettre à jour</button></div>
      </div>
      <label class="row">
        <div><strong>Sauvegarder avant chaque mise à jour</strong><small>Pour revenir en arrière si ça casse.</small></div>
        <input type="checkbox" class="switch" checked disabled />
      </label>
    </div>

  {:else if page === 'rollback'}
    <h2>Revenir à une version <span class="soon">Bientôt</span></h2>
    <p class="lead">Réinstalle une version précédente à partir d'une sauvegarde.</p>
  {/if}
</SubPages>

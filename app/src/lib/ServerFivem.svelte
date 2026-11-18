<script lang="ts">
  import { LayoutDashboard, KeyRound, FileCog, Eye, EyeOff, ExternalLink } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import type { Server } from './api'

  let { server, address }: { server: Server; address: string } = $props()
  let txadmin = $state(server.txadmin ?? true)
  let showKey = $state(false)
  const pages = [
    { id: 'txadmin', label: 'txAdmin', icon: LayoutDashboard },
    { id: 'license', label: 'Licence', icon: KeyRound },
    { id: 'cfg', label: 'server.cfg', icon: FileCog },
  ]
  let page = $state('txadmin')
</script>

<SubPages {pages} bind:page label="Réglages FiveM">
  {#if page === 'txadmin'}
    <h2>txAdmin</h2>
    <p class="lead">Le panneau web de FiveM : ressources, joueurs, bans, redémarrages.</p>
    <div class="card">
      <label class="row">
        <div><strong>Utiliser txAdmin <span class="soon">Bientôt réglable</span></strong><small>Sans txAdmin, Foyer lance FXServer directement avec le fichier server.cfg ci-dessous.</small></div>
        <input type="checkbox" class="switch" bind:checked={txadmin} disabled />
      </label>
      {#if server.admin_port}
        <div class="row">
          <div><strong>Ouvrir txAdmin</strong><small>Port {server.admin_port}. Le code PIN du premier accès s'affiche dans la console.</small></div>
          <div class="val"><a class="btn" href="http://{address}:{server.admin_port}" target="_blank" rel="noreferrer"><ExternalLink size={14} /> Ouvrir</a></div>
        </div>
      {/if}
    </div>

  {:else if page === 'license'}
    <h2>Clé de licence Cfx.re <span class="soon">Bientôt</span></h2>
    <p class="lead">Obligatoire pour que le serveur démarre. Gratuite sur portal.cfx.re.</p>
    <div class="card">
      <label class="row">
        <div><strong>Clé (sv_licenseKey)</strong><small>Reste sur ta machine, n'est jamais affichée aux joueurs.</small></div>
        <div class="key">
          <input class="input" type={showKey ? 'text' : 'password'} placeholder="cfxk_…" autocomplete="off" spellcheck="false" disabled />
          <button class="btn icon" onclick={() => (showKey = !showKey)} aria-label={showKey ? 'Masquer la clé' : 'Afficher la clé'} disabled>{#if showKey}<EyeOff size={14} />{:else}<Eye size={14} />{/if}</button>
        </div>
      </label>
      <div class="row">
        <div><strong>Obtenir une clé</strong><small>Connexion avec ton compte Cfx.re, puis « Server Keys ».</small></div>
        <div class="val"><a class="btn" href="https://portal.cfx.re/servers/registration-keys" target="_blank" rel="noreferrer"><ExternalLink size={14} /> portal.cfx.re</a></div>
      </div>
    </div>

  {:else if page === 'cfg'}
    <h2>server.cfg <span class="soon">Bientôt</span></h2>
    <p class="lead">{txadmin ? 'Avec txAdmin, ces réglages se font dans txAdmin.' : 'Réglages principaux du serveur.'}</p>
    <div class="card">
      <label class="row"><div><strong>Nom affiché</strong><small>sv_hostname, visible dans la liste des serveurs.</small></div><input class="input" placeholder={server.name} disabled /></label>
      <label class="row"><div><strong>Joueurs maximum</strong><small>sv_maxclients, 48 sans abonnement.</small></div><input class="input" type="number" value="48" disabled /></label>
      <label class="row"><div><strong>OneSync</strong><small>Requis au-delà de 32 joueurs.</small></div><select class="input" disabled><option>Activé</option><option>Désactivé</option></select></label>
      <label class="row"><div><strong>Serveur privé</strong><small>Caché de la liste publique FiveM.</small></div><input type="checkbox" class="switch" checked disabled /></label>
    </div>
  {/if}
</SubPages>

<style>
  .key { display: flex; gap: 6px; }
  .key .input { flex: 1; min-width: 0; }
  a.btn { text-decoration: none; }
</style>

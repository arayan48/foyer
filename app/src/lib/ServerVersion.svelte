<script lang="ts">
  import { Package, RefreshCw, History } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import { call, patchServer, type Catalog, type Conn, type Server } from './api'

  let { conn, server }: { conn: Conn; server: Server } = $props()
  let javas = $state<number[]>([8, 11, 17, 21, 22, 23, 24, 25])
  call<Catalog>(conn, '/api/catalog').then((c) => (javas = c.java)).catch(() => {})
  let java = $state(server.java ?? 25)
  let javaError = $state('')
  async function setJava() {
    javaError = ''
    try {
      await patchServer(conn, server.id, { java })
    } catch (e) {
      javaError = (e as Error).message
      java = server.java ?? 25
    }
  }
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
      <div class="card">
        <div class="row">
          <div><strong>Installé</strong><small>Paper, dernière version stable au moment de l'installation.</small></div>
          <div class="val">{server.version ?? 'Paper'}</div>
        </div>
        <label class="row">
          <div><strong>Java</strong><small>{javaError || 'Pris en compte au prochain démarrage. Installé à ce moment-là s\'il manque.'}</small></div>
          <select class="input" bind:value={java} onchange={setJava}>
            {#each javas as v (v)}<option value={v}>Java {v}{[8, 11, 17, 21, 25].includes(v) ? '' : ' (courte durée)'}</option>{/each}
          </select>
        </label>
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
      <div class="card">
        <div class="row">
          <div><strong>Build installé</strong><small>Build recommandé au moment de l'installation.</small></div>
          <div class="val">{server.version ?? 'Recommandé'}</div>
        </div>
        <div class="row">
          <div><strong>Mode</strong><small>{server.txadmin === false ? 'FXServer lancé directement avec server-data/server.cfg.' : 'Géré par txAdmin.'}</small></div>
          <div class="val">{server.txadmin === false ? ({ esx: 'ESX Legacy', qbcore: 'QBCore' } as Record<string, string>)[server.framework ?? ''] ?? 'Vanilla' : 'txAdmin'}</div>
        </div>
        <label class="row">
          <div><strong>Canal <span class="soon">Bientôt</span></strong><small>Recommandé est le plus sûr ; Dernier a les nouveautés.</small></div>
          <select class="input" disabled><option>Recommandé</option><option>Optionnel</option><option>Dernier</option><option>Build précis…</option></select>
        </label>
      </div>
    {/if}

  {:else if page === 'updates'}
    <h2>Mises à jour <span class="soon">Bientôt</span></h2>
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
  {/if}
</SubPages>

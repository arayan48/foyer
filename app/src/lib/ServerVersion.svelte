<script lang="ts">
  import { Package, Download, RefreshCw, History } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import Confirm from './Confirm.svelte'
  import { SOFTWARE, call, cmpVersion, patchServer, software, type Catalog, type Conn, type Server, type Software } from './api'

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

  // changer de logiciel ou de version (Minecraft)
  const now = $derived(software(server))
  let pick = $state<Software>(software(server).id)
  let list = $state<Catalog['minecraft'] | null>(null)
  let version = $state('')
  let error = $state('')
  let busy = $state(false)
  let ask = $state(false)
  const idle = $derived(server.status === 'stopped' || server.status === 'error')
  const same = $derived(pick === now.id && version === now.mc)
  const older = $derived(!!now.mc && !!version && cmpVersion(version, now.mc) < 0)

  async function loadVersions() {
    const want = pick
    list = null
    version = ''
    error = ''
    try {
      const l = await call<Catalog['minecraft']>(conn, `/api/catalog/${want}`)
      if (want !== pick) return // un autre choix entre-temps
      list = l
      // garde la version actuelle si ce logiciel l'a, sinon la plus récente
      version = (l.find((v) => v.id === now.mc) ?? l[0])?.id ?? ''
    } catch (e) {
      if (want === pick) error = `Liste des versions indisponible : ${(e as Error).message}`
    }
  }

  async function install() {
    busy = true
    error = ''
    try {
      await call(conn, `/api/servers/${server.id}/software`, { method: 'POST', body: JSON.stringify({ software: pick, version }) })
      java = list?.find((v) => v.id === version)?.java ?? java
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }

  const pages = [
    { id: 'software', label: mc ? 'Logiciel' : 'Artefacts', icon: Package },
    ...(mc ? [{ id: 'install', label: 'Changer', icon: Download }] : []),
    { id: 'updates', label: 'Mises à jour', icon: RefreshCw },
    { id: 'rollback', label: 'Retour arrière', icon: History },
  ]
  let page = $state('software')
</script>

<SubPages {pages} bind:page label="Réglages de version" onselect={() => page === 'install' && !list && loadVersions()}>
  {#if page === 'software'}
    {#if mc}
      <h2>Logiciel serveur</h2>
      <div class="card">
        <div class="row">
          <div><strong>Installé</strong></div>
          <div class="val">{server.version ?? 'Paper'} <button class="btn" onclick={() => { page = 'install'; if (!list) loadVersions() }}>Changer</button></div>
        </div>
        <label class="row">
          <div><strong>Java</strong><small>{javaError || 'Pris en compte au prochain démarrage. Installé à ce moment-là s\'il manque.'}</small></div>
          <select class="input" bind:value={java} onchange={setJava}>
            {#each javas as v (v)}<option value={v}>Java {v}{[8, 11, 17, 21, 25].includes(v) ? '' : ' (courte durée)'}</option>{/each}
          </select>
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

  {:else if page === 'install'}
    <h2>Changer de logiciel</h2>
    <form onsubmit={(e) => { e.preventDefault(); ask = true }}>
      <fieldset disabled={busy || !idle}>
        <legend>Logiciel</legend>
        <div class="choices">
          {#each Object.entries(SOFTWARE) as [k, s] (k)}
            <label class="choice" class:on={pick === k}>
              <input type="radio" name="software" value={k} bind:group={pick} onchange={loadVersions} />
              <span><strong>{s.label}</strong><small>{s.hint}</small></span>
            </label>
          {/each}
        </div>
      </fieldset>
      <label class="field">
        <span>Version de Minecraft</span>
        <select class="input" bind:value={version} disabled={!list || busy || !idle}>
          {#if !list}<option value="">{error ? 'Indisponible' : 'Chargement…'}</option>{/if}
          {#each list ?? [] as v (v.id)}<option value={v.id}>{v.id}</option>{/each}
        </select>
      </label>
      {#if error}<p class="err-text" role="alert">{error}</p>{/if}
      <div class="go">
        <button class="btn primary" disabled={!version || same || busy || !idle}><Download size={14} /> {server.status === 'installing' ? 'Installation…' : 'Installer'}</button>
        {#if server.status !== 'installing' && !idle}<small>Arrête le serveur d'abord.</small>{:else if same}<small>Déjà installé.</small>{/if}
      </div>
    </form>

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

<Confirm
  bind:open={ask}
  title="Installer {SOFTWARE[pick].label} {version} ?"
  text={older
    ? `Cette version est plus ancienne que ${now.mc} : un monde déjà ouvert en ${now.mc} risque d'être abîmé. Copie le dossier du serveur avant.`
    : 'Le monde et les réglages restent, mais un monde déjà créé peut mal supporter le changement. Copie le dossier du serveur avant.'}
  action="Installer"
  onconfirm={install}
/>

<style>
  form { display: grid; gap: 20px; max-width: 560px; margin-top: 14px; }
  fieldset { border: 0; padding: 0; margin: 0; }
  legend { color: var(--muted); font-size: 0.89rem; margin-bottom: 6px; }
  .choices { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 8px; }
  .choice { display: flex; padding: 8px 4px; cursor: pointer; opacity: 0.6; transition: opacity 150ms var(--ease); }
  .choice:hover { opacity: 0.85; }
  .choice.on { opacity: 1; }
  .choice.on strong { color: var(--primary); }
  .choice input { position: absolute; opacity: 0; pointer-events: none; }
  .choice:has(input:focus-visible) { outline: 2px solid var(--primary); outline-offset: 2px; border-radius: var(--r); }
  fieldset:disabled .choice { opacity: 0.4; cursor: not-allowed; }
  .choice > span { display: grid; }
  .choice small, .go small { color: var(--muted); font-size: 0.85rem; }
  .go { display: flex; align-items: center; gap: 12px; }
</style>

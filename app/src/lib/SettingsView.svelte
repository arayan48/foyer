<script lang="ts">
  import { LogOut, RefreshCw, Cpu, AppWindow, Plug } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import { applySettings, loadSettings } from './settings'
  import { bytes, type Conn, type Host } from './api'

  let { conn, sys, onlogout }: { conn: Conn; sys: Host | null; onlogout: () => void } = $props()

  // appli installée sur le serveur lui-même : se déconnecter de soi n'a pas de sens
  const local = new URL(conn.url).hostname === '127.0.0.1'
  const pages = [
    { id: 'system', label: 'Système', icon: Cpu },
    { id: 'app', label: 'Application', icon: AppWindow },
    ...(local ? [] : [{ id: 'conn', label: 'Connexion', icon: Plug }]),
  ]
  let page = $state('system')

  function since(sec = 0) {
    const d = Math.floor(sec / 86400), h = Math.floor((sec % 86400) / 3600), m = Math.floor((sec % 3600) / 60)
    return d ? `${d} j ${h} h` : h ? `${h} h ${m} min` : `${m} min`
  }
  const pct = (a: number, b: number) => (b ? Math.round((a / b) * 100) : 0)

  const inApp = '__TAURI_INTERNALS__' in window
  let s = $state(loadSettings())
  let autostart = $state(false)
  let version = $state('')
  let upd = $state<'idle' | 'checking' | 'none' | 'found' | 'installing' | 'error'>('idle')
  let newVersion = $state('')
  let install: (() => Promise<void>) | null = null

  $effect(() => {
    if (!inApp) return
    import('@tauri-apps/api/app').then(({ getVersion }) => getVersion()).then((v) => (version = v))
    import('@tauri-apps/plugin-autostart').then(({ isEnabled }) => isEnabled()).then((v) => (autostart = v))
  })

  function toggleTray() {
    s.closeToTray = !s.closeToTray
    applySettings(s)
  }

  async function toggleAutostart() {
    const a = await import('@tauri-apps/plugin-autostart')
    autostart ? await a.disable() : await a.enable()
    autostart = await a.isEnabled()
  }

  async function checkUpdate() {
    upd = 'checking'
    try {
      const { check } = await import('@tauri-apps/plugin-updater')
      const u = await check()
      if (!u) return (upd = 'none')
      newVersion = u.version
      install = async () => {
        await u.downloadAndInstall()
        const { relaunch } = await import('@tauri-apps/plugin-process')
        await relaunch()
      }
      upd = 'found'
    } catch {
      upd = 'error'
    }
  }

  async function doInstall() {
    upd = 'installing'
    try {
      await install?.()
    } catch {
      upd = 'error'
    }
  }
</script>

<header><h1>Paramètres</h1></header>

<SubPages {pages} bind:page label="Rubriques">
  {#if page === 'system'}
    <h2>Système</h2>
    <p class="lead">La machine où tourne l'agent Foyer.</p>
    {#if sys}
      <h3>Machine</h3>
      <div class="card">
        <div class="row"><div><strong>Nom</strong></div><div class="val">{sys.name || '—'}</div></div>
        <div class="row"><div><strong>Système</strong></div><div class="val">{sys.os || '—'}</div></div>
        {#if sys.kernel}<div class="row"><div><strong>Noyau / build</strong></div><div class="val num">{sys.kernel}</div></div>{/if}
        {#if sys.arch}<div class="row"><div><strong>Architecture</strong></div><div class="val">{sys.arch}</div></div>{/if}
        {#if sys.uptime != null}<div class="row"><div><strong>Allumée depuis</strong></div><div class="val num">{since(sys.uptime)}</div></div>{/if}
        <div class="row"><div><strong>Adresse de l'agent</strong></div><div class="val num">{conn.url}</div></div>
        {#if sys.agent}<div class="row"><div><strong>Version de l'agent</strong></div><div class="val num">{sys.agent}</div></div>{/if}
      </div>
      <h3>Matériel</h3>
      <div class="card">
        {#if sys.cpu_model}<div class="row"><div><strong>Processeur</strong></div><div class="val">{sys.cpu_model}</div></div>{/if}
        {#if sys.threads}<div class="row"><div><strong>Cœurs</strong></div><div class="val num">{sys.cores ?? '—'} cœurs physiques · {sys.threads} logiques</div></div>{/if}
        <div class="row"><div><strong>Charge processeur</strong></div><div class="val num">{Math.round(sys.cpu)} %</div></div>
        <div class="row"><div><strong>Mémoire</strong></div><div class="val num">{bytes(sys.mem_used)} utilisés sur {bytes(sys.mem_total)} ({pct(sys.mem_used, sys.mem_total)} %)</div></div>
        <div class="row"><div><strong>Disque des serveurs</strong></div><div class="val num">{bytes(sys.disk_free)} libres sur {bytes(sys.disk_total)}</div></div>
      </div>
    {:else}
      <p class="hint">Chargement…</p>
    {/if}

  {:else if page === 'app'}
    <h2>Application</h2>
    <p class="lead">Le comportement de Foyer sur cet ordinateur.</p>
    <h3>Fenêtre</h3>
    <div class="card">
      <label class="row">
        <div><strong>Rester en arrière-plan à la fermeture</strong><small>La croix cache la fenêtre. Pour quitter : clic droit sur l'icône près de l'horloge, puis « Quitter Foyer ».</small></div>
        <input type="checkbox" class="switch" checked={s.closeToTray} onchange={toggleTray} disabled={!inApp} />
      </label>
      <label class="row">
        <div><strong>Lancer Foyer au démarrage de l'ordinateur</strong><small>Foyer démarre caché, prêt à recevoir les alertes.</small></div>
        <input type="checkbox" class="switch" checked={autostart} onchange={toggleAutostart} disabled={!inApp} />
      </label>
    </div>
    {#if !inApp}<p class="hint">Ces réglages ne marchent que dans l'application installée.</p>{/if}
    <h3>Mises à jour</h3>
    <div class="card">
      <div class="row">
        <div>
          <strong>Version {version || '—'}</strong>
          <small>
            {#if upd === 'checking'}Recherche…
            {:else if upd === 'none'}Foyer est à jour.
            {:else if upd === 'found'}Foyer {newVersion} est disponible.
            {:else if upd === 'installing'}Installation, Foyer va redémarrer…
            {:else if upd === 'error'}Impossible de vérifier. Réessaie plus tard.
            {:else}Vérifiée au lancement puis toutes les 6 h.{/if}
          </small>
        </div>
        <div class="val">
          {#if upd === 'found' || upd === 'installing'}
            <button class="btn primary" onclick={doInstall} disabled={upd === 'installing'}>Mettre à jour</button>
          {:else}
            <button class="btn" onclick={checkUpdate} disabled={!inApp || upd === 'checking'}><RefreshCw size={14} /> Vérifier</button>
          {/if}
        </div>
      </div>
    </div>

  {:else}
    <h2>Connexion</h2>
    <p class="lead">L'agent que pilote cette application.</p>
    <div class="card">
      <div class="row">
        <div><strong>Agent</strong><small class="num">{conn.url}</small></div>
        <div class="val"><button class="btn danger" onclick={onlogout}><LogOut size={14} /> Se déconnecter</button></div>
      </div>
    </div>
  {/if}
</SubPages>

<style>
  header { margin-bottom: 20px; }
  .num { font-variant-numeric: tabular-nums; }
  .hint { color: var(--muted); font-size: 0.85rem; margin-top: 8px; }
</style>

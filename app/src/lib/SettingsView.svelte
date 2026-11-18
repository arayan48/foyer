<script lang="ts">
  import { LogOut, RefreshCw, Cpu, AppWindow, Plug } from 'lucide-svelte'
  import { applySettings, loadSettings } from './settings'
  import { bytes, type Conn, type Host } from './api'

  let { conn, sys, onlogout }: { conn: Conn; sys: Host | null; onlogout: () => void } = $props()

  // appli installée sur le serveur lui-même : se déconnecter de soi n'a pas de sens
  const local = new URL(conn.url).hostname === '127.0.0.1'
  const tabs = [
    { id: 'system', label: 'Système', icon: Cpu },
    { id: 'app', label: 'Application', icon: AppWindow },
    ...(local ? [] : [{ id: 'conn', label: 'Connexion', icon: Plug }]),
  ]
  let tab = $state('system')

  function keys(e: KeyboardEvent) {
    const d = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0
    if (!d) return
    const i = tabs.findIndex((t) => t.id === tab)
    tab = tabs[(i + d + tabs.length) % tabs.length].id
    queueMicrotask(() => (document.querySelector('[role=tab][aria-selected=true]') as HTMLElement)?.focus())
  }

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

<div class="tabs" role="tablist" aria-label="Rubriques" tabindex="-1" onkeydown={keys}>
  {#each tabs as t}
    <button role="tab" aria-selected={tab === t.id} tabindex={tab === t.id ? 0 : -1} onclick={() => (tab = t.id)}><t.icon size={15} /> {t.label}</button>
  {/each}
</div>

{#if tab === 'system'}
  {#if sys}
    <section>
      <h2>Machine</h2>
      <dl class="info">
        <dt>Nom</dt><dd>{sys.name || '—'}</dd>
        <dt>Système</dt><dd>{sys.os || '—'}</dd>
        {#if sys.kernel}<dt>Noyau / build</dt><dd class="num">{sys.kernel}</dd>{/if}
        {#if sys.arch}<dt>Architecture</dt><dd>{sys.arch}</dd>{/if}
        {#if sys.uptime != null}<dt>Allumée depuis</dt><dd class="num">{since(sys.uptime)}</dd>{/if}
        <dt>Adresse de l'agent</dt><dd class="num">{conn.url}</dd>
        {#if sys.agent}<dt>Version de l'agent</dt><dd class="num">{sys.agent}</dd>{/if}
      </dl>
    </section>
    <section>
      <h2>Matériel</h2>
      <dl class="info">
        {#if sys.cpu_model}<dt>Processeur</dt><dd>{sys.cpu_model}</dd>{/if}
        {#if sys.threads}<dt>Cœurs</dt><dd class="num">{sys.cores ?? '—'} cœurs · {sys.threads} threads</dd>{/if}
        <dt>Charge processeur</dt><dd class="num">{Math.round(sys.cpu)} %</dd>
        <dt>Mémoire</dt><dd class="num">{bytes(sys.mem_used)} utilisés sur {bytes(sys.mem_total)} ({pct(sys.mem_used, sys.mem_total)} %)</dd>
        <dt>Disque des serveurs</dt><dd class="num">{bytes(sys.disk_free)} libres sur {bytes(sys.disk_total)} ({pct(sys.disk_total - sys.disk_free, sys.disk_total)} % pris)</dd>
      </dl>
    </section>
  {:else}
    <p class="hint">Chargement…</p>
  {/if}
{:else if tab === 'app'}
<section>
  <h2>Fenêtre</h2>
  <div class="opt">
    <div>
      <strong>Rester en arrière-plan à la fermeture</strong>
      <p>La croix cache la fenêtre. Foyer reste dans la barre des tâches, près de l'horloge. Pour quitter : clic droit sur l'icône, puis « Quitter Foyer ».</p>
    </div>
    <button class="switch" role="switch" aria-checked={s.closeToTray} aria-label="Rester en arrière-plan à la fermeture" onclick={toggleTray} disabled={!inApp}><span></span></button>
  </div>
  <div class="opt">
    <div>
      <strong>Lancer Foyer au démarrage de l'ordinateur</strong>
      <p>Foyer démarre caché dans la barre, prêt à recevoir les alertes.</p>
    </div>
    <button class="switch" role="switch" aria-checked={autostart} aria-label="Lancer Foyer au démarrage" onclick={toggleAutostart} disabled={!inApp}><span></span></button>
  </div>
  {#if !inApp}<p class="hint">Ces réglages ne marchent que dans l'application installée.</p>{/if}
</section>

<section>
  <h2>Mises à jour</h2>
  <div class="opt">
    <div>
      <strong>Version {version || '—'}</strong>
      <p>
        {#if upd === 'checking'}Recherche…
        {:else if upd === 'none'}Foyer est à jour.
        {:else if upd === 'found'}Foyer {newVersion} est disponible.
        {:else if upd === 'installing'}Installation, Foyer va redémarrer…
        {:else if upd === 'error'}Impossible de vérifier. Réessaie plus tard.
        {:else}Foyer vérifie tout seul au lancement puis toutes les 6 h.{/if}
      </p>
    </div>
    {#if upd === 'found' || upd === 'installing'}
      <button class="btn primary" onclick={doInstall} disabled={upd === 'installing'}>Mettre à jour</button>
    {:else}
      <button class="btn" onclick={checkUpdate} disabled={!inApp || upd === 'checking'}><RefreshCw size={14} /> Vérifier</button>
    {/if}
  </div>
</section>
{:else}
<section>
  <h2>Connexion</h2>
  <div class="opt">
    <div>
      <strong>Agent : <span class="num">{conn.url}</span></strong>
      <p>Se déconnecter pour piloter un autre serveur.</p>
    </div>
    <button class="btn danger" onclick={onlogout}><LogOut size={14} /> Se déconnecter</button>
  </div>
</section>
{/if}

<style>
  header { margin-bottom: 20px; }
  .tabs { display: flex; gap: 2px; border-bottom: 1px solid var(--line); margin-bottom: 20px; overflow-x: auto; scrollbar-width: none; }
  .tabs button {
    display: inline-flex; align-items: center; gap: 7px; height: 38px; padding: 0 12px; border: 0; background: none; cursor: pointer;
    color: var(--muted); white-space: nowrap; border-bottom: 2px solid transparent; margin-bottom: -1px; border-radius: 6px 6px 0 0;
    transition: color 150ms, background 150ms, border-color 150ms;
  }
  .tabs button:hover { color: var(--ink); background: var(--surface); }
  .tabs button[aria-selected='true'] { color: var(--ink); border-bottom-color: var(--primary); }
  .tabs button[aria-selected='true'] :global(svg) { color: var(--primary); }
  .tabs button:focus-visible { outline-offset: -2px; }
  .info { display: grid; grid-template-columns: 180px 1fr; background: var(--surface); border-radius: var(--r); }
  .info dt, .info dd { padding: 11px 16px; border-top: 1px solid var(--line); }
  .info dt:first-of-type, .info dt:first-of-type + dd { border-top: 0; }
  .info dt { color: var(--muted); }
  .info dd { min-width: 0; overflow-wrap: anywhere; }
  section { max-width: 680px; margin-bottom: 28px; }
  h2 { font-size: 0.89rem; color: var(--muted); font-weight: 500; margin-bottom: 8px; }
  .opt { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 14px 16px; background: var(--surface); border-radius: var(--r); }
  .opt + .opt { margin-top: 2px; }
  .opt p { color: var(--muted); margin-top: 2px; max-width: 60ch; }
  .opt strong { font-weight: 600; }
  .num { font-variant-numeric: tabular-nums; font-weight: 500; }
  .hint { color: var(--muted); font-size: 0.85rem; margin-top: 8px; }
  .switch {
    flex: none; width: 40px; height: 24px; border-radius: 12px; border: 0; padding: 3px; cursor: pointer;
    background: var(--line); transition: background 200ms var(--ease);
  }
  .switch span { display: block; width: 18px; height: 18px; border-radius: 50%; background: var(--ink); transition: transform 200ms var(--ease); }
  .switch[aria-checked='true'] { background: var(--primary); }
  .switch[aria-checked='true'] span { transform: translateX(16px); background: var(--primary-ink); }
  .switch:disabled { opacity: 0.45; cursor: not-allowed; }
</style>

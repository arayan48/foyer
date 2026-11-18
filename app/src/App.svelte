<script lang="ts">
  import { Flame, LayoutDashboard, Server as ServerIcon, Settings as SettingsIcon } from 'lucide-svelte'
  import Connect from './lib/Connect.svelte'
  import ServerList from './lib/ServerList.svelte'
  import Dashboard from './lib/Dashboard.svelte'
  import NewServer from './lib/NewServer.svelte'
  import ServerView from './lib/ServerView.svelte'
  import Update from './lib/Update.svelte'
  import SettingsView from './lib/SettingsView.svelte'
  import { applySettings, loadSettings } from './lib/settings'
  import { isTauri } from '@tauri-apps/api/core'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { call, loadConn, saveConn, host, bytes, AuthError, type Conn, type Host, type Server } from './lib/api'

  let conn = $state<Conn | null>(loadConn())
  let servers = $state<Server[]>([])
  let sys = $state<Host | null>(null)
  let loading = $state(true)
  let view = $state<{ name: 'home' } | { name: 'list' } | { name: 'new' } | { name: 'settings' } | { name: 'server'; id: string }>({ name: 'home' })
  let toast = $state('')
  let toastTimer: ReturnType<typeof setTimeout>

  const current = $derived(view.name === 'server' ? servers.find((s) => s.id === (view as { id: string }).id) : undefined)
  const address = $derived(conn ? host(conn) : '')

  applySettings(loadSettings()) // transmet « rester en arrière-plan » à la fenêtre

  function say(m: string) {
    toast = m
    clearTimeout(toastTimer)
    toastTimer = setTimeout(() => (toast = ''), 5000)
  }

  async function refresh() {
    if (!conn) return
    try {
      ;[servers, sys] = await Promise.all([call<Server[]>(conn, '/api/servers'), call<Host>(conn, '/api/system')])
      loading = false
    } catch (e) {
      if (e instanceof AuthError) logout()
    }
  }

  $effect(() => {
    if (!conn) return
    loading = true
    refresh()
    const t = setInterval(refresh, 2000)
    return () => clearInterval(t)
  })

  function connect(c: Conn) {
    saveConn(c)
    conn = c
  }
  function logout() {
    saveConn(null)
    conn = null
    servers = []
    view = { name: 'home' }
  }

  async function act(s: Server, a: 'start' | 'stop' | 'restart') {
    try {
      await call(conn!, `/api/servers/${s.id}/${a}`, { method: 'POST' })
    } catch (e) {
      say((e as Error).message)
    }
    refresh()
  }

  async function remove(s: Server) {
    try {
      await call(conn!, `/api/servers/${s.id}`, { method: 'DELETE' })
      view = { name: 'list' }
      say(`« ${s.name} » supprimé.`)
    } catch (e) {
      say((e as Error).message)
    }
    refresh()
  }

  // la webview Tauri ignore target="_blank" : on ouvre les liens externes dans le navigateur
  function openExternal(e: MouseEvent) {
    const a = (e.target as Element).closest?.('a[target="_blank"]') as HTMLAnchorElement | null
    if (!a || !isTauri()) return
    e.preventDefault()
    openUrl(a.href)
  }
</script>

<svelte:document onclick={openExternal} />

{#if !conn}
  <Connect onconnect={connect} />
{:else}
  <div class="shell">
    <nav>
      <div class="brand"><Flame size={18} /> Foyer</div>
      <button class="nav-item" class:on={view.name === 'home'} aria-current={view.name === 'home' ? 'page' : undefined} onclick={() => (view = { name: 'home' })}>
        <LayoutDashboard size={16} /> Tableau de bord
      </button>
      <button class="nav-item" class:on={view.name !== 'settings' && view.name !== 'home'} aria-current={view.name !== 'settings' && view.name !== 'home' ? 'page' : undefined} onclick={() => (view = { name: 'list' })}>
        <ServerIcon size={16} /> Serveurs
      </button>
      <button class="nav-item" class:on={view.name === 'settings'} aria-current={view.name === 'settings' ? 'page' : undefined} onclick={() => (view = { name: 'settings' })}>
        <SettingsIcon size={16} /> Paramètres
      </button>
      <div class="machine">
        {#if sys}
          <dl>
            <dt>CPU</dt><dd><span class="bar"><span style:width="{sys.cpu}%"></span></span>{Math.round(sys.cpu)} %</dd>
            <dt>RAM</dt><dd><span class="bar"><span style:width="{(sys.mem_used / sys.mem_total) * 100}%"></span></span>{bytes(sys.mem_used)}</dd>
            <dt>Disque</dt><dd><span class="bar"><span style:width="{(1 - sys.disk_free / sys.disk_total) * 100}%"></span></span>{bytes(sys.disk_free)} libres</dd>
          </dl>
        {/if}
      </div>
    </nav>
    <main>
      {#if view.name === 'settings'}
        <SettingsView conn={conn} {sys} onlogout={logout} />
      {:else if view.name === 'home'}
        <Dashboard {servers} {sys} {loading} onnew={() => (view = { name: 'new' })} onopen={(s) => (view = { name: 'server', id: s.id })} onaction={act} />
      {:else if view.name === 'new'}
        <NewServer conn={conn} {servers} oncancel={() => (view = { name: 'list' })} oncreated={(s) => { refresh(); view = { name: 'server', id: s.id } }} />
      {:else if view.name === 'server' && current}
        <ServerView conn={conn} server={current} {address} onback={() => (view = { name: 'list' })} onaction={act} ondelete={remove} />
      {:else}
        <ServerList {servers} {address} {loading} onnew={() => (view = { name: 'new' })} onopen={(s) => (view = { name: 'server', id: s.id })} onaction={act} />
      {/if}
    </main>
  </div>
{/if}

<Update />

{#if toast}<div class="toast" role="status">{toast}</div>{/if}

<style>
  .shell { display: grid; grid-template-columns: 232px 1fr; height: 100%; }
  nav { background: var(--side); border-right: 1px solid var(--line); padding: 16px 12px; display: flex; flex-direction: column; gap: 4px; }
  .brand { display: flex; align-items: center; gap: 8px; font-weight: 700; font-size: 1rem; padding: 4px 10px 18px; color: var(--ink); }
  .brand :global(svg) { color: var(--primary); }
  .nav-item { display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 10px; border-radius: 8px; border: 0; background: none; color: var(--muted); cursor: pointer; text-align: left; transition: background 150ms, color 150ms; }
  .nav-item:hover { background: var(--surface); color: var(--ink); }
  .nav-item.on { background: var(--surface); color: var(--ink); }
  .machine { margin-top: auto; display: grid; gap: 4px; }
  dl { display: grid; grid-template-columns: auto 1fr; gap: 6px 10px; margin: 10px 10px 12px; font-size: 0.79rem; }
  dt { color: var(--muted); }
  dd { margin: 0; display: grid; gap: 3px; font-variant-numeric: tabular-nums; color: var(--muted); }
  .bar { height: 3px; border-radius: 2px; background: var(--surface-2); overflow: hidden; }
  .bar span { display: block; height: 100%; background: var(--primary); transition: width 400ms var(--ease); }
  main { overflow: auto; padding: 32px 40px; }
  .toast { position: fixed; bottom: 20px; left: 50%; translate: -50% 0; z-index: var(--z-toast); background: var(--surface-2); border: 1px solid var(--line); padding: 10px 16px; border-radius: var(--r); animation: up 200ms var(--ease); }
  @keyframes up { from { opacity: 0; translate: -50% 8px; } }
  @media (max-width: 700px) {
    .shell { grid-template-columns: 1fr; grid-template-rows: auto 1fr; }
    nav { flex-direction: row; align-items: center; border-right: 0; border-bottom: 1px solid var(--line); }
    .brand { padding: 0 10px; }
    .machine { margin: 0 0 0 auto; }
    .machine > :not(.nav-item) { display: none; }
    main { padding: 20px 16px; }
  }
</style>

<script lang="ts">
  import wordmark from './assets/wordmark.svg'
  import { LayoutDashboard, Server as ServerIcon, Settings as SettingsIcon, Activity, Users, Archive, CalendarClock, ScrollText, Globe, Cpu, MemoryStick, HardDrive } from 'lucide-svelte'
  import Logo from './lib/Logo.svelte'
  import GithubMark from './lib/GithubMark.svelte'
  import ServerSoon from './lib/ServerSoon.svelte'
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
  let view = $state<{ name: 'home' } | { name: 'list' } | { name: 'new' } | { name: 'settings' } | { name: 'server'; id: string } | { name: 'soon'; id: string }>({ name: 'home' })

  // pages prévues : visibles dès maintenant pour situer la suite
  const SOON = [
    { id: 'traffic', label: 'Trafic', icon: Activity, title: 'Analyse du trafic', text: 'Joueurs connectés dans le temps, heures de pointe, bande passante et latence par serveur.' },
    { id: 'players', label: 'Joueurs', icon: Users, title: 'Joueurs', text: 'Qui joue où, historique des connexions, bannissements et liste blanche au même endroit.' },
    { id: 'backups', label: 'Sauvegardes', icon: Archive, title: 'Sauvegardes', text: 'Copies automatiques des mondes, restauration en un clic.' },
    { id: 'tasks', label: 'Planification', icon: CalendarClock, title: 'Tâches planifiées', text: 'Redémarrage chaque nuit, annonces, sauvegardes à heure fixe.' },
    { id: 'logs', label: 'Journal', icon: ScrollText, title: 'Journal', text: "Tout ce qui s'est passé : démarrages, plantages, changements de réglages." },
  ]
  const soon = $derived(view.name === 'soon' ? SOON.find((p) => p.id === (view as { id: string }).id) : undefined)
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
      <div class="brand"><Logo size={32} /><img src={wordmark} alt="Foyer" height="17" /></div>
      <p class="group">Général</p>
      <button class="nav-item" class:on={view.name === 'home'} aria-current={view.name === 'home' ? 'page' : undefined} onclick={() => (view = { name: 'home' })}>
        <LayoutDashboard size={17} /> Tableau de bord
      </button>
      <button class="nav-item" class:on={['list', 'new', 'server'].includes(view.name)} aria-current={['list', 'new', 'server'].includes(view.name) ? 'page' : undefined} onclick={() => (view = { name: 'list' })}>
        <ServerIcon size={17} /> Serveurs <span class="n">{servers.length}</span>
      </button>
      <p class="group">Bientôt</p>
      {#each SOON as p (p.id)}
        <button class="nav-item later" class:on={soon?.id === p.id} aria-current={soon?.id === p.id ? 'page' : undefined} onclick={() => (view = { name: 'soon', id: p.id })}>
          <p.icon size={17} /> {p.label}
        </button>
      {/each}
      <div class="bottom">
        {#if sys}
          <div class="machine" aria-label="Machine">
            <div class="m"><Cpu size={13} /><span class="bar"><span style:width="{sys.cpu}%"></span></span><span class="v">{Math.round(sys.cpu)} %</span></div>
            <div class="m"><MemoryStick size={13} /><span class="bar"><span style:width="{(sys.mem_used / sys.mem_total) * 100}%"></span></span><span class="v">{bytes(sys.mem_used)}</span></div>
            <div class="m"><HardDrive size={13} /><span class="bar"><span style:width="{(1 - sys.disk_free / sys.disk_total) * 100}%"></span></span><span class="v">{bytes(sys.disk_free)}</span></div>
          </div>
        {/if}
        <button class="nav-item" class:on={view.name === 'settings'} aria-current={view.name === 'settings' ? 'page' : undefined} onclick={() => (view = { name: 'settings' })}>
          <SettingsIcon size={17} /> Paramètres
        </button>
        <div class="links">
          <a class="ext" href="https://github.com/arayan48/foyer" target="_blank" rel="noreferrer" aria-label="Code source sur GitHub" title="GitHub"><GithubMark size={16} /></a>
          <span class="ext off" title="Site web : bientôt" aria-label="Site web, bientôt disponible"><Globe size={16} /></span>
          <span class="host" title={address}>{address}</span>
        </div>
      </div>
    </nav>
    <main>
      <div class="page">
      {#if view.name === 'settings'}
        <SettingsView conn={conn} {sys} onlogout={logout} />
      {:else if view.name === 'home'}
        <Dashboard {servers} {sys} {loading} onnew={() => (view = { name: 'new' })} onopen={(s) => (view = { name: 'server', id: s.id })} onaction={act} />
      {:else if view.name === 'soon' && soon}
        <header class="page-head"><h1>{soon.label}</h1></header>
        <ServerSoon icon={soon.icon} title={soon.title} text={soon.text} />
      {:else if view.name === 'new'}
        <NewServer conn={conn} {servers} oncancel={() => (view = { name: 'list' })} oncreated={(s) => { refresh(); view = { name: 'server', id: s.id } }} />
      {:else if view.name === 'server' && current}
        <ServerView conn={conn} server={current} {address} onback={() => (view = { name: 'list' })} onaction={act} ondelete={remove} />
      {:else}
        <ServerList {servers} {address} {loading} onnew={() => (view = { name: 'new' })} onopen={(s) => (view = { name: 'server', id: s.id })} onaction={act} />
      {/if}
      </div>
    </main>
  </div>
{/if}

<Update />

{#if toast}<div class="toast" role="status">{toast}</div>{/if}

<style>
  .shell { display: grid; grid-template-columns: 248px minmax(0, 1fr); height: 100%; }
  nav { background: var(--side); border-right: 1px solid var(--line-soft); padding: 18px 12px 12px; display: flex; flex-direction: column; gap: 2px; overflow-y: auto; }
  .brand { display: flex; align-items: center; gap: 10px; font-weight: 700; font-size: 1.07rem; letter-spacing: -0.01em; padding: 0 8px 18px; color: var(--ink); }
  .group { font-size: 0.72rem; font-weight: 600; letter-spacing: 0.06em; text-transform: uppercase; color: var(--faint); padding: 14px 10px 6px; }
  .group:first-of-type { padding-top: 4px; }
  .nav-item { display: flex; align-items: center; gap: 11px; height: 2.4rem; padding: 0 10px; border-radius: var(--r-sm); border: 0; background: none; color: var(--muted); cursor: pointer; text-align: left; transition: background 150ms, color 150ms; }
  .nav-item:hover { background: var(--surface); color: var(--ink); }
  .nav-item.on { background: var(--surface-2); color: var(--ink); }
  .nav-item.on :global(svg) { color: var(--primary); }
  .nav-item.later { color: var(--faint); }
  .nav-item.later:hover, .nav-item.later.on { color: var(--ink); }
  .n { margin-left: auto; font-size: 0.75rem; color: var(--faint); font-variant-numeric: tabular-nums; }
  .bottom { margin-top: auto; padding-top: 16px; display: grid; gap: 4px; }
  .machine { display: grid; gap: 8px; padding: 12px; margin-bottom: 6px; border-radius: var(--r); background: var(--surface); border: 1px solid var(--line-soft); }
  .m { display: grid; grid-template-columns: auto 1fr auto; align-items: center; gap: 8px; color: var(--faint); font-size: 0.75rem; font-variant-numeric: tabular-nums; }
  .m .v { min-width: 4.2em; text-align: right; color: var(--muted); }
  .bar { height: 4px; border-radius: 2px; background: var(--surface-2); overflow: hidden; }
  .bar span { display: block; height: 100%; background: var(--primary); border-radius: 2px; transition: width 400ms var(--ease); }
  .links { display: flex; align-items: center; gap: 2px; padding: 8px 4px 0; margin-top: 6px; border-top: 1px solid var(--line-soft); }
  .ext { display: grid; place-items: center; width: 2rem; height: 2rem; border-radius: var(--r-sm); color: var(--muted); transition: background 150ms, color 150ms; }
  a.ext:hover { background: var(--surface); color: var(--ink); }
  .ext.off { opacity: 0.45; cursor: not-allowed; }
  .host { margin-left: auto; font-size: 0.72rem; color: var(--faint); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 120px; }
  main { overflow: auto; padding: 32px clamp(16px, 3vw, 48px); scrollbar-gutter: stable; }
  /* contenu centré et borné : pas de lignes étirées sur 4K */
  .page { max-width: 1440px; margin-inline: auto; }
  .page-head { margin-bottom: 20px; }
  .toast { position: fixed; bottom: 20px; left: 50%; translate: -50% 0; z-index: var(--z-toast); background: var(--surface-2); border: 1px solid var(--line); padding: 10px 16px; border-radius: var(--r); animation: up 200ms var(--ease); }
  @keyframes up { from { opacity: 0; translate: -50% 8px; } }
  @media (max-width: 860px) {
    .shell { grid-template-columns: 64px minmax(0, 1fr); }
    nav { padding: 14px 8px; align-items: center; }
    .brand img, .group, .n, .machine, .host { display: none; }
    .brand { padding: 0 0 12px; }
    .nav-item { width: 44px; justify-content: center; padding: 0; font-size: 0; gap: 0; }
    .links { flex-direction: column; border-top: 0; }
    main { padding: 20px 16px; }
  }
</style>

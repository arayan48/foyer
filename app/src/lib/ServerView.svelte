<script lang="ts">
  import { ArrowLeft, Play, Square, RotateCw, Terminal, SlidersHorizontal, Network, Package, KeyRound, FileCog, Users, FolderOpen, Archive, Puzzle } from 'lucide-svelte'
  import GameMark from './GameMark.svelte'
  import ServerConsole from './ServerConsole.svelte'
  import ServerGeneral from './ServerGeneral.svelte'
  import ServerNetwork from './ServerNetwork.svelte'
  import ServerVersion from './ServerVersion.svelte'
  import ServerFivem from './ServerFivem.svelte'
  import ServerProperties from './ServerProperties.svelte'
  import ServerPlugins from './ServerPlugins.svelte'
  import ServerSoon from './ServerSoon.svelte'
  import { GAMES, STATUS, type Conn, type Server } from './api'

  let { conn, server, address, onback, onaction, ondelete }: {
    conn: Conn
    server: Server
    address: string
    onback: () => void
    onaction: (s: Server, a: 'start' | 'stop' | 'restart') => void
    ondelete: (s: Server) => void
  } = $props()

  const mc = $derived(server.game === 'minecraft')
  const tabs = $derived([
    { id: 'console', label: 'Console', icon: Terminal },
    { id: 'general', label: 'Général', icon: SlidersHorizontal },
    { id: 'network', label: 'Réseau', icon: Network },
    { id: 'version', label: 'Version', icon: Package },
    mc ? { id: 'properties', label: 'Propriétés', icon: FileCog } : { id: 'fivem', label: 'FiveM', icon: KeyRound },
    ...(mc ? [{ id: 'plugins', label: 'Plugins', icon: Puzzle }] : []),
    { id: 'players', label: 'Joueurs', icon: Users, soon: true },
    { id: 'files', label: 'Fichiers', icon: FolderOpen, soon: true },
    { id: 'backups', label: 'Sauvegardes', icon: Archive, soon: true },
  ])
  let tab = $state('console')

  // flèches gauche/droite entre onglets (motif ARIA tablist)
  function keys(e: KeyboardEvent) {
    const i = tabs.findIndex((t) => t.id === tab)
    const d = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0
    if (!d) return
    tab = tabs[(i + d + tabs.length) % tabs.length].id
    queueMicrotask(() => (document.querySelector('[role=tab][aria-selected=true]') as HTMLElement)?.focus())
  }

  const canRun = $derived(server.status === 'stopped' || server.status === 'error')
</script>

<header>
  <button class="btn icon" onclick={onback} aria-label="Retour"><ArrowLeft size={16} /></button>
  <GameMark game={server.game} size={40} />
  <div class="title">
    <h1>{server.name}</h1>
    <p class="sub"><span class="st {server.status}">{STATUS[server.status]}</span> · {GAMES[server.game].label} · <span class="num">{address}:{server.port}</span></p>
  </div>
  <div class="actions">
    {#if canRun}
      <button class="btn primary" onclick={() => onaction(server, 'start')}><Play size={15} /> Démarrer</button>
    {:else}
      <button class="btn" disabled={server.status !== 'running'} onclick={() => onaction(server, 'restart')}><RotateCw size={15} /> Redémarrer</button>
      <button class="btn" disabled={server.status === 'installing' || server.status === 'stopping'} onclick={() => onaction(server, 'stop')}><Square size={14} /> Arrêter</button>
    {/if}
  </div>
</header>

<div class="tabs" role="tablist" aria-label="Sections du serveur" tabindex="-1" onkeydown={keys}>
  {#each tabs as t (t.id)}
    <button role="tab" id="tab-{t.id}" aria-selected={tab === t.id} aria-controls="page" tabindex={tab === t.id ? 0 : -1} class:soon-tab={t.soon} onclick={() => (tab = t.id)}>
      <t.icon size={15} /> {t.label}
    </button>
  {/each}
</div>

<div id="page" role="tabpanel" aria-labelledby="tab-{tab}">
  {#key server.id + tab}
    <div class="page">
      {#if tab === 'console'}
        <ServerConsole {conn} {server} {address} />
      {:else if tab === 'general'}
        <ServerGeneral {conn} {server} {ondelete} />
      {:else if tab === 'network'}
        <ServerNetwork {conn} {server} {address} />
      {:else if tab === 'version'}
        <ServerVersion {conn} {server} />
      {:else if tab === 'fivem'}
        <ServerFivem {server} {address} />
      {:else if tab === 'properties'}
        <ServerProperties />
      {:else if tab === 'plugins'}
        <ServerPlugins {conn} {server} />
      {:else if tab === 'players'}
        <ServerSoon icon={Users} title="Joueurs" text={mc ? 'Voir qui est connecté, expulser, bannir, donner les droits d\'opérateur et gérer la liste blanche.' : 'Voir qui est connecté, expulser et bannir. En attendant, txAdmin le fait déjà.'} />
      {:else if tab === 'files'}
        <ServerSoon icon={FolderOpen} title="Fichiers" text="Parcourir les fichiers du serveur, modifier les configurations, envoyer et télécharger." />
      {:else}
        <ServerSoon icon={Archive} title="Sauvegardes" text="Sauvegardes à la main ou planifiées, restauration en un clic, nombre de copies à garder." />
      {/if}
    </div>
  {/key}
</div>

<style>
  header { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; }
  .title { flex: 1; min-width: 0; }
  .title h1 { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { color: var(--muted); }
  .num { font-variant-numeric: tabular-nums; }
  .st.running { color: var(--ok); } .st.error { color: var(--err); }
  .st.starting, .st.installing, .st.stopping { color: var(--warn); }
  .actions { display: flex; gap: 6px; }
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
  .soon-tab { opacity: 0.7; }
  .page { max-width: 820px; animation: in 220ms var(--ease); }
  #page:has(:global(.console)) .page, #page:has(:global(.catalog)) .page { max-width: none; }
  @keyframes in { from { opacity: 0; translate: 0 4px; } }
  @media (prefers-reduced-motion: reduce) { .page { animation: none; } }
</style>

<script lang="ts">
  import { Play, Square, Plus, TriangleAlert } from 'lucide-svelte'
  import GameMark from './GameMark.svelte'
  import { GAMES, STATUS, bytes, type Host, type Server } from './api'

  let { servers, sys, loading, onopen, onaction, onnew }: {
    servers: Server[]
    sys: Host | null
    loading: boolean
    onopen: (s: Server) => void
    onaction: (s: Server, a: 'start' | 'stop') => void
    onnew: () => void
  } = $props()

  const live = (s: Server) => s.status === 'running' || s.status === 'starting'
  const running = $derived(servers.filter(live).length)
  const memPct = $derived(sys ? (sys.mem_used / sys.mem_total) * 100 : 0)
  const diskPct = $derived(sys ? (1 - sys.disk_free / sys.disk_total) * 100 : 0)
</script>

<header>
  <div>
    <h1>Tableau de bord</h1>
    <p class="sub">{running} serveur{running > 1 ? 's' : ''} en marche sur {servers.length}</p>
  </div>
  <button class="btn primary" onclick={onnew}><Plus size={16} /> Nouveau serveur</button>
</header>

<div class="stats">
  <div class="stat">
    <span class="lbl">Serveurs en marche</span>
    <strong>{running} / {servers.length}</strong>
  </div>
  <div class="stat">
    <span class="lbl">CPU machine</span>
    <strong>{sys ? Math.round(sys.cpu) : '–'} %</strong>
    <span class="bar"><span style:width="{sys?.cpu ?? 0}%"></span></span>
  </div>
  <div class="stat">
    <span class="lbl">RAM machine</span>
    <strong>{sys ? `${bytes(sys.mem_used)} / ${bytes(sys.mem_total)}` : '–'}</strong>
    <span class="bar"><span style:width="{memPct}%"></span></span>
  </div>
  <div class="stat" class:warn={diskPct > 85}>
    <span class="lbl">Disque</span>
    <strong>{sys ? Math.round(diskPct) : '–'} %</strong>
    <span class="bar"><span style:width="{diskPct}%"></span></span>
  </div>
</div>

{#if sys && diskPct > 85}
  <p class="alert" role="status"><TriangleAlert size={16} /> Disque presque plein : {bytes(sys.disk_free)} libres.</p>
{/if}

<h2>Serveurs</h2>
{#if loading}
  <div class="table skeleton"></div>
{:else if servers.length === 0}
  <p class="sub">Aucun serveur pour l'instant.</p>
{:else}
  <ul class="table">
    {#each servers as s (s.id)}
      <li>
        <button class="name" onclick={() => onopen(s)} aria-label="Ouvrir {s.name}">
          <GameMark game={s.game} />
          <span><strong>{s.name}</strong><span class="meta">{GAMES[s.game].label}</span></span>
        </button>
        <span class="status {s.status}"><i></i>{STATUS[s.status]}</span>
        <span class="num">{live(s) ? Math.round(s.cpu) + ' %' : '–'}</span>
        <span class="num">{live(s) ? bytes(s.mem) : '–'}</span>
        {#if s.status === 'stopped' || s.status === 'error'}
          <button class="btn icon" onclick={() => onaction(s, 'start')} aria-label="Démarrer {s.name}"><Play size={15} /></button>
        {:else if s.status === 'running'}
          <button class="btn icon" onclick={() => onaction(s, 'stop')} aria-label="Arrêter {s.name}"><Square size={15} /></button>
        {:else}
          <span></span>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  header { display: flex; justify-content: space-between; align-items: center; gap: 16px; margin-bottom: 24px; }
  .sub { color: var(--muted); }
  h2 { margin: 28px 0 12px; }
  .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(130px, 1fr)); gap: 12px; }
  .stat { background: var(--surface); border: 1px solid var(--line); border-radius: var(--r); padding: 14px 16px; display: grid; gap: 4px; animation: in 300ms var(--ease) both; }
  .stat .lbl { color: var(--muted); font-size: 0.79rem; }
  .stat strong { font-size: 1.27rem; font-weight: 600; font-variant-numeric: tabular-nums; }
  .bar { height: 4px; border-radius: 2px; background: var(--surface-2); overflow: hidden; margin-top: 4px; }
  .bar span { display: block; height: 100%; background: var(--primary); transition: width 400ms var(--ease); }
  .stat.warn strong { color: var(--warn); }
  .stat.warn .bar span { background: var(--warn); }
  .alert { display: flex; align-items: center; gap: 8px; margin-top: 12px; padding: 10px 14px; border: 1px solid var(--warn); border-radius: var(--r); color: var(--warn); }
  .table { list-style: none; margin: 0; padding: 0; background: var(--surface); border: 1px solid var(--line); border-radius: var(--r); overflow: hidden; }
  .table.skeleton { height: 120px; }
  li { display: grid; grid-template-columns: minmax(0, 2fr) 1fr 70px 80px 40px; align-items: center; gap: 12px; padding: 8px 12px; }
  li + li { border-top: 1px solid var(--line); }
  .name { display: flex; align-items: center; gap: 10px; border: 0; background: none; color: inherit; cursor: pointer; text-align: left; padding: 4px; border-radius: 8px; min-width: 0; transition: background 150ms; }
  .name:hover { background: var(--surface-2); }
  .name > span { display: grid; min-width: 0; }
  .meta { color: var(--muted); font-size: 0.79rem; }
  .num { font-variant-numeric: tabular-nums; color: var(--muted); }
  .status { display: flex; align-items: center; gap: 6px; color: var(--muted); }
  .status i { width: 7px; height: 7px; border-radius: 50%; background: var(--muted); }
  .status.running i { background: var(--ok); }
  .status.starting i, .status.stopping i, .status.installing i { background: var(--warn); }
  .status.error i { background: var(--err); }
  @keyframes in { from { opacity: 0; translate: 0 6px; } }
  @media (max-width: 700px) { li { grid-template-columns: minmax(0, 1fr) auto 40px; } .num { display: none; } }
</style>

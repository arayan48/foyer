<script lang="ts">
  import { Play, Square, RotateCw, Plus, TerminalSquare, Cpu, MemoryStick } from 'lucide-svelte'
  import GameMark from './GameMark.svelte'
  import { GAMES, STATUS, bytes, type Server } from './api'

  let { servers, address, loading, onopen, onaction, onnew }: {
    servers: Server[]
    address: string
    loading: boolean
    onopen: (s: Server) => void
    onaction: (s: Server, a: 'start' | 'stop' | 'restart') => void
    onnew: () => void
  } = $props()

  const live = (s: Server) => s.status === 'running' || s.status === 'starting'
</script>

<header>
  <div>
    <h1>Serveurs</h1>
    <p class="sub">{servers.length} serveur{servers.length > 1 ? 's' : ''} sur {address}</p>
  </div>
  <button class="btn primary" onclick={onnew}><Plus size={16} /> Nouveau serveur</button>
</header>

{#if loading}
  <ul class="list">
    {#each [0, 1] as _}<li class="row skeleton"></li>{/each}
  </ul>
{:else if servers.length === 0}
  <div class="empty">
    <h2>Aucun serveur pour l'instant</h2>
    <p>Crée ton premier serveur : Foyer télécharge le jeu, ouvre les ports sur le réseau local et le lance pour toi.</p>
    <button class="btn primary" onclick={onnew}><Plus size={16} /> Créer un serveur</button>
  </div>
{:else}
  <ul class="list">
    {#each servers as s (s.id)}
      <li class="row">
        <button class="main" onclick={() => onopen(s)} aria-label="Ouvrir {s.name}">
          <GameMark game={s.game} />
          <span class="name">
            <strong>{s.name}</strong>
            <span class="meta">{GAMES[s.game].label} · {address}:{s.port}</span>
          </span>
        </button>
        <span class="status {s.status}"><i></i>{STATUS[s.status]}</span>
        <span class="meter" title="Processeur">
          <span class="lbl"><Cpu size={14} /></span>
          <span class="bar"><span style:width="{Math.min(100, s.cpu)}%"></span></span>
          <span class="val">{live(s) ? Math.round(s.cpu) + ' %' : '–'}</span>
        </span>
        <span class="meter" title="Mémoire (limite {bytes(s.ram_mb * 1024 ** 2)})">
          <span class="lbl"><MemoryStick size={14} /></span>
          <span class="bar"><span style:width="{Math.min(100, (s.mem / (s.ram_mb * 1024 ** 2)) * 100)}%"></span></span>
          <span class="val">{live(s) ? bytes(s.mem) : '–'}</span>
        </span>
        <span class="actions">
          {#if s.status === 'stopped' || s.status === 'error'}
            <span class="ph"></span>
            <button class="btn act" onclick={() => onaction(s, 'start')}><Play size={15} /> Démarrer</button>
          {:else}
            <button class="btn icon" title="Redémarrer" aria-label="Redémarrer" disabled={s.status !== 'running'} onclick={() => onaction(s, 'restart')}><RotateCw size={15} /></button>
            <button class="btn act" disabled={s.status === 'installing' || s.status === 'stopping'} onclick={() => onaction(s, 'stop')}><Square size={14} /> Arrêter</button>
          {/if}
          <button class="btn icon" title="Console" aria-label="Console" onclick={() => onopen(s)}><TerminalSquare size={15} /></button>
        </span>
      </li>
    {/each}
  </ul>
{/if}

<style>
  header { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; margin-bottom: 24px; }
  .sub { color: var(--muted); margin-top: 2px; }
  .list { list-style: none; margin: 0; padding: 0; display: grid; gap: 8px; }
  .row {
    display: grid; grid-template-columns: minmax(200px, 1fr) 110px 130px 130px 190px; align-items: center; gap: 20px;
    padding: 12px 16px; border-radius: var(--r); background: var(--surface); border: 1px solid transparent;
    transition: border-color 150ms var(--ease);
  }
  .row:hover { border-color: var(--line); }
  .skeleton { height: 66px; animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.5; } }
  .main { display: flex; align-items: center; gap: 12px; background: none; border: 0; padding: 0; text-align: left; cursor: pointer; min-width: 0; }
  .name { display: grid; min-width: 0; }
  .name strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; }
  .meta { color: var(--muted); font-size: 0.89rem; font-variant-numeric: tabular-nums; }
  .status { display: inline-flex; align-items: center; gap: 8px; color: var(--muted); }
  .status i { width: 8px; height: 8px; border-radius: 50%; background: oklch(0.5 0.01 265); }
  .status.running { color: var(--ok); } .status.running i { background: var(--ok); box-shadow: 0 0 0 3px oklch(0.78 0.16 152 / 0.18); }
  .status.starting i, .status.installing i, .status.stopping i { background: var(--warn); animation: pulse 1s infinite; }
  .status.starting, .status.installing, .status.stopping { color: var(--warn); }
  .status.error { color: var(--err); } .status.error i { background: var(--err); }
  .meter { display: grid; grid-template-columns: 14px 1fr; grid-template-rows: auto auto; column-gap: 8px; font-size: 0.79rem; }
  .lbl { color: var(--muted); grid-row: span 2; align-self: center; }
  .bar { height: 4px; border-radius: 2px; background: var(--surface-2); overflow: hidden; align-self: end; }
  .bar span { display: block; height: 100%; background: var(--primary); border-radius: 2px; transition: width 400ms var(--ease); }
  .val { color: var(--muted); font-variant-numeric: tabular-nums; }
  .actions { display: flex; gap: 6px; justify-content: flex-end; }
  .ph { width: 32px; flex: none; }
  .act { min-width: 104px; justify-content: center; }
  .empty { padding: 48px 32px; border: 1px dashed var(--line); border-radius: var(--r); display: grid; gap: 10px; justify-items: start; }
  .empty p { color: var(--muted); max-width: 60ch; margin-bottom: 8px; }
  @media (max-width: 1080px) {
    .row { grid-template-columns: 1fr 110px auto; }
    .meter { display: none; }

  }
</style>

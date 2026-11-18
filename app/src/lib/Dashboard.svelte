<script module lang="ts">
  // historique gardé hors du composant : il survit quand on change de page
  const N = 60 // 60 relevés × 2 s = 2 min
  const hist: { cpu: number[]; mem: number[]; srv: Record<string, number[]> } = { cpu: [], mem: [], srv: {} }
  const push = (a: number[], v: number) => { a.push(v); if (a.length > N) a.shift() }
</script>

<script lang="ts">
  import { Play, Square, Plus, TriangleAlert, CircleCheck, Cpu, MemoryStick, HardDrive } from 'lucide-svelte'
  import { untrack } from 'svelte'
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
  const MB = 1024 ** 2

  let tick = $state(0) // force le rafraîchissement des courbes
  $effect(() => {
    if (!sys) return
    push(hist.cpu, sys.cpu)
    push(hist.mem, (sys.mem_used / sys.mem_total) * 100)
    for (const s of servers) push((hist.srv[s.id] ??= []), live(s) ? s.cpu : 0)
    untrack(() => tick++) // sinon l'effet se relance lui-même
  })

  const up = $derived(servers.filter(live))
  const down = $derived(servers.filter((s) => !live(s)))
  const diskPct = $derived(sys ? (1 - sys.disk_free / sys.disk_total) * 100 : 0)
  const memPct = $derived(sys ? (sys.mem_used / sys.mem_total) * 100 : 0)

  // répartition de la RAM : chaque serveur en marche, le reste de la machine, le libre
  const ramParts = $derived.by(() => {
    if (!sys) return []
    const srv = up.filter((s) => s.mem > 0).map((s, i) => ({ label: s.name, v: s.mem, cls: `s${i % 4}` }))
    const used = srv.reduce((a, p) => a + p.v, 0)
    return [...srv, { label: 'Système et autres', v: Math.max(0, sys.mem_used - used), cls: 'other' }]
  })

  const alerts = $derived.by(() => {
    const a: { lvl: 'err' | 'warn'; text: string; s?: Server }[] = []
    for (const s of servers) {
      if (s.status === 'error') a.push({ lvl: 'err', text: `${s.name} s'est arrêté sur une erreur`, s })
      else if (s.status === 'installing') a.push({ lvl: 'warn', text: `${s.name} est en cours d'installation`, s })
      else if (live(s) && s.mem > s.ram_mb * MB * 0.9) a.push({ lvl: 'warn', text: `${s.name} utilise presque toute sa RAM (${bytes(s.mem)} sur ${bytes(s.ram_mb * MB)})`, s })
    }
    if (sys) {
      const promised = up.reduce((n, s) => n + s.ram_mb * MB, 0)
      if (promised > sys.mem_total) a.push({ lvl: 'warn', text: `Les serveurs en marche peuvent demander ${bytes(promised)}, la machine n'en a que ${bytes(sys.mem_total)}` })
      if (diskPct > 85) a.push({ lvl: diskPct > 95 ? 'err' : 'warn', text: `Disque presque plein : ${bytes(sys.disk_free)} libres` })
      if (sys.cpu > 90) a.push({ lvl: 'warn', text: `Processeur saturé (${Math.round(sys.cpu)} %)` })
    }
    return a
  })

  // tracé d'une courbe 0-100 dans une boîte w×h
  function path(v: number[], w: number, h: number, close = false) {
    if (v.length < 2) return ''
    const x = (i: number) => (i / (N - 1)) * w + (N - v.length) * (w / (N - 1))
    const y = (n: number) => h - (Math.min(100, Math.max(0, n)) / 100) * h
    const d = v.map((n, i) => `${i ? 'L' : 'M'}${x(i).toFixed(1)},${y(n).toFixed(1)}`).join('')
    return close ? `${d}L${x(v.length - 1).toFixed(1)},${h}L${x(0).toFixed(1)},${h}Z` : d
  }
</script>

<header>
  <div>
    <h1>Tableau de bord</h1>
    <p class="sub">{alerts.length ? `${alerts.length} point${alerts.length > 1 ? 's' : ''} à surveiller` : 'Tout tourne normalement'}</p>
  </div>
  <button class="btn primary" onclick={onnew}><Plus size={16} /> Nouveau serveur</button>
</header>

<div class="grid">
  <!-- Machine : courbes en direct -->
  <section class="card machine" aria-label="Activité de la machine">
    <div class="chart">
      <div class="head">
        <span class="lbl"><Cpu size={14} /> Processeur</span>
        <strong>{sys ? Math.round(sys.cpu) : '–'} %</strong>
      </div>
      {#key tick}
        <svg viewBox="0 0 300 80" preserveAspectRatio="none" aria-hidden="true">
          <path class="area" d={path(hist.cpu, 300, 80, true)} />
          <path class="line" d={path(hist.cpu, 300, 80)} />
        </svg>
      {/key}
      <span class="axis">2 dernières minutes</span>
    </div>
    <div class="chart">
      <div class="head">
        <span class="lbl"><MemoryStick size={14} /> Mémoire</span>
        <strong>{sys ? `${bytes(sys.mem_used)} / ${bytes(sys.mem_total)}` : '–'}</strong>
      </div>
      {#key tick}
        <svg viewBox="0 0 300 80" preserveAspectRatio="none" aria-hidden="true">
          <path class="area alt" d={path(hist.mem, 300, 80, true)} />
          <path class="line alt" d={path(hist.mem, 300, 80)} />
        </svg>
      {/key}
      <span class="axis">{Math.round(memPct)} % utilisés</span>
    </div>
  </section>

  <!-- À surveiller -->
  <section class="card watch" aria-label="À surveiller">
    <h2>À surveiller</h2>
    {#if alerts.length === 0}
      <p class="calm"><CircleCheck size={18} /> Rien à signaler.</p>
    {:else}
      <ul>
        {#each alerts as a}
          <li class={a.lvl}>
            <TriangleAlert size={15} />
            {#if a.s}
              <button class="link" onclick={() => onopen(a.s!)}>{a.text}</button>
            {:else}
              <span>{a.text}</span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- Qui prend la RAM -->
  <section class="card ram" aria-label="Répartition de la mémoire">
    <h2>Qui utilise la mémoire</h2>
    {#if sys}
      <div class="stack" role="img" aria-label="Mémoire utilisée par serveur">
        {#each ramParts as p}
          <span class={p.cls} style:flex-grow={p.v} title="{p.label} : {bytes(p.v)}"></span>
        {/each}
        <span class="free" style:flex-grow={sys.mem_total - sys.mem_used} title="Libre : {bytes(sys.mem_total - sys.mem_used)}"></span>
      </div>
      <ul class="legend">
        {#each ramParts as p}
          <li><i class={p.cls}></i>{p.label}<span>{bytes(p.v)}</span></li>
        {/each}
        <li><i class="free"></i>Libre<span>{bytes(sys.mem_total - sys.mem_used)}</span></li>
      </ul>
      <div class="disk" class:warn={diskPct > 85}>
        <span class="lbl"><HardDrive size={14} /> Disque</span>
        <span class="bar"><span style:width="{diskPct}%"></span></span>
        <span class="num">{bytes(sys.disk_free)} libres</span>
      </div>
    {:else}
      <div class="skeleton"></div>
    {/if}
  </section>
</div>

<!-- En marche : une tuile par serveur avec sa courbe -->
<h2 class="title">En marche <span class="count">{up.length}</span></h2>
{#if loading}
  <div class="tiles"><div class="tile skeleton"></div></div>
{:else if up.length === 0}
  <p class="sub">Aucun serveur en marche.</p>
{:else}
  <div class="tiles">
    {#each up as s (s.id)}
      <article class="tile">
        <button class="open" onclick={() => onopen(s)} aria-label="Ouvrir {s.name}">
          <GameMark game={s.game} size={32} />
          <span><strong>{s.name}</strong><span class="meta">{GAMES[s.game].label} · port {s.port}</span></span>
        </button>
        {#key tick}
          <svg viewBox="0 0 200 36" preserveAspectRatio="none" aria-hidden="true">
            <path class="line" d={path(hist.srv[s.id] ?? [], 200, 36)} />
          </svg>
        {/key}
        <div class="foot">
          <span class="num">{Math.round(s.cpu)} % CPU</span>
          <span class="num">{bytes(s.mem)} / {bytes(s.ram_mb * MB)}</span>
          {#if s.status === 'running'}
            <button class="btn icon" onclick={() => onaction(s, 'stop')} aria-label="Arrêter {s.name}"><Square size={14} /></button>
          {:else}
            <span class="status">{STATUS[s.status]}</span>
          {/if}
        </div>
      </article>
    {/each}
  </div>
{/if}

<!-- Arrêtés : une ligne compacte, juste de quoi relancer -->
{#if down.length}
  <h2 class="title">À l'arrêt <span class="count">{down.length}</span></h2>
  <div class="chips">
    {#each down as s (s.id)}
      <div class="chip" class:err={s.status === 'error'}>
        <button class="open" onclick={() => onopen(s)}>{s.name} <span class="meta">{STATUS[s.status]}</span></button>
        {#if s.status === 'stopped' || s.status === 'error'}
          <button class="btn icon" onclick={() => onaction(s, 'start')} aria-label="Démarrer {s.name}"><Play size={14} /></button>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  header { display: flex; justify-content: space-between; align-items: center; gap: 16px; margin-bottom: 24px; }
  .sub { color: var(--muted); }
  h2 { font-size: 0.93rem; margin: 0 0 12px; }
  .title { margin: 28px 0 12px; display: flex; align-items: center; gap: 8px; }
  .count { font-size: 0.79rem; color: var(--muted); background: var(--surface-2); border-radius: 99px; padding: 1px 8px; font-weight: 500; }
  .lbl { display: inline-flex; align-items: center; gap: 6px; color: var(--muted); font-size: 0.79rem; }
  .num { font-variant-numeric: tabular-nums; color: var(--muted); font-size: 0.86rem; }
  .meta { color: var(--muted); font-size: 0.79rem; font-weight: 400; }
  .skeleton { background: var(--surface-2); border-radius: var(--r); min-height: 60px; }

  .grid { display: grid; grid-template-columns: 2fr 1fr; gap: 12px; }
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--r); padding: 16px; animation: in 300ms var(--ease) both; }
  .machine { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
  .ram { grid-column: 1 / -1; }

  .chart .head { display: flex; justify-content: space-between; align-items: baseline; gap: 8px; }
  .chart strong { white-space: nowrap; font-size: 1.13rem; font-weight: 600; font-variant-numeric: tabular-nums; }
  .chart svg { display: block; width: 100%; height: 80px; margin-top: 10px; }
  .axis { color: var(--muted); font-size: 0.72rem; }
  .line { fill: none; stroke: var(--primary); stroke-width: 2; vector-effect: non-scaling-stroke; stroke-linejoin: round; }
  .area { fill: color-mix(in oklch, var(--primary) 18%, transparent); }
  .line.alt { stroke: var(--ok); }
  .area.alt { fill: color-mix(in oklch, var(--ok) 15%, transparent); }

  .watch ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 8px; }
  .watch li { display: flex; gap: 8px; align-items: flex-start; font-size: 0.86rem; }
  .watch li :global(svg) { flex: none; margin-top: 2px; }
  .watch li.warn :global(svg) { color: var(--warn); }
  .watch li.err :global(svg) { color: var(--err); }
  .calm { display: flex; align-items: center; gap: 8px; color: var(--ok); margin: 0; }
  .link { border: 0; background: none; color: inherit; text-align: left; padding: 0; cursor: pointer; font: inherit; text-decoration: underline; text-decoration-color: var(--line); text-underline-offset: 3px; }
  .link:hover { text-decoration-color: var(--primary); }
  .link:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; border-radius: 4px; }

  .stack { display: flex; height: 14px; border-radius: 7px; overflow: hidden; gap: 2px; background: var(--surface-2); }
  .stack span { flex-basis: 0; transition: flex-grow 400ms var(--ease); }
  .s0 { background: var(--primary); }
  .s1 { background: var(--ok); }
  .s2 { background: var(--warn); }
  .s3 { background: oklch(0.72 0.12 250); }
  .other { background: var(--muted); opacity: 0.5; }
  .free { background: var(--surface-2); }
  .legend { list-style: none; margin: 12px 0 0; padding: 0; display: flex; flex-wrap: wrap; gap: 6px 18px; font-size: 0.86rem; }
  .legend li { display: flex; align-items: center; gap: 6px; }
  .legend i { width: 9px; height: 9px; border-radius: 3px; }
  .legend i.free { box-shadow: inset 0 0 0 1px var(--line); }
  .legend span { color: var(--muted); font-variant-numeric: tabular-nums; }
  .disk { display: grid; grid-template-columns: auto 1fr auto; align-items: center; gap: 12px; margin-top: 16px; padding-top: 14px; border-top: 1px solid var(--line); }
  .bar { height: 4px; border-radius: 2px; background: var(--surface-2); overflow: hidden; }
  .bar span { display: block; height: 100%; background: var(--primary); transition: width 400ms var(--ease); }
  .disk.warn .bar span { background: var(--warn); }
  .disk.warn .num { color: var(--warn); }

  .tiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 12px; }
  .tile { background: var(--surface); border: 1px solid var(--line); border-radius: var(--r); padding: 12px; display: grid; gap: 8px; animation: in 300ms var(--ease) both; }
  .tile.skeleton { min-height: 140px; }
  .tile svg { width: 100%; height: 36px; display: block; }
  .open { display: flex; align-items: center; gap: 10px; border: 0; background: none; color: inherit; cursor: pointer; text-align: left; padding: 4px; border-radius: 8px; min-width: 0; font: inherit; transition: background 150ms; }
  .open:hover { background: var(--surface-2); }
  .open:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }
  .open > span { display: grid; min-width: 0; }
  .open strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .foot { display: flex; align-items: center; gap: 12px; }
  .foot .btn { margin-left: auto; }
  .status { margin-left: auto; color: var(--warn); font-size: 0.79rem; }

  .chips { display: flex; flex-wrap: wrap; gap: 8px; }
  .chip { display: flex; align-items: center; gap: 2px; background: var(--surface); border: 1px solid var(--line); border-radius: 99px; padding: 2px 4px 2px 6px; }
  .chip .open { gap: 6px; padding: 4px 8px; border-radius: 99px; }
  .chip.err { border-color: var(--err); }
  .chip.err .meta { color: var(--err); }

  @keyframes in { from { opacity: 0; translate: 0 6px; } }
  @media (prefers-reduced-motion: reduce) { .card, .tile { animation: none; } }
  @media (max-width: 800px) { .grid, .machine { grid-template-columns: 1fr; } }
</style>

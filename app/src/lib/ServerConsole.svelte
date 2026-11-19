<script lang="ts">
  import { ArrowDown, Check, Copy, Eraser, ExternalLink, Search, Users } from 'lucide-svelte'
  import { bytes, openConsole, type Conn, type Console, type Server } from './api'

  let { conn, server, address }: { conn: Conn; server: Server; address: string } = $props()

  let lines = $state<string[]>([])
  let cmd = $state('')
  let connected = $state(false)
  let out: HTMLElement
  let ws: Console | null = null
  let stick = true

  // id seul : la liste est rafraîchie toutes les 2 s, l'objet server change mais pas l'id
  const serverId = $derived(server.id)

  $effect(() => {
    const id = serverId
    let closed = false
    let retry: ReturnType<typeof setTimeout>
    const open = async () => {
      lines = []
      try {
        const c = await openConsole(conn, id, (l) => {
          lines.push(l)
          if (lines.length > 2000) lines.splice(0, lines.length - 2000)
        }, () => {
          connected = false
          if (!closed) retry = setTimeout(open, 2000)
        })
        if (closed) return c.close()
        ws = c
        connected = true
      } catch {
        if (!closed) retry = setTimeout(open, 2000)
      }
    }
    open()
    return () => {
      closed = true
      clearTimeout(retry)
      ws?.close()
      ws = null
    }
  })

  $effect(() => {
    lines.length
    if (stick && out) queueMicrotask(() => (out.scrollTop = out.scrollHeight))
  })

  function send(e: SubmitEvent) {
    e.preventDefault()
    if (!cmd.trim() || !ws || !connected) return
    ws.send(cmd)
    cmd = ''
  }

  const kind = (l: string) => (l.startsWith('[foyer]') ? 'sys' : l.startsWith('> ') ? 'cmd' : /\b(ERROR|SEVERE|Exception)\b/.test(l) ? 'bad' : /\bWARN/.test(l) ? 'warn' : '')
  const idle = $derived(server.status === 'stopped' || server.status === 'error')
  const ramMax = $derived(server.ram_mb * 1024 ** 2)
  const memPct = $derived(idle ? 0 : Math.min(100, (server.mem / ramMax) * 100))
  const cpuPct = $derived(idle ? 0 : Math.min(100, server.cpu))
  const uptime = (s: number) => (s < 3600 ? `${Math.floor(s / 60)} min` : s < 86400 ? `${Math.floor(s / 3600)} h ${Math.floor((s % 3600) / 60)} min` : `${Math.floor(s / 86400)} j ${Math.floor((s % 86400) / 3600)} h`)

  let filter = $state('')
  const shown = $derived(filter ? lines.filter((l) => l.toLowerCase().includes(filter.toLowerCase())) : lines)
  let copied = $state(false)
  const full = $derived(`${address}:${server.port}`)
  async function copy() {
    await navigator.clipboard.writeText(full)
    copied = true
    setTimeout(() => (copied = false), 1500)
  }
  function bottom() {
    stick = true
    out.scrollTop = out.scrollHeight
  }
</script>

<section class="tiles">
  <div class="tile">
    <span>Processeur</span>
    <strong>{idle ? '–' : Math.round(server.cpu) + ' %'}</strong>
    <i class="meter"><b style:width="{cpuPct}%"></b></i>
  </div>
  <div class="tile">
    <span>Mémoire</span>
    <strong>{idle ? '–' : bytes(server.mem)} <small>/ {bytes(ramMax)}</small></strong>
    <i class="meter"><b style:width="{memPct}%" class:hi={memPct > 90}></b></i>
  </div>
  <div class="tile">
    <span>Joueurs</span>
    {#if server.players}
      <strong>{idle ? '–' : server.players.length} <small>/ {server.max_players ?? '?'}</small></strong>
    {:else}
      <strong class="later">Bientôt</strong>
    {/if}
  </div>
  <div class="tile">
    <span>En ligne depuis</span>
    {#if server.uptime_s != null}
      <strong>{idle ? '–' : uptime(server.uptime_s)}</strong>
    {:else}
      <strong class="later">Bientôt</strong>
    {/if}
  </div>
  <div class="tile">
    <span>Version</span>
    <strong>{server.version ?? '–'}</strong>
  </div>
  <div class="tile wide">
    <span>Adresse à donner aux joueurs</span>
    <div class="addr">
      <strong class="num">{full}</strong>
      <button class="icon" onclick={copy} title="Copier l'adresse" aria-label="Copier l'adresse">{#if copied}<Check size={14} />{:else}<Copy size={14} />{/if}</button>
      {#if server.admin_port}
        <a href="http://{address}:{server.admin_port}" target="_blank" rel="noreferrer">txAdmin <ExternalLink size={13} /></a>
      {/if}
    </div>
  </div>
</section>

<div class="body">
  <section class="console">
    <div class="bar">
      <span class="dot" class:on={connected}>{connected ? 'Connectée' : 'Reconnexion…'}</span>
      <span class="count num">{shown.length} ligne{shown.length > 1 ? 's' : ''}</span>
      <label class="search"><Search size={13} /><input bind:value={filter} placeholder="Filtrer" spellcheck="false" aria-label="Filtrer la console" /></label>
      <button class="icon" onclick={() => (lines = [])} title="Vider l'affichage" aria-label="Vider l'affichage"><Eraser size={14} /></button>
    </div>
    <div class="out">
      <pre bind:this={out} onscroll={() => (stick = out.scrollTop + out.clientHeight >= out.scrollHeight - 24)}>{#each shown as l}<span class={kind(l)}>{l}
</span>{:else}<span class="sys">{filter ? 'Aucune ligne ne correspond.' : "Rien pour l'instant. Démarre le serveur pour voir ses messages ici."}</span>{/each}</pre>
      {#if !stick}
        <button class="jump" onclick={bottom}><ArrowDown size={13} /> Revenir en bas</button>
      {/if}
    </div>
    <form onsubmit={send}>
      <span class="prompt">&gt;</span>
      <input bind:value={cmd} placeholder={server.status === 'running' ? 'Commande (ex. say Salut)' : 'Le serveur doit tourner pour recevoir des commandes'} disabled={server.status !== 'running' && server.status !== 'starting'} spellcheck="false" autocomplete="off" />
    </form>
  </section>

  <aside class="players">
    <div class="bar"><span>Joueurs connectés</span>{#if server.players}<span class="num">{server.players.length}</span>{/if}</div>
    {#if server.players?.length && !idle}
      <ul>{#each server.players as p}<li><Users size={13} />{p}</li>{/each}</ul>
    {:else}
      <p class="empty"><Users size={18} />{server.players ? 'Personne pour le moment.' : 'La liste des joueurs arrive dans une prochaine version.'}</p>
    {/if}
  </aside>
</div>

<style>
  .num { font-variant-numeric: tabular-nums; }
  .tiles { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 10px; margin-bottom: 14px; }
  .tile { display: grid; align-content: start; gap: 4px; padding: 12px 14px; background: var(--surface); border: 1px solid var(--line-soft); border-radius: var(--r); min-width: 0; }
  .tile.wide { grid-column: span 2; }
  .tile > span { color: var(--muted); font-size: 0.8rem; }
  .tile strong { font-size: 1.05rem; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tile small { color: var(--faint); font-weight: 400; font-size: 0.85rem; }
  .tile .later { color: var(--faint); font-weight: 500; font-size: 0.95rem; }
  .meter { display: block; height: 4px; margin-top: 4px; border-radius: 2px; background: var(--surface-2); overflow: hidden; }
  .meter b { display: block; height: 100%; background: var(--primary); transition: width 0.6s var(--ease); }
  .meter b.hi { background: var(--err); }
  .addr { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .addr a { margin-left: auto; color: var(--primary); display: inline-flex; gap: 4px; align-items: center; font-weight: 600; font-size: 0.85rem; text-decoration: none; white-space: nowrap; }
  .addr a:hover { color: var(--primary-hover); }
  .icon { display: inline-grid; place-items: center; width: 26px; height: 26px; border: 0; border-radius: var(--r-sm); background: none; color: var(--muted); cursor: pointer; }
  .icon:hover { background: var(--hover); color: var(--ink); }
  .icon:focus-visible, .jump:focus-visible, .addr a:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }
  .body { display: grid; grid-template-columns: 1fr 240px; gap: 10px; height: min(62vh, 640px); }
  @media (max-width: 900px) { .body { grid-template-columns: 1fr; height: auto; } .console { height: 60vh; } .tile.wide { grid-column: auto; } }
  .console { display: grid; grid-template-rows: auto 1fr auto; min-height: 0; border-radius: var(--r); overflow: hidden; background: oklch(0.12 0.006 265); border: 1px solid var(--line); }
  .bar { display: flex; align-items: center; gap: 12px; min-height: 40px; padding: 0 8px 0 14px; border-bottom: 1px solid var(--line); color: var(--muted); font-size: 0.85rem; }
  .count { color: var(--faint); margin-right: auto; }
  .dot { display: inline-flex; align-items: center; gap: 6px; }
  .dot::before { content: ''; width: 6px; height: 6px; border-radius: 50%; background: var(--warn); }
  .dot.on::before { background: var(--ok); }
  .search { display: inline-flex; align-items: center; gap: 6px; height: 28px; padding: 0 8px; border: 1px solid var(--line-soft); border-radius: var(--r-sm); color: var(--faint); }
  .search:focus-within { border-color: var(--primary); }
  .search input { width: 120px; border: 0; background: none; font-size: 0.8rem; outline: none; }
  .out { position: relative; min-height: 0; }
  pre { height: 100%; box-sizing: border-box; margin: 0; padding: 12px 14px; overflow: auto; font: 12.5px/1.55 ui-monospace, 'SF Mono', Consolas, monospace; white-space: pre-wrap; word-break: break-word; color: oklch(0.88 0.005 265); }
  pre .sys { color: var(--primary); } pre .cmd { color: var(--ok); } pre .bad { color: var(--err); } pre .warn { color: var(--warn); }
  .jump { position: absolute; right: 14px; bottom: 10px; display: inline-flex; align-items: center; gap: 6px; height: 28px; padding: 0 10px; border: 1px solid var(--line); border-radius: 999px; background: var(--surface-2); color: var(--ink); font-size: 0.8rem; cursor: pointer; box-shadow: var(--shadow); }
  .jump:hover { background: var(--hover); }
  .console form { display: flex; align-items: center; gap: 8px; padding: 0 14px; border-top: 1px solid var(--line); }
  .prompt { color: var(--primary); font-family: ui-monospace, monospace; }
  .console input:not(.search input) { flex: 1; height: 42px; border: 0; background: none; font-family: ui-monospace, 'SF Mono', Consolas, monospace; font-size: 13px; outline: none; }
  .console input:disabled { cursor: not-allowed; }
  .console input::placeholder { color: oklch(0.62 0.01 265); }
  .players { display: grid; grid-template-rows: auto 1fr; min-height: 0; border-radius: var(--r); background: var(--surface); border: 1px solid var(--line-soft); overflow: hidden; }
  .players .bar { justify-content: space-between; padding-right: 14px; border-color: var(--line-soft); }
  .players ul { margin: 0; padding: 6px; list-style: none; overflow: auto; }
  .players li { display: flex; align-items: center; gap: 8px; padding: 6px 8px; border-radius: var(--r-sm); font-size: 0.875rem; }
  .players li:hover { background: var(--hover); }
  .empty { display: grid; justify-items: center; align-content: center; gap: 8px; margin: 0; padding: 20px; color: var(--faint); font-size: 0.85rem; text-align: center; }
</style>

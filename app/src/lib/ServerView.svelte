<script lang="ts">
  import { ArrowLeft, Play, Square, RotateCw, Trash2, ExternalLink } from 'lucide-svelte'
  import GameMark from './GameMark.svelte'
  import { GAMES, STATUS, bytes, consoleUrl, type Conn, type Server } from './api'

  let { conn, server, address, onback, onaction, ondelete }: {
    conn: Conn
    server: Server
    address: string
    onback: () => void
    onaction: (s: Server, a: 'start' | 'stop' | 'restart') => void
    ondelete: (s: Server) => void
  } = $props()

  let lines = $state<string[]>([])
  let cmd = $state('')
  let connected = $state(false)
  let out: HTMLElement
  let ws: WebSocket | null = null
  let stick = true
  let confirmDelete = $state(false)

  // id seul : la liste est rafraîchie toutes les 2 s, l'objet server change mais pas l'id
  const serverId = $derived(server.id)

  $effect(() => {
    const id = serverId
    let closed = false
    let retry: ReturnType<typeof setTimeout>
    const open = () => {
      lines = []
      ws = new WebSocket(consoleUrl(conn, id))
      ws.onopen = () => (connected = true)
      ws.onmessage = (e) => {
        lines.push(e.data)
        if (lines.length > 2000) lines.splice(0, lines.length - 2000)
      }
      ws.onclose = () => {
        connected = false
        if (!closed) retry = setTimeout(open, 2000)
      }
    }
    open()
    return () => {
      closed = true
      clearTimeout(retry)
      ws?.close()
    }
  })

  $effect(() => {
    lines.length
    if (stick && out) queueMicrotask(() => (out.scrollTop = out.scrollHeight))
  })

  function send(e: SubmitEvent) {
    e.preventDefault()
    if (!cmd.trim() || ws?.readyState !== WebSocket.OPEN) return
    ws.send(cmd)
    cmd = ''
  }

  const kind = (l: string) => (l.startsWith('[foyer]') ? 'sys' : l.startsWith('> ') ? 'cmd' : /\b(ERROR|SEVERE|Exception)\b/.test(l) ? 'bad' : /\bWARN/.test(l) ? 'warn' : '')
  const canRun = $derived(server.status === 'stopped' || server.status === 'error')
</script>

<header>
  <button class="btn icon" onclick={onback} aria-label="Retour"><ArrowLeft size={16} /></button>
  <GameMark game={server.game} size={36} />
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

<section class="facts">
  <div><span>Processeur</span><strong>{canRun ? '–' : Math.round(server.cpu) + ' %'}</strong></div>
  <div><span>Mémoire</span><strong>{canRun ? '–' : bytes(server.mem)}</strong><small>/ {bytes(server.ram_mb * 1024 ** 2)}</small></div>
  <div><span>Adresse à donner aux joueurs</span><strong class="num">{address}:{server.port}</strong></div>
  {#if server.admin_port}
    <div><span>txAdmin</span><a href="http://{address}:{server.admin_port}" target="_blank" rel="noreferrer">Ouvrir <ExternalLink size={13} /></a></div>
  {/if}
</section>

<section class="console">
  <div class="bar">
    <span>Console</span>
    <span class="dot" class:on={connected}>{connected ? 'Connectée' : 'Reconnexion…'}</span>
  </div>
  <pre bind:this={out} onscroll={() => (stick = out.scrollTop + out.clientHeight >= out.scrollHeight - 24)}>{#each lines as l}<span class={kind(l)}>{l}
</span>{:else}<span class="sys">Rien pour l'instant. Démarre le serveur pour voir ses messages ici.</span>{/each}</pre>
  <form onsubmit={send}>
    <span class="prompt">&gt;</span>
    <input bind:value={cmd} placeholder={server.status === 'running' ? 'Commande (ex. say Salut)' : 'Le serveur doit tourner pour recevoir des commandes'} disabled={server.status !== 'running' && server.status !== 'starting'} spellcheck="false" autocomplete="off" />
  </form>
</section>

<footer>
  {#if confirmDelete}
    <span class="warn">Supprimer « {server.name} » et tous ses fichiers ? C'est définitif.</span>
    <button class="btn" onclick={() => (confirmDelete = false)}>Annuler</button>
    <button class="btn danger" onclick={() => ondelete(server)}><Trash2 size={14} /> Supprimer définitivement</button>
  {:else}
    <button class="btn danger" onclick={() => (confirmDelete = true)}><Trash2 size={14} /> Supprimer le serveur</button>
  {/if}
</footer>

<style>
  header { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; }
  .title { flex: 1; min-width: 0; }
  .title h1 { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sub { color: var(--muted); }
  .num { font-variant-numeric: tabular-nums; }
  .st.running { color: var(--ok); } .st.error { color: var(--err); }
  .st.starting, .st.installing, .st.stopping { color: var(--warn); }
  .actions { display: flex; gap: 6px; }
  .facts { display: flex; flex-wrap: wrap; gap: 32px; padding: 0 2px 20px; }
  .facts div { display: grid; gap: 2px; }
  .facts span { color: var(--muted); font-size: 0.85rem; }
  .facts strong { font-size: 1.125rem; font-weight: 600; font-variant-numeric: tabular-nums; }
  .facts small { color: var(--muted); }
  .facts a { color: var(--primary); display: inline-flex; gap: 4px; align-items: center; font-weight: 600; text-decoration: none; }
  .console { display: grid; grid-template-rows: auto 1fr auto; height: min(62vh, 640px); border-radius: var(--r); overflow: hidden; background: oklch(0.12 0.006 357); border: 1px solid var(--line); }
  .bar { display: flex; justify-content: space-between; padding: 8px 14px; border-bottom: 1px solid var(--line); color: var(--muted); font-size: 0.85rem; }
  .dot { display: inline-flex; align-items: center; gap: 6px; }
  .dot::before { content: ''; width: 6px; height: 6px; border-radius: 50%; background: var(--warn); }
  .dot.on::before { background: var(--ok); }
  pre { margin: 0; padding: 12px 14px; overflow: auto; font: 12.5px/1.55 ui-monospace, 'SF Mono', Consolas, monospace; white-space: pre-wrap; word-break: break-word; color: oklch(0.88 0.005 357); }
  pre .sys { color: var(--primary); } pre .cmd { color: var(--ok); } pre .bad { color: var(--err); } pre .warn { color: var(--warn); }
  .console form { display: flex; align-items: center; gap: 8px; padding: 0 14px; border-top: 1px solid var(--line); }
  .prompt { color: var(--primary); font-family: ui-monospace, monospace; }
  .console input { flex: 1; height: 42px; border: 0; background: none; font-family: ui-monospace, 'SF Mono', Consolas, monospace; font-size: 13px; outline: none; }
  .console input::placeholder { color: oklch(0.62 0.01 357); }
  footer { display: flex; align-items: center; gap: 8px; justify-content: flex-end; margin-top: 20px; }
  .warn { color: var(--err); margin-right: auto; }
</style>

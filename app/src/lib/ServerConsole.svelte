<script lang="ts">
  import { ExternalLink } from 'lucide-svelte'
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
</script>

<section class="facts">
  <div><span>Processeur</span><strong>{idle ? '–' : Math.round(server.cpu) + ' %'}</strong></div>
  <div><span>Mémoire</span><strong>{idle ? '–' : bytes(server.mem)}</strong><small>/ {bytes(server.ram_mb * 1024 ** 2)}</small></div>
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

<style>
  .num { font-variant-numeric: tabular-nums; }
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
</style>

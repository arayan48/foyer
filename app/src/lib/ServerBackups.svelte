<script lang="ts">
  import { onMount } from 'svelte'
  import { Archive, RotateCcw, Trash2 } from 'lucide-svelte'
  import Confirm from './Confirm.svelte'
  import { bytes, call, when, type Conn, type Server } from './api'

  // auto : seulement les sauvegardes automatiques (page Retour arrière)
  let { conn, server, auto = false }: { conn: Conn; server: Server; auto?: boolean } = $props()

  type Backup = { id: number; auto: boolean; reason: string; version: string | null; size: number }
  let data = $state<{ backups: Backup[]; running: boolean; error: string | null } | null>(null)
  let failed = $state('')
  let busy = $state(false)
  let target = $state<Backup | null>(null)
  let askRestore = $state(false)
  let askDelete = $state(false)

  const base = $derived(`/api/servers/${server.id}/backups`)
  const list = $derived(data?.backups.filter((b) => !auto || b.auto) ?? [])
  const idle = $derived(server.status === 'stopped' || server.status === 'error')
  const off = $derived(busy || !data || data.running)
  const error = $derived(failed || data?.error || '')

  async function load() {
    data = await call(conn, base)
  }
  onMount(() => {
    load().catch((e) => (failed = (e as Error).message))
    const t = setInterval(() => load().catch(() => {}), 2000)
    return () => clearInterval(t)
  })

  async function run(method: string, path = '') {
    busy = true
    failed = ''
    try {
      await call(conn, base + path, { method })
      await load()
    } catch (e) {
      failed = (e as Error).message
    } finally {
      busy = false
    }
  }
</script>

<header>
  <h2>{auto ? 'Revenir en arrière' : 'Sauvegardes'}</h2>
  {#if !auto}<button class="btn primary" disabled={off || !(idle || server.status === 'running')} onclick={() => run('POST')}><Archive size={15} /> Sauvegarder</button>{/if}
</header>
{#if data?.running}<p class="muted" role="status">En cours…</p>{/if}
{#if error}<p class="err-text" role="alert">{error}</p>{/if}
{#if !auto && server.game === 'fivem'}<p class="muted">La base de données n'est pas incluse.</p>{/if}

{#if data && list.length === 0}
  <p class="muted">{auto ? 'Une sauvegarde se fait avant chaque changement de version.' : 'Aucune sauvegarde.'}</p>
{:else if data}
  {#if !idle && !data.running}<p class="muted">Arrête le serveur pour restaurer.</p>{/if}
  <ul>
    {#each list as b (b.id)}
      <li>
        <span class="who"><strong>{when(b.id)}</strong><small>{b.reason}{b.version ? ` · ${b.version}` : ''} · {bytes(b.size)}</small></span>
        <button class="btn" disabled={off || !idle} onclick={() => { target = b; askRestore = true }}><RotateCcw size={15} /> Restaurer</button>
        <button class="btn icon ghost danger" disabled={off} onclick={() => { target = b; askDelete = true }} title="Effacer" aria-label="Effacer la sauvegarde du {when(b.id)}"><Trash2 size={15} /></button>
      </li>
    {/each}
  </ul>
{/if}

<Confirm bind:open={askRestore} title="Restaurer la sauvegarde ?" text="Le monde, les réglages et la version{target?.version ? ` (${target.version})` : ''} reviennent au {target ? when(target.id) : ''}. L'état actuel est sauvegardé avant." action="Restaurer" onconfirm={() => run('POST', `/${target!.id}`)} />
<Confirm bind:open={askDelete} title="Effacer la sauvegarde ?" text="Elle ne pourra plus être restaurée." action="Effacer" onconfirm={() => run('DELETE', `/${target!.id}`)} />

<style>
  header { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 14px; }
  h2 { font-size: 1.125rem; }
  ul { list-style: none; margin: 0; padding: 0; }
  li { display: flex; align-items: center; gap: 12px; padding: 12px 0; }
  .who { display: grid; flex: 1; min-width: 0; }
  .who strong { font-weight: 600; }
  .who small, .muted { color: var(--muted); }
  .muted, .err-text { margin-bottom: 12px; }
</style>

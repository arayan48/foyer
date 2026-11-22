<script lang="ts">
  import { RotateCcw, Trash2 } from 'lucide-svelte'
  import GameMark from './GameMark.svelte'
  import Confirm from './Confirm.svelte'
  import { GAMES, call, when, type Conn, type Server } from './api'

  let { conn, onrestored }: { conn: Conn; onrestored: (s: Server) => void } = $props()

  type Trashed = Server & { deleted_at: number }
  let list = $state<Trashed[] | null>(null)
  let error = $state('')
  let busy = $state(false)
  let target = $state<Trashed | null>(null)
  let ask = $state(false)

  async function load() {
    try {
      list = await call<Trashed[]>(conn, '/api/trash')
    } catch (e) {
      error = (e as Error).message
    }
  }
  load()

  async function run(f: () => Promise<void>) {
    busy = true
    error = ''
    try {
      await f()
      await load()
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }
  const restore = (s: Trashed) => run(async () => onrestored(await call<Server>(conn, `/api/trash/${s.id}`, { method: 'POST' })))
  const purge = () => run(() => call(conn, `/api/trash/${target!.id}`, { method: 'DELETE' }))
  const left = (s: Trashed) => Math.max(0, Math.ceil(30 - (Date.now() / 1000 - s.deleted_at) / 86400))
</script>

<header><h1>Corbeille</h1></header>
{#if error}<p class="err-text" role="alert">{error}</p>{/if}
{#if list === null}
  {#if !error}<p class="muted">Chargement…</p>{/if}
{:else if list.length === 0}
  <p class="muted">La corbeille est vide.</p>
{:else}
  <ul>
    {#each list as s (s.id)}
      <li>
        <GameMark game={s.game} />
        <span class="who"><strong>{s.name}</strong><small>{GAMES[s.game].label} · supprimé le {when(s.deleted_at)} · effacé dans {left(s)} j</small></span>
        <button class="btn" disabled={busy} onclick={() => restore(s)}><RotateCcw size={15} /> Restaurer</button>
        <button class="btn icon ghost danger" disabled={busy} onclick={() => { target = s; ask = true }} title="Effacer" aria-label="Effacer {s.name}"><Trash2 size={15} /></button>
      </li>
    {/each}
  </ul>
{/if}
<Confirm bind:open={ask} title="Effacer « {target?.name} » ?" text="Le monde et tous les fichiers sont perdus." action="Effacer" onconfirm={purge} />

<style>
  header { margin-bottom: 20px; }
  ul { list-style: none; margin: 0; padding: 0; }
  li { display: flex; align-items: center; gap: 12px; padding: 12px; border-radius: var(--r-sm); transition: background 150ms var(--ease); }
  li:hover { background: var(--line-soft); }
  .who { display: grid; flex: 1; min-width: 0; }
  .who strong { font-weight: 600; overflow-wrap: anywhere; }
  .who small, .muted { color: var(--muted); }
  .err-text { margin-bottom: 12px; }
</style>

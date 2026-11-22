<script lang="ts">
  import { Users, Shield, ListChecks, Ban, UserX, X, Plus } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import Confirm from './Confirm.svelte'
  import { call, type Conn, type Server } from './api'

  let { conn, server }: { conn: Conn; server: Server } = $props()

  type Lists = { online: string[]; ops: string[]; whitelist: string[]; banned: string[] }
  const pages = [
    { id: 'online', label: 'Connectés', icon: Users },
    { id: 'ops', label: 'Opérateurs', icon: Shield },
    { id: 'whitelist', label: 'Liste blanche', icon: ListChecks },
    { id: 'banned', label: 'Bannis', icon: Ban },
  ]
  // ajout et retrait pour chaque liste
  const ACTIONS: Record<string, { add: string; remove: string; undo: string; empty: string }> = {
    ops: { add: 'op', remove: 'deop', undo: 'Retirer', empty: 'Aucun opérateur.' },
    whitelist: { add: 'whitelist_add', remove: 'whitelist_remove', undo: 'Retirer', empty: 'Liste blanche vide.' },
    banned: { add: 'ban', remove: 'pardon', undo: 'Débannir', empty: 'Aucun joueur banni.' },
  }
  let page = $state('online')
  let lists = $state<Lists | null>(null)
  let name = $state('')
  let error = $state('')
  let busy = $state(false)
  let toBan = $state('')
  let askBan = $state(false)
  const running = $derived(server.status === 'running')
  const shown = $derived(lists?.[page as keyof Lists] ?? [])

  // svelte-ignore state_referenced_locally
  const base = `/api/servers/${server.id}/players`
  async function load() {
    try {
      lists = await call<Lists>(conn, base)
    } catch (e) {
      error = (e as Error).message
    }
  }
  $effect(() => {
    load()
    const t = setInterval(load, 3000)
    return () => clearInterval(t)
  })

  async function act(player: string, action: string) {
    busy = true
    error = ''
    try {
      await call(conn, `${base}/${encodeURIComponent(player)}/${action}`, { method: 'POST' })
      // le jeu écrit ses listes juste après la commande
      await new Promise((ok) => setTimeout(ok, 500))
      await load()
      return true
    } catch (e) {
      error = (e as Error).message
      return false
    } finally {
      busy = false
    }
  }
  async function add(e: SubmitEvent) {
    e.preventDefault()
    if (await act(name.trim(), ACTIONS[page].add)) name = ''
  }
</script>

<SubPages {pages} bind:page label="Joueurs" onselect={() => { name = ''; error = '' }}>
  <h2>{pages.find((p) => p.id === page)?.label}</h2>
  {#if !running}<p class="lead">Démarre le serveur pour modifier les joueurs.</p>{/if}
  {#if error}<p class="err-text" role="alert">{error}</p>{/if}

  {#if page !== 'online'}
    <form onsubmit={add}>
      <input class="input" bind:value={name} maxlength="17" placeholder="Pseudo" aria-label="Pseudo" disabled={!running} />
      <button class="btn primary" disabled={!running || busy || !name.trim()}><Plus size={15} /> {page === 'banned' ? 'Bannir' : 'Ajouter'}</button>
    </form>
  {/if}

  {#if lists === null}
    {#if !error}<p class="muted">Chargement…</p>{/if}
  {:else if shown.length === 0}
    <p class="muted">{page === 'online' ? 'Personne pour le moment.' : ACTIONS[page].empty}</p>
  {:else}
    <ul>
      {#each shown as p (p)}
        <li>
          <span class="nm">{p}{#if page === 'online' && lists.ops.includes(p)}<span class="badge">Opérateur</span>{/if}</span>
          {#if page === 'online'}
            <button class="btn ghost" disabled={!running || busy} onclick={() => act(p, lists?.ops.includes(p) ? 'deop' : 'op')}><Shield size={15} /> {lists.ops.includes(p) ? 'Retirer op' : 'Passer op'}</button>
            <button class="btn ghost" disabled={!running || busy} onclick={() => act(p, 'kick')}><UserX size={15} /> Expulser</button>
            <button class="btn ghost danger" disabled={!running || busy} onclick={() => { toBan = p; askBan = true }}><Ban size={15} /> Bannir</button>
          {:else}
            <button class="btn ghost" disabled={!running || busy} onclick={() => act(p, ACTIONS[page].remove)}><X size={15} /> {ACTIONS[page].undo}</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</SubPages>

<Confirm bind:open={askBan} title="Bannir « {toBan} » ?" text="Le joueur est expulsé et ne peut plus revenir." action="Bannir" onconfirm={() => act(toBan, 'ban')} />

<style>
  form { display: flex; gap: 8px; margin: 12px 0; max-width: 420px; }
  form input { flex: 1; min-width: 0; }
  ul { list-style: none; margin: 10px 0 0; padding: 0; }
  li { display: flex; align-items: center; flex-wrap: wrap; gap: 4px; padding: 5px 4px 5px 10px; border-radius: var(--r-sm); transition: background 150ms var(--ease); }
  li:hover { background: var(--line-soft); }
  .nm { display: flex; align-items: center; gap: 10px; flex: 1; min-width: 0; font-weight: 500; }
  .muted { color: var(--muted); margin-top: 10px; }
</style>

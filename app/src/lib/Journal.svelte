<script lang="ts">
  import { call, when, type Conn, type Server } from './api'

  let { conn, servers }: { conn: Conn; servers: Server[] } = $props()

  type Line = { at: number; ip: string; action: string; status: number | null }
  let list = $state<Line[] | null>(null)
  let error = $state('')

  async function load() {
    try {
      list = await call<Line[]>(conn, '/api/journal')
      error = ''
    } catch (e) {
      error = (e as Error).message
    }
  }
  $effect(() => {
    load()
    const t = setInterval(load, 5000)
    return () => clearInterval(t)
  })

  const PLAYER: Record<string, string> = { kick: 'expulsé', ban: 'banni', pardon: 'débanni', op: 'passé opérateur', deop: 'plus opérateur', whitelist_add: 'ajouté à la liste blanche', whitelist_remove: 'retiré de la liste blanche' }
  const FILE: Record<string, string> = { POST: 'Dossier créé', PUT: 'Fichier écrit', PATCH: 'Renommé', DELETE: 'Supprimé' }
  const decode = (s: string) => { try { return decodeURIComponent(s) } catch { return s } }

  /** Traduit une ligne de l'agent ; une action inconnue reste affichée telle quelle. */
  function read(action: string): { what: string; id?: string } {
    const con = action.match(/^CONSOLE (\w+) ([\s\S]*)$/)
    if (con) return { what: `Commande : ${con[2]}`, id: con[1] }
    const [method, url = ''] = action.split(' ')
    const u = new URL(url, 'http://x')
    const [, , root, id, sub, a, b] = u.pathname.split('/').map(decode)
    if (root === 'shutdown') return { what: "Agent arrêté" }
    if (root === 'trash') return { what: method === 'POST' ? 'Restauré de la corbeille' : 'Effacé de la corbeille', id }
    if (root !== 'servers') return { what: action }
    if (!id) return { what: 'Serveur créé' }
    if (!sub) return { what: method === 'DELETE' ? 'Mis à la corbeille' : 'Réglages modifiés', id }
    if (sub === 'start') return { what: 'Démarrage', id }
    if (sub === 'stop') return { what: 'Arrêt', id }
    if (sub === 'restart') return { what: 'Redémarrage', id }
    if (sub === 'properties') return { what: 'Propriétés modifiées', id }
    if (sub === 'plugins') return { what: method === 'POST' ? 'Plugin ajouté' : `Plugin ${method === 'DELETE' ? 'supprimé' : 'modifié'} : ${a}`, id }
    if (sub === 'files') return { what: `${FILE[method] ?? method} : ${u.searchParams.get('path') ?? ''}`, id }
    if (sub === 'players') return { what: `${a} ${PLAYER[b] ?? b}`, id }
    return { what: action, id }
  }
  const rows = $derived((list ?? []).map((l) => {
    const r = read(l.action)
    return { ...l, what: r.what, where: r.id ? (servers.find((s) => s.id === r.id)?.name ?? r.id) : '', failed: l.status !== null && l.status >= 400 }
  }))
</script>

<header><h1>Journal</h1></header>
{#if error}<p class="err-text" role="alert">{error}</p>{/if}
{#if list === null}
  {#if !error}<p class="muted">Chargement…</p>{/if}
{:else if rows.length === 0}
  <p class="muted">Aucune action pour l'instant.</p>
{:else}
  <table>
    <thead><tr><th>Date</th><th>Serveur</th><th>Action</th><th>Adresse</th></tr></thead>
    <tbody>
      {#each rows as r}
        <tr>
          <td class="dim">{when(r.at)}</td>
          <td>{r.where}</td>
          <td class="what">{r.what}{#if r.failed}<span class="badge err">Refusé</span>{/if}</td>
          <td class="dim">{r.ip}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  header { margin-bottom: 20px; }
  table { width: 100%; border-collapse: collapse; }
  th { text-align: left; font-weight: 500; font-size: 0.85rem; color: var(--faint); padding: 0 12px 8px; }
  td { padding: 8px 12px; vertical-align: top; overflow-wrap: anywhere; }
  tbody tr { transition: background 150ms var(--ease); }
  tbody tr:hover { background: var(--line-soft); }
  .dim { color: var(--muted); font-size: 0.87rem; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .what .badge { margin-left: 8px; }
  .muted { color: var(--muted); }
</style>

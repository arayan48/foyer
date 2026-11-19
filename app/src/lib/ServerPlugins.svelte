<script lang="ts">
  import { Puzzle, Search, Download, Trash2, Sparkles, ChevronLeft, ChevronRight, ExternalLink, Heart, Clock, Check } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import Confirm from './Confirm.svelte'
  import { call, bytes, type Conn, type Server } from './api'

  let { conn, server }: { conn: Conn; server: Server } = $props()
  type Plugin = { file: string; enabled: boolean; size: number }
  type Hit = { id: string; slug?: string; title: string; description: string; author: string; downloads: number; follows: number; updated: string; icon: string | null; categories?: string[] }

  const base = `/api/servers/${server.id}/plugins`
  const pages = [
    { id: 'installed', label: 'Installés', icon: Puzzle },
    { id: 'add', label: 'Ajouter', icon: Download },
    { id: 'shaders', label: 'Shaders', icon: Sparkles },
  ]
  const SORTS = [
    ['relevance', 'Pertinence'],
    ['downloads', 'Popularité'],
    ['follows', 'Favoris'],
    ['newest', 'Plus récents'],
    ['updated', 'Mis à jour'],
  ]
  // catégories Modrinth (identifiants de l'API, libellés en français)
  const PLUGIN_CATS: Record<string, string> = {
    '': 'Tout', management: 'Gestion', economy: 'Économie', 'game-mechanics': 'Mécaniques', utility: 'Outils', social: 'Social',
    minigame: 'Mini-jeux', worldgen: 'Génération', optimization: 'Optimisation', adventure: 'Aventure', magic: 'Magie',
    mobs: 'Créatures', technology: 'Technologie', transportation: 'Transport', storage: 'Stockage', library: 'Bibliothèques',
  }
  const SHADER_CATS: Record<string, string> = {
    '': 'Tout', 'vanilla-like': 'Proche du jeu', 'semi-realistic': 'Semi-réaliste', realistic: 'Réaliste', fantasy: 'Fantaisie',
    cartoon: 'Cartoon', potato: 'PC modeste', low: 'Léger', high: 'Puissant', 'path-tracing': 'Path tracing',
  }

  let page = $state('installed')
  let list = $state<Plugin[] | null>(null)
  let error = $state('')
  let busy = $state('')
  let toDelete = $state<Plugin | null>(null)
  let askDelete = $state(false)

  // une recherche par onglet : plugins (via l'agent) et shaders (Modrinth direct, rien à installer sur le serveur)
  const blank = () => ({ q: '', sort: 'downloads', cat: '', page: 0, hits: null as Hit[] | null, total: 0, per: 20, loading: false })
  let ps = $state(blank())
  let ss = $state(blank())

  async function load() {
    try { list = await call<Plugin[]>(conn, base) } catch (e) { error = (e as Error).message }
  }
  load()

  async function run(key: string, f: () => Promise<unknown>) {
    busy = key
    error = ''
    try { await f(); await load() } catch (e) { error = (e as Error).message } finally { busy = '' }
  }
  const toggle = (p: Plugin) => run(p.file, () => call(conn, `${base}/${encodeURIComponent(p.file)}`, { method: 'PATCH', body: JSON.stringify({ enabled: !p.enabled }) }))
  const remove = (p: Plugin) => run(p.file, () => call(conn, `${base}/${encodeURIComponent(p.file)}`, { method: 'DELETE' }))
  const install = (h: Hit) => run(h.id, () => call(conn, base, { method: 'POST', body: JSON.stringify({ project: h.id }) }))

  async function searchPlugins() {
    ps.loading = true
    error = ''
    try {
      const qs = new URLSearchParams({ q: ps.q, sort: ps.sort, page: String(ps.page), cat: ps.cat })
      const r = await call<{ hits: Hit[]; total: number; per_page: number }>(conn, `${base}/search?${qs}`)
      ;[ps.hits, ps.total, ps.per] = [r.hits, r.total, r.per_page]
    } catch (e) { error = (e as Error).message } finally { ps.loading = false }
  }

  async function searchShaders() {
    ss.loading = true
    error = ''
    try {
      const facets = [['project_type:shader'], ...(ss.cat ? [[`categories:${ss.cat}`]] : [])]
      const qs = new URLSearchParams({ query: ss.q, index: ss.sort, limit: String(ss.per), offset: String(ss.page * ss.per), facets: JSON.stringify(facets) })
      const r = await fetch(`https://api.modrinth.com/v2/search?${qs}`).then((r) => (r.ok ? r.json() : Promise.reject(new Error(`Modrinth : ${r.status}`))))
      ss.hits = r.hits.map((h: any) => ({ id: h.project_id, slug: h.slug, title: h.title, description: h.description, author: h.author, downloads: h.downloads, follows: h.follows, updated: h.date_modified, icon: h.icon_url, categories: h.display_categories }))
      ss.total = r.total_hits
    } catch (e) { error = (e as Error).message } finally { ss.loading = false }
  }

  // un filtre qui change revient à la page 1
  const refilter = (s: typeof ps, go: () => void) => { s.page = 0; go() }
  const goPage = (s: typeof ps, n: number, go: () => void) => { s.page = n; go(); document.querySelector('main')?.scrollTo({ top: 0, behavior: 'smooth' }) }

  function onselect() {
    if (page === 'add' && !ps.hits) searchPlugins()
    if (page === 'shaders' && !ss.hits) searchShaders()
  }

  const name = (f: string) => f.replace(/\.jar(\.disabled)?$/, '')
  const short = (n: number) => (n >= 1e6 ? (n / 1e6).toFixed(1).replace('.', ',') + ' M' : n >= 1e3 ? Math.round(n / 1e3) + ' k' : String(n))
  const ago = (d: string) => {
    const days = Math.floor((Date.now() - new Date(d).getTime()) / 864e5)
    return days < 1 ? "aujourd'hui" : days < 30 ? `il y a ${days} j` : days < 365 ? `il y a ${Math.floor(days / 30)} mois` : `il y a ${Math.floor(days / 365)} an${days >= 730 ? 's' : ''}`
  }
  const installed = (h: Hit) => list?.some((p) => name(p.file).toLowerCase().startsWith(h.title.toLowerCase().replace(/\s+/g, '')))
  const pageCount = (s: typeof ps) => Math.max(1, Math.min(Math.ceil(s.total / s.per), 500))
  // numéros affichés : 1 … 4 5 [6] 7 8 … 40
  function pageNums(cur: number, n: number) {
    const set = new Set([0, n - 1, cur - 2, cur - 1, cur, cur + 1, cur + 2].filter((i) => i >= 0 && i < n))
    const out: (number | null)[] = []
    let prev = -1
    for (const i of [...set].sort((a, b) => a - b)) {
      if (i - prev > 1) out.push(null)
      out.push(i)
      prev = i
    }
    return out
  }
</script>

{#snippet browser(s: typeof ps, go: () => void, cats: Record<string, string>, placeholder: string, shader: boolean)}
  <form class="bar" onsubmit={(e) => { e.preventDefault(); refilter(s, go) }}>
    <div class="search">
      <Search size={16} />
      <input class="input" type="search" {placeholder} bind:value={s.q} aria-label="Rechercher" />
    </div>
    <select class="input sort" bind:value={s.sort} onchange={() => refilter(s, go)} aria-label="Trier par">
      {#each SORTS as [v, l]}<option value={v}>{l}</option>{/each}
    </select>
    <button class="btn primary" disabled={s.loading}>Rechercher</button>
  </form>
  <div class="cats" role="group" aria-label="Catégories">
    {#each Object.entries(cats) as [v, l]}
      <button class="cat" aria-pressed={s.cat === v} onclick={() => { s.cat = v; refilter(s, go) }}>{l}</button>
    {/each}
  </div>

  {#if s.hits}
    <p class="count">{s.total.toLocaleString('fr-FR')} résultat{s.total > 1 ? 's' : ''}{s.total ? ` · page ${s.page + 1} sur ${pageCount(s)}` : ''}</p>
  {/if}

  {#if s.loading && !s.hits}
    <div class="grid">{#each Array(6) as _}<div class="tile skel"></div>{/each}</div>
  {:else if s.hits?.length === 0}
    <div class="empty"><Search size={20} /><p>Rien trouvé. Essaie un autre mot ou une autre catégorie.</p></div>
  {:else if s.hits}
    <div class="grid" class:dim={s.loading}>
      {#each s.hits as h (h.id)}
        <article class="tile">
          <div class="top">
            {#if h.icon}<img src={h.icon} alt="" width="44" height="44" loading="lazy" />{:else}<span class="ph">{#if shader}<Sparkles size={18} />{:else}<Puzzle size={18} />{/if}</span>{/if}
            <div class="ttl"><strong>{h.title}</strong><small>par {h.author}</small></div>
          </div>
          <p class="desc">{h.description}</p>
          <div class="stats">
            <span title="Téléchargements"><Download size={13} />{short(h.downloads)}</span>
            <span title="Favoris"><Heart size={13} />{short(h.follows ?? 0)}</span>
            {#if h.updated}<span title="Dernière mise à jour"><Clock size={13} />{ago(h.updated)}</span>{/if}
          </div>
          <div class="act">
            {#if shader}
              <a class="btn" href="https://modrinth.com/shader/{h.slug}" target="_blank" rel="noreferrer"><ExternalLink size={14} /> Voir sur Modrinth</a>
            {:else if installed(h)}
              <span class="btn done" aria-disabled="true"><Check size={14} /> Installé</span>
            {:else}
              <button class="btn" disabled={!!busy} onclick={() => install(h)}><Download size={14} /> {busy === h.id ? 'Installation…' : 'Installer'}</button>
            {/if}
          </div>
        </article>
      {/each}
    </div>

    {#if pageCount(s) > 1}
      <nav class="pager" aria-label="Pages">
        <button class="btn icon" disabled={s.page === 0 || s.loading} onclick={() => goPage(s, s.page - 1, go)} aria-label="Page précédente"><ChevronLeft size={16} /></button>
        {#each pageNums(s.page, pageCount(s)) as n}
          {#if n === null}<span class="gap">…</span>
          {:else}<button class="btn icon num" aria-current={n === s.page ? 'page' : undefined} disabled={s.loading} onclick={() => goPage(s, n, go)}>{n + 1}</button>{/if}
        {/each}
        <button class="btn icon" disabled={s.page >= pageCount(s) - 1 || s.loading} onclick={() => goPage(s, s.page + 1, go)} aria-label="Page suivante"><ChevronRight size={16} /></button>
      </nav>
    {/if}
  {/if}
{/snippet}

<SubPages {pages} bind:page label="Plugins" {onselect}>
  {#if error}<p class="err" role="alert">{error}</p>{/if}

  {#if page === 'installed'}
    <h2>Plugins installés</h2>
    <p class="lead">Dossier <code>plugins</code> du serveur. Les changements comptent au prochain redémarrage.</p>
    {#if list === null}
      <p class="muted">Chargement…</p>
    {:else if list.length === 0}
      <div class="empty">
        <Puzzle size={22} />
        <p>Aucun plugin pour l'instant.</p>
        <button class="btn primary" onclick={() => { page = 'add'; onselect() }}><Download size={14} /> Parcourir le catalogue</button>
      </div>
    {:else}
      <div class="card">
        {#each list as p (p.file)}
          <div class="row">
            <div class="who">
              <span class="ph sm"><Puzzle size={15} /></span>
              <div><strong class:off={!p.enabled}>{name(p.file)}</strong><small>{p.enabled ? 'Actif' : 'Désactivé'} · {bytes(p.size)}</small></div>
            </div>
            <div class="val acts">
              <input type="checkbox" class="switch" checked={p.enabled} disabled={busy === p.file} onchange={() => toggle(p)} aria-label="Activer {name(p.file)}" />
              <button class="btn icon ghost danger" disabled={busy === p.file} onclick={() => { toDelete = p; askDelete = true }} aria-label="Supprimer {name(p.file)}"><Trash2 size={15} /></button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  {:else if page === 'add'}
    <h2>Ajouter un plugin</h2>
    <p class="lead">Catalogue Modrinth, filtré pour Paper{server.version ? ` et ${server.version.replace('Paper ', 'Minecraft ')}` : ''}.</p>
    {@render browser(ps, searchPlugins, PLUGIN_CATS, 'EssentialsX, LuckPerms, WorldEdit…', false)}

  {:else if page === 'shaders'}
    <h2>Shaders</h2>
    <p class="lead">Ils tournent sur le PC de chaque joueur (Iris ou OptiFine), pas sur le serveur. Choisis-en un ici et partage le lien à tes joueurs.</p>
    {@render browser(ss, searchShaders, SHADER_CATS, 'Complementary, BSL, Sildur…', true)}
  {/if}
</SubPages>

<Confirm bind:open={askDelete} title="Supprimer {toDelete ? name(toDelete.file) : ''} ?" text="Le fichier est retiré du dossier plugins. Ses réglages restent, tu pourras le réinstaller." onconfirm={() => toDelete && remove(toDelete)} />

<style>
  .err { color: var(--err); margin-bottom: 12px; }
  .muted { color: var(--muted); }
  .off { color: var(--muted); text-decoration: line-through; }
  .acts { display: flex; align-items: center; gap: 6px; justify-content: flex-end; }
  .row:has(.acts) { grid-template-columns: 1fr auto; }
  .who { display: flex !important; gap: 12px; align-items: center; }
  .who > div { display: grid; gap: 2px; min-width: 0; }
  code { font-size: 0.92em; }

  .bar { display: flex; gap: 8px; margin-bottom: 12px; flex-wrap: wrap; }
  .search { position: relative; flex: 1 1 240px; display: flex; }
  .search :global(svg) { position: absolute; left: 12px; top: 50%; translate: 0 -50%; color: var(--faint); pointer-events: none; }
  .search .input { flex: 1; padding-left: 38px; }
  .sort { width: auto; min-width: 150px; }

  .cats { display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 16px; }
  .cat { height: 1.9rem; padding: 0 0.8rem; border-radius: 999px; border: 1px solid var(--line-soft); background: none; color: var(--muted); cursor: pointer; font-size: 0.85rem; transition: background 120ms, color 120ms, border-color 120ms; }
  .cat:hover { color: var(--ink); background: var(--surface); }
  .cat[aria-pressed='true'] { background: var(--primary-soft); border-color: oklch(0.75 0.15 52 / 0.4); color: var(--primary); }
  .count { color: var(--faint); font-size: 0.82rem; margin-bottom: 10px; font-variant-numeric: tabular-nums; }

  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 270px), 1fr)); gap: 12px; transition: opacity 150ms; }
  .grid.dim { opacity: 0.5; pointer-events: none; }
  .tile { display: flex; flex-direction: column; gap: 10px; padding: 16px; border-radius: var(--r-lg); background: var(--surface); border: 1px solid var(--line-soft); box-shadow: var(--shadow); transition: border-color 150ms, translate 200ms var(--ease); }
  .tile:hover { border-color: var(--line); translate: 0 -1px; }
  .tile.skel { min-height: 190px; animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.5; } }
  .top { display: flex; gap: 12px; align-items: center; min-width: 0; }
  .ttl { display: grid; min-width: 0; }
  .ttl strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .ttl small { color: var(--faint); font-size: 0.8rem; }
  .desc { color: var(--muted); font-size: 0.87rem; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; min-height: 2.6em; }
  .stats { display: flex; gap: 14px; color: var(--faint); font-size: 0.8rem; font-variant-numeric: tabular-nums; }
  .stats span { display: inline-flex; align-items: center; gap: 5px; }
  .act { margin-top: auto; display: flex; }
  .act .btn { flex: 1; justify-content: center; text-decoration: none; }
  .btn.done { color: var(--ok); cursor: default; background: oklch(0.78 0.15 155 / 0.1); border-color: transparent; }
  img, .ph { width: 44px; height: 44px; border-radius: 12px; flex: none; background: var(--bg); object-fit: cover; }
  .ph { display: grid; place-items: center; color: var(--faint); }
  .ph.sm { width: 34px; height: 34px; border-radius: 10px; }

  .pager { display: flex; justify-content: center; align-items: center; gap: 4px; margin-top: 20px; flex-wrap: wrap; }
  .pager .num { font-variant-numeric: tabular-nums; background: none; border-color: transparent; }
  .pager .num:hover { background: var(--surface-2); }
  .pager .num[aria-current='page'] { background: var(--primary); color: var(--primary-ink); font-weight: 600; }
  .gap { color: var(--faint); padding: 0 4px; }

  .empty { display: grid; justify-items: center; gap: 10px; text-align: center; padding: 40px 16px; border: 1px dashed var(--line); border-radius: var(--r-lg); color: var(--muted); }
</style>

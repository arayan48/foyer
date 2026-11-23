<script lang="ts">
  import { Puzzle, Search, Download, Trash2, Sparkles, ChevronLeft, ChevronRight, ExternalLink, Heart, Clock, Check, Package, Eye, Plus, X } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import Confirm from './Confirm.svelte'
  import { SOFTWARE, call, bytes, software, upload, download, type Conn, type Server } from './api'
  import { buildPack, type PackItem } from './pack'

  let { conn, server, address }: { conn: Conn; server: Server; address: string } = $props()
  type Plugin = { file: string; enabled: boolean; size: number }
  type Hit = { id: string; slug?: string; title: string; description: string; author: string; downloads: number; follows: number; updated: string; icon: string | null; categories?: string[] }

  const sw = software(server)
  const folder = SOFTWARE[sw.id].addons ?? 'plugins'
  const word = folder === 'mods' ? 'mod' : 'plugin'
  const base = `/api/servers/${server.id}/plugins`
  const pages = [
    { id: 'installed', label: 'Installés', icon: Puzzle },
    { id: 'add', label: 'Ajouter', icon: Download },
    { id: 'shaders', label: 'Shaders', icon: Sparkles },
    { id: 'pack', label: 'Pack joueurs', icon: Package },
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
  let cs = $state(blank())

  // pack joueurs : gardé sur ce PC, par serveur
  const packKey = `foyer.pack.${server.id}`
  let pack = $state<PackItem[]>(JSON.parse(localStorage.getItem(packKey) ?? '[]'))
  $effect(() => localStorage.setItem(packKey, JSON.stringify(pack)))
  const inPack = (h: Hit) => pack.some((p) => p.id === h.id)
  const togglePack = (h: Hit, kind: PackItem['kind']) => (pack = inPack(h) ? pack.filter((p) => p.id !== h.id) : [...pack, { id: h.id, title: h.title, kind }])
  let packInfo = $state('')

  async function exportPack() {
    busy = 'pack'
    error = packInfo = ''
    try {
      const { bytes: zip, missing } = await buildPack($state.snapshot(pack), sw.mc, server.name, { name: server.name, ip: `${address}:${server.port}` })
      const file = `${server.name.replace(/[^\w-]+/g, '-')}.mrpack`
      const path = `/api/servers/${server.id}/files/content?path=${encodeURIComponent(file)}`
      await upload(conn, path, new File([zip], file))
      const saved = await download(conn, path, file)
      packInfo = (saved ? `Enregistré : ${saved}` : 'Pack téléchargé') + (missing.length ? `. Pas dispo pour ${sw.mc} : ${missing.join(', ')}` : '')
    } catch (e) { error = (e as Error).message } finally { busy = '' }
  }

  // aperçu d'un shader : captures publiées sur Modrinth
  let preview = $state<{ title: string; slug?: string; shots: { url: string; title?: string }[] } | null>(null)
  let dlg = $state<HTMLDialogElement>()
  async function openPreview(h: Hit) {
    preview = { title: h.title, slug: h.slug, shots: [] }
    dlg?.showModal()
    try {
      const p = await fetch(`https://api.modrinth.com/v2/project/${h.id}`).then((r) => r.json())
      if (preview?.title === h.title) preview.shots = p.gallery.map((g: any) => ({ url: g.url, title: g.title }))
    } catch { /* pas d'aperçu, le lien Modrinth reste */ }
  }

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
  const install = (h: Hit) => run(h.id, async () => { await call(conn, base, { method: 'POST', body: JSON.stringify({ project: h.id }) }); added = new Set(added).add(h.id) })

  async function searchPlugins() {
    ps.loading = true
    error = ''
    try {
      const qs = new URLSearchParams({ q: ps.q, sort: ps.sort, page: String(ps.page), cat: ps.cat })
      const r = await call<{ hits: Hit[]; total: number; per_page: number }>(conn, `${base}/search?${qs}`)
      ;[ps.hits, ps.total, ps.per] = [r.hits, r.total, r.per_page]
    } catch (e) { error = (e as Error).message } finally { ps.loading = false }
  }

  const searchShaders = () => searchModrinth(ss, [['project_type:shader']])
  const searchClient = () => searchModrinth(cs, [['project_type:mod'], ['categories:fabric'], [`versions:${sw.mc}`], ['client_side:required', 'client_side:optional']])

  async function searchModrinth(ss: ReturnType<typeof blank>, base: string[][]) {
    ss.loading = true
    error = ''
    try {
      const facets = [...base, ...(ss.cat ? [[`categories:${ss.cat}`]] : [])]
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
    if (page === 'pack' && !cs.hits && sw.mc) searchClient()
  }

  const name = (f: string) => f.replace(/\.jar(\.disabled)?$/, '')
  const short = (n: number) => (n >= 1e6 ? (n / 1e6).toFixed(1).replace('.', ',') + ' M' : n >= 1e3 ? Math.round(n / 1e3) + ' k' : String(n))
  const ago = (d: string) => {
    const days = Math.floor((Date.now() - new Date(d).getTime()) / 864e5)
    return days < 1 ? "aujourd'hui" : days < 30 ? `il y a ${days} j` : days < 365 ? `il y a ${Math.floor(days / 30)} mois` : `il y a ${Math.floor(days / 365)} an${days >= 730 ? 's' : ''}`
  }
  // le fichier ne garde pas l'id Modrinth : on compare titre et nom de fichier sans espaces ni tirets
  const norm = (t: string) => t.toLowerCase().replace(/[^a-z0-9]/g, '')
  let added = $state(new Set<string>())
  const installed = (h: Hit) => added.has(h.id) || !!list?.some((p) => norm(name(p.file)).startsWith(norm(h.title)))
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

{#snippet browser(s: typeof ps, go: () => void, cats: Record<string, string>, placeholder: string, mode: 'plugin' | 'shader' | 'client')}
  {@const shader = mode === 'shader'}
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
    <div class="grid catalog">{#each Array(8) as _}<div class="tile skel"></div>{/each}</div>
  {:else if s.hits?.length === 0}
    <div class="empty"><Search size={20} /><p>Rien trouvé. Essaie un autre mot ou une autre catégorie.</p></div>
  {:else if s.hits}
    <div class="grid catalog" class:dim={s.loading}>
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
            {#if mode !== 'plugin'}
              {#if shader}<button class="btn" onclick={() => openPreview(h)}><Eye size={14} /> Aperçu</button>{/if}
              <button class="btn" class:done={inPack(h)} aria-pressed={inPack(h)} onclick={() => togglePack(h, shader ? 'shader' : 'mod')}>
                {#if inPack(h)}<Check size={14} /> Dans le pack{:else}<Plus size={14} /> Au pack{/if}
              </button>
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

<SubPages {pages} bind:page label={folder === 'mods' ? 'Mods' : 'Plugins'} {onselect}>
  {#if error}<p class="err" role="alert">{error}</p>{/if}

  {#if page === 'installed'}
    <h2>{folder === 'mods' ? 'Mods' : 'Plugins'} installés</h2>
    <p class="lead">Dossier <code>{folder}</code> du serveur. Les changements comptent au prochain redémarrage.</p>
    {#if list === null}
      <p class="muted">Chargement…</p>
    {:else if list.length === 0}
      <div class="empty">
        <Puzzle size={22} />
        <p>Aucun {word} pour l'instant.</p>
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
    <h2>Ajouter un {word}</h2>
    <p class="lead">Catalogue Modrinth, filtré pour {SOFTWARE[sw.id].label}{sw.mc ? ` et Minecraft ${sw.mc}` : ''}.</p>
    {@render browser(ps, searchPlugins, PLUGIN_CATS, folder === 'mods' ? 'Lithium, Fabric API, Create…' : 'EssentialsX, LuckPerms, WorldEdit…', 'plugin')}

  {:else if page === 'shaders'}
    <h2>Shaders</h2>
    <p class="lead">Ils tournent sur le PC de chaque joueur. Ajoute-les au pack joueurs : Iris et Sodium sont inclus d'office.</p>
    {@render browser(ss, searchShaders, SHADER_CATS, 'Complementary, BSL, Sildur…', 'shader')}

  {:else if page === 'pack'}
    <h2>Pack joueurs</h2>
    {#if !sw.mc}
      <p class="lead">Choisis d'abord une version de Minecraft pour ce serveur.</p>
    {:else}
      <p class="lead">Un fichier à ouvrir avec Modrinth App ou Prism Launcher : Minecraft {sw.mc}, Fabric, les mods et le shader choisis, et ce serveur déjà dans la liste.</p>
      {#if pack.length}
        <div class="card">
          {#each pack as p (p.id)}
            <div class="row">
              <div class="who">
                <span class="ph sm">{#if p.kind === 'shader'}<Sparkles size={15} />{:else}<Puzzle size={15} />{/if}</span>
                <div><strong>{p.title}</strong><small>{p.kind === 'shader' ? 'Shader' : 'Mod joueur'}</small></div>
              </div>
              <div class="val acts">
                <button class="btn icon ghost" onclick={() => (pack = pack.filter((x) => x.id !== p.id))} aria-label="Retirer {p.title}"><X size={15} /></button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
      <div class="export">
        <button class="btn primary" disabled={!pack.length || !!busy} onclick={exportPack}><Download size={14} /> {busy === 'pack' ? 'Préparation…' : 'Télécharger le pack'}</button>
        {#if packInfo}<span class="muted">{packInfo}</span>{/if}
      </div>
      <h3>Ajouter des mods joueurs</h3>
      {@render browser(cs, searchClient, {}, 'Sodium, Xaero, JourneyMap…', 'client')}
    {/if}
  {/if}
</SubPages>

<dialog class="dialog preview" bind:this={dlg} onclick={(e) => e.target === dlg && dlg?.close()} aria-label="Aperçu de {preview?.title}">
  {#if preview}
    <div class="phead"><h2>{preview.title}</h2><button class="btn icon ghost" onclick={() => dlg?.close()} aria-label="Fermer"><X size={16} /></button></div>
    <div class="shots">
      {#each preview.shots as g}<img src={g.url} alt={g.title ?? ''} loading="lazy" />{:else}<p class="muted">Chargement des captures…</p>{/each}
    </div>
    <div class="foot"><a class="btn" href="https://modrinth.com/shader/{preview.slug}" target="_blank" rel="noreferrer"><ExternalLink size={14} /> Voir sur Modrinth</a></div>
  {/if}
</dialog>

<Confirm bind:open={askDelete} title="Supprimer {toDelete ? name(toDelete.file) : ''} ?" text="Le fichier est retiré du dossier {folder}. Ses réglages restent, tu pourras le réinstaller." onconfirm={() => toDelete && remove(toDelete)} />

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
  .cat[aria-pressed='true'] { background: var(--primary-soft); border-color: rgb(198 113 57 / 0.4); color: var(--primary); }
  .count { color: var(--faint); font-size: 0.82rem; margin-bottom: 10px; font-variant-numeric: tabular-nums; }

  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 260px), 1fr)); gap: 12px; transition: opacity 150ms; }
  .grid.dim { opacity: 0.5; pointer-events: none; }
  .tile { display: flex; flex-direction: column; gap: 10px; padding: 16px 2px; border-radius: 0; background: none; border: 0; box-shadow: none; transition: border-color 150ms, translate 200ms var(--ease); }
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
  .btn.done { color: var(--ok); cursor: default; background: rgb(149 166 118 / 0.12); border-color: transparent; }
  img, .ph { width: 44px; height: 44px; border-radius: 12px; flex: none; background: var(--bg); object-fit: cover; }
  .ph { display: grid; place-items: center; color: var(--faint); }
  .ph.sm { width: 34px; height: 34px; border-radius: 10px; }

  .pager { display: flex; justify-content: center; align-items: center; gap: 4px; margin-top: 20px; flex-wrap: wrap; }
  .pager .num { font-variant-numeric: tabular-nums; background: none; border-color: transparent; }
  .pager .num:hover { background: var(--surface-2); }
  .pager .num[aria-current='page'] { background: var(--primary); color: var(--primary-ink); font-weight: 600; }
  .gap { color: var(--faint); padding: 0 4px; }

  .act { gap: 6px; }
  .export { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; margin: 16px 0 32px; }
  h3 { font-size: 1rem; margin-bottom: 12px; }
  .preview { width: min(900px, 92vw); }
  .phead { display: flex; justify-content: space-between; align-items: center; padding: 16px 22px; }
  .shots { display: grid; gap: 10px; padding: 0 22px 18px; max-height: 70vh; overflow: auto; }
  .shots img { width: 100%; height: auto; border-radius: 10px; }
  .preview .foot { display: flex; justify-content: flex-end; padding: 14px 22px; border-top: 1px solid var(--line-soft); }
  .empty { display: grid; justify-items: center; gap: 10px; text-align: center; padding: 40px 16px; border: 1px dashed var(--line); border-radius: var(--r-lg); color: var(--muted); }
</style>

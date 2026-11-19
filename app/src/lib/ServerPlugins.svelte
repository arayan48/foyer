<script lang="ts">
  import { Puzzle, Search, Download, Trash2, Sparkles } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import { call, bytes, type Conn, type Server } from './api'

  let { conn, server }: { conn: Conn; server: Server } = $props()
  type Plugin = { file: string; enabled: boolean; size: number }
  type Hit = { id: string; title: string; description: string; author: string; downloads: number; icon: string | null }

  const base = `/api/servers/${server.id}/plugins`
  const pages = [
    { id: 'installed', label: 'Installés', icon: Puzzle },
    { id: 'add', label: 'Ajouter', icon: Download },
    { id: 'shaders', label: 'Shaders', icon: Sparkles },
  ]
  let page = $state('installed')
  let list = $state<Plugin[] | null>(null)
  let error = $state('')
  let q = $state('')
  let hits = $state<Hit[] | null>(null)
  let searching = $state(false)
  let busy = $state('')

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
  const remove = (p: Plugin) => confirm(`Supprimer ${p.file} ?`) && run(p.file, () => call(conn, `${base}/${encodeURIComponent(p.file)}`, { method: 'DELETE' }))
  const install = (h: Hit) => run(h.id, () => call(conn, base, { method: 'POST', body: JSON.stringify({ project: h.id }) }))

  async function search(e?: Event) {
    e?.preventDefault()
    searching = true
    error = ''
    try { hits = await call<Hit[]>(conn, `${base}/search?q=${encodeURIComponent(q)}`) } catch (e) { error = (e as Error).message } finally { searching = false }
  }
  const name = (f: string) => f.replace(/\.jar(\.disabled)?$/, '')
  const short = (n: number) => (n >= 1e6 ? (n / 1e6).toFixed(1).replace('.', ',') + ' M' : n >= 1e3 ? Math.round(n / 1e3) + ' k' : String(n))
</script>

<SubPages {pages} bind:page label="Plugins" onselect={() => page === 'add' && !hits && search()}>
  {#if error}<p class="err" role="alert">{error}</p>{/if}

  {#if page === 'installed'}
    <h2>Plugins installés</h2>
    <p class="lead">Dossier <code>plugins</code> du serveur. Les changements comptent au prochain redémarrage.</p>
    {#if list === null}
      <p class="muted">Chargement…</p>
    {:else if list.length === 0}
      <div class="card"><div class="row wide"><div><strong>Aucun plugin</strong><small>Va dans « Ajouter » pour en installer un.</small></div></div></div>
    {:else}
      <div class="card">
        {#each list as p (p.file)}
          <div class="row">
            <div><strong class:off={!p.enabled}>{name(p.file)}</strong><small>{p.enabled ? 'Actif' : 'Désactivé'} · {bytes(p.size)}</small></div>
            <div class="val acts">
              <input type="checkbox" class="switch" checked={p.enabled} disabled={busy === p.file} onchange={() => toggle(p)} aria-label="Activer {name(p.file)}" />
              <button class="btn icon danger" disabled={busy === p.file} onclick={() => remove(p)} aria-label="Supprimer {name(p.file)}"><Trash2 size={14} /></button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  {:else if page === 'add'}
    <h2>Ajouter un plugin</h2>
    <p class="lead">Catalogue Modrinth, filtré pour Paper{server.version ? ` et ${server.version.replace('Paper ', 'Minecraft ')}` : ''}.</p>
    <form class="bar" onsubmit={search}>
      <input class="input" type="search" placeholder="EssentialsX, LuckPerms, WorldEdit…" bind:value={q} aria-label="Rechercher un plugin" />
      <button class="btn primary" disabled={searching}><Search size={14} /> Rechercher</button>
    </form>
    {#if searching}
      <p class="muted">Recherche…</p>
    {:else if hits?.length === 0}
      <p class="muted">Rien trouvé pour cette version.</p>
    {:else if hits}
      <div class="card">
        {#each hits as h (h.id)}
          <div class="row hit">
            <div class="who">
              {#if h.icon}<img src={h.icon} alt="" width="36" height="36" loading="lazy" />{:else}<span class="ph"><Puzzle size={16} /></span>{/if}
              <div><strong>{h.title}</strong><small>{h.description}</small><small class="meta">{h.author} · {short(h.downloads)} téléchargements</small></div>
            </div>
            <div class="val">
              <button class="btn" disabled={!!busy} onclick={() => install(h)}>
                <Download size={14} /> {busy === h.id ? 'Installation…' : 'Installer'}
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  {:else if page === 'shaders'}
    <h2>Shaders et mods</h2>
    <p class="lead">Ce qui ne s'installe pas sur le serveur.</p>
    <div class="card">
      <div class="row wide"><div><strong>Shaders</strong><small>Ils tournent sur le PC de chaque joueur (Iris ou OptiFine), pas sur le serveur. Rien à faire ici.</small></div></div>
      <div class="row wide"><div><strong>Mods <span class="soon">Bientôt</span></strong><small>Il faut un serveur Fabric ou Forge à la place de Paper. Le choix arrivera dans Version › Type.</small></div></div>
    </div>
  {/if}
</SubPages>

<style>
  .err { color: var(--err); margin-bottom: 12px; }
  .muted { color: var(--muted); }
  .off { color: var(--muted); text-decoration: line-through; }
  .acts { display: flex; align-items: center; gap: 10px; justify-content: flex-end; }
  .bar { display: flex; gap: 8px; margin-bottom: 16px; }
  .bar .input { flex: 1; }
  .hit { grid-template-columns: 1fr auto; }
  .who { display: flex !important; gap: 12px; align-items: flex-start; }
  .who > div { display: grid; gap: 2px; min-width: 0; }
  .who small { color: var(--muted); }
  .meta { font-size: 12px; }
  img, .ph { width: 36px; height: 36px; border-radius: 8px; flex: none; background: var(--bg); }
  .ph { display: grid; place-items: center; color: var(--muted); }
  code { font-size: 0.92em; }
</style>

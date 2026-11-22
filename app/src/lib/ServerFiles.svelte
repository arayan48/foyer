<script lang="ts">
  import { Folder, File as FileIcon, FolderPlus, FilePlus, Upload, Download, Pencil, Trash2, ArrowLeft, ChevronRight } from 'lucide-svelte'
  import Confirm from './Confirm.svelte'
  import SaveBar from './SaveBar.svelte'
  import { call, callText, download, upload, size, when, type Conn, type Server } from './api'

  let { conn, server }: { conn: Conn; server: Server } = $props()

  type Entry = { name: string; dir: boolean; size: number; modified: number }
  const EDIT_MAX = 1024 ** 2
  const UPLOAD_MAX = 500 * 1024 ** 2

  let cwd = $state('')
  let list = $state<Entry[] | null>(null)
  let error = $state('')
  let info = $state('')
  let busy = $state('')
  // formulaire de nom : nouveau dossier, nouveau fichier ou renommage
  let naming = $state<{ kind: 'dir' | 'file' | 'rename'; from: string; value: string } | null>(null)
  let target = $state<Entry | null>(null)
  let askDelete = $state(false)
  let pending = $state<File[]>([])
  let askReplace = $state(false)
  let editor = $state<{ path: string; name: string; crlf: boolean; saved: string; text: string } | null>(null)
  let askLeave = $state(false)
  let picker = $state<HTMLInputElement>()!

  // svelte-ignore state_referenced_locally
  const base = `/api/servers/${server.id}/files`
  const join = (n: string) => (cwd ? `${cwd}/${n}` : n)
  const at = (p: string) => `${base}?path=${encodeURIComponent(p)}`
  const content = (p: string) => `${base}/content?path=${encodeURIComponent(p)}`
  const crumbs = $derived(cwd ? cwd.split('/').map((n, i, a) => ({ n, path: a.slice(0, i + 1).join('/') })) : [])
  const names = $derived(pending.filter((f) => list?.some((e) => e.name === f.name)).map((f) => f.name))

  let turn = 0
  async function open(path: string) {
    const t = ++turn
    naming = null
    error = info = ''
    try {
      const l = await call<Entry[]>(conn, at(path))
      if (t === turn) [cwd, list] = [path, l] // un clic plus récent a gagné
    } catch (e) {
      if (t === turn) {
        error = (e as Error).message
        list ??= []
      }
    }
  }
  open('')

  async function run(label: string, f: () => Promise<unknown>) {
    busy = label
    error = info = ''
    try {
      await f()
      list = await call<Entry[]>(conn, at(cwd))
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = ''
    }
  }

  async function openFile(e: Entry) {
    error = info = ''
    if (e.size > EDIT_MAX) return void (error = `« ${e.name} » est trop gros pour l'éditeur (1 Mo max). Télécharge-le.`)
    busy = 'Ouverture…'
    const path = join(e.name)
    try {
      const raw = await callText(conn, content(path))
      // octets illisibles : enregistrer abîmerait le fichier
      if (/[\u0000�]/.test(raw)) throw new Error(`« ${e.name} » n'est pas un fichier texte. Télécharge-le.`)
      const text = raw.replace(/\r\n/g, '\n')
      editor = { path, name: e.name, crlf: raw.includes('\r\n'), saved: text, text }
    } catch (x) {
      error = (x as Error).message
    } finally {
      busy = ''
    }
  }

  async function save() {
    const ed = editor
    if (!ed) return
    busy = 'Enregistrement…'
    error = ''
    const text = ed.text
    try {
      await callText(conn, content(ed.path), { method: 'PUT', body: ed.crlf ? text.replace(/\n/g, '\r\n') : text })
      ed.saved = text
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = ''
    }
  }

  function leave() {
    editor = null
    open(cwd)
  }
  const back = () => (editor && editor.text !== editor.saved ? (askLeave = true) : leave())

  function submitName(e: SubmitEvent) {
    e.preventDefault()
    const n = naming
    const value = n?.value.trim()
    if (!n || !value || value === n.from) return void (naming = null)
    if (n.kind !== 'rename' && list?.some((x) => x.name.toLowerCase() === value.toLowerCase())) return void (error = 'Ce nom existe déjà')
    run('En cours…', async () => {
      if (n.kind === 'dir') await call(conn, at(join(value)), { method: 'POST' })
      else if (n.kind === 'file') await callText(conn, content(join(value)), { method: 'PUT', body: '' })
      else await call(conn, at(join(n.from)), { method: 'PATCH', body: JSON.stringify({ to: join(value) }) })
      naming = null
    })
  }

  function pick() {
    pending = [...(picker.files ?? [])]
    picker.value = ''
    const big = pending.find((f) => f.size > UPLOAD_MAX)
    if (big) return void (error = `« ${big.name} » dépasse 500 Mo.`)
    if (names.length) askReplace = true
    else send()
  }
  const send = () =>
    run('Envoi…', async () => {
      for (const [i, f] of pending.entries()) {
        busy = `Envoi ${i + 1} / ${pending.length}…`
        await upload(conn, content(join(f.name)), f)
      }
    })

  const get = (e: Entry) =>
    run('Téléchargement…', async () => {
      const saved = await download(conn, content(join(e.name)), e.name)
      if (saved) info = `Enregistré : ${saved}`
    })
  const remove = () => run('Suppression…', () => call(conn, at(join(target!.name)), { method: 'DELETE' }))
  const focus = (el: HTMLInputElement) => (el.focus(), el.select())
</script>

{#if editor}
  <div class="bar">
    <button class="btn icon" onclick={back} aria-label="Retour aux fichiers"><ArrowLeft size={16} /></button>
    <h2 class="path">{editor.path}</h2>
  </div>
  <textarea class="input code" bind:value={editor.text} spellcheck="false" aria-label="Contenu de {editor.name}" onkeydown={(e) => { if ((e.ctrlKey || e.metaKey) && e.key === 's') { e.preventDefault(); if (!busy) save() } }}></textarea>
  <SaveBar dirty={editor.text !== editor.saved} busy={!!busy} {error} onsave={save} onreset={() => { if (editor) editor.text = editor.saved; error = '' }} />
{:else}
  <div class="bar">
    <nav class="crumbs" aria-label="Dossier">
      <button onclick={() => open('')} aria-current={cwd ? undefined : 'page'}>{server.name}</button>
      {#each crumbs as c (c.path)}
        <ChevronRight size={14} />
        <button onclick={() => open(c.path)} aria-current={c.path === cwd ? 'page' : undefined}>{c.n}</button>
      {/each}
    </nav>
    <button class="btn" disabled={!!busy || !list} onclick={() => (naming = { kind: 'dir', from: '', value: '' })}><FolderPlus size={15} /> Dossier</button>
    <button class="btn" disabled={!!busy || !list} onclick={() => (naming = { kind: 'file', from: '', value: '' })}><FilePlus size={15} /> Fichier</button>
    <button class="btn primary" disabled={!!busy || !list} onclick={() => picker.click()}><Upload size={15} /> Envoyer</button>
    <input bind:this={picker} type="file" multiple hidden onchange={pick} />
  </div>

  {#if error}<p class="err-text" role="alert">{error}</p>{/if}
  {#if busy || info}<p class="note" role="status">{busy || info}</p>{/if}

  {#if naming}
    <form class="name" onsubmit={submitName}>
      <input class="input" bind:value={naming.value} use:focus maxlength="120" aria-label={naming.kind === 'rename' ? `Nouveau nom de ${naming.from}` : naming.kind === 'dir' ? 'Nom du dossier' : 'Nom du fichier'} placeholder={naming.kind === 'dir' ? 'Nom du dossier' : 'Nom du fichier'} onkeydown={(e) => e.key === 'Escape' && (naming = null)} />
      <button type="button" class="btn" onclick={() => (naming = null)}>Annuler</button>
      <button class="btn primary" disabled={!!busy || !naming.value.trim()}>{naming.kind === 'rename' ? 'Renommer' : 'Créer'}</button>
    </form>
  {/if}

  {#if list === null}
    <p class="note">Chargement…</p>
  {:else if list.length === 0}
    <p class="note">Dossier vide.</p>
  {:else}
    <ul>
      {#each list as e (e.name)}
        <li>
          <button class="open" disabled={!!busy} onclick={() => (e.dir ? open(join(e.name)) : openFile(e))}>
            <span class="ic" class:dir={e.dir}>{#if e.dir}<Folder size={17} />{:else}<FileIcon size={17} />{/if}</span>
            <span class="nm">{e.name}</span>
          </button>
          <span class="meta sz">{e.dir ? '' : size(e.size)}</span>
          <span class="meta dt">{when(e.modified)}</span>
          <span class="acts">
            {#if !e.dir}<button class="btn icon ghost" disabled={!!busy} onclick={() => get(e)} title="Télécharger" aria-label="Télécharger {e.name}"><Download size={15} /></button>{/if}
            <button class="btn icon ghost" disabled={!!busy} onclick={() => (naming = { kind: 'rename', from: e.name, value: e.name })} title="Renommer" aria-label="Renommer {e.name}"><Pencil size={15} /></button>
            <button class="btn icon ghost danger" disabled={!!busy} onclick={() => { target = e; askDelete = true }} title="Supprimer" aria-label="Supprimer {e.name}"><Trash2 size={15} /></button>
          </span>
        </li>
      {/each}
    </ul>
  {/if}
{/if}

<Confirm bind:open={askDelete} title="Supprimer « {target?.name} » ?" text={target?.dir ? 'Le dossier et tout son contenu sont effacés, sans corbeille.' : 'Le fichier est effacé, sans corbeille.'} onconfirm={remove} />
<Confirm bind:open={askReplace} title={names.length > 1 ? `Remplacer ${names.length} fichiers ?` : `Remplacer « ${names[0]} » ?`} text="L'ancienne version est perdue." action="Remplacer" onconfirm={send} />
<Confirm bind:open={askLeave} title="Quitter sans enregistrer ?" text="Les modifications de ce fichier sont perdues." action="Quitter" onconfirm={leave} />

<style>
  .bar { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; margin-bottom: 14px; }
  .crumbs { display: flex; align-items: center; flex-wrap: wrap; gap: 2px; flex: 1; min-width: 0; color: var(--faint); }
  .crumbs button { border: 0; background: none; padding: 4px 6px; border-radius: 6px; color: var(--muted); cursor: pointer; }
  .crumbs button:hover { color: var(--ink); background: var(--surface); }
  .crumbs button[aria-current] { color: var(--ink); font-weight: 600; }
  .path { flex: 1; min-width: 0; overflow-wrap: anywhere; font-size: 1rem; }
  .note { color: var(--muted); margin-bottom: 10px; overflow-wrap: anywhere; }
  .err-text { margin-bottom: 10px; }
  .name { display: flex; gap: 8px; margin-bottom: 12px; }
  .name input { flex: 1; min-width: 0; }
  ul { list-style: none; margin: 0; padding: 0; }
  li { display: grid; grid-template-columns: minmax(0, 1fr) 80px 130px 120px; align-items: center; gap: 12px; padding: 2px 4px 2px 0; border-radius: var(--r-sm); transition: background 150ms var(--ease); }
  li:hover { background: var(--line-soft); }
  .open { display: flex; align-items: center; gap: 12px; min-width: 0; padding: 8px 10px; border: 0; border-radius: var(--r-sm); background: none; text-align: left; cursor: pointer; }
  .open:disabled { cursor: default; opacity: 0.6; }
  .ic { display: flex; color: var(--faint); }
  .ic.dir { color: var(--primary); }
  .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .meta { color: var(--muted); font-size: 0.85rem; font-variant-numeric: tabular-nums; text-align: right; white-space: nowrap; }
  .acts { display: flex; justify-content: flex-end; }
  .code { width: 100%; height: calc(100vh - 330px); min-height: 260px; padding: 14px; resize: vertical; color: var(--ink); font: 0.87rem/1.55 ui-monospace, 'Cascadia Mono', Consolas, Menlo, monospace; tab-size: 2; white-space: pre; overflow: auto; }
  @media (max-width: 820px) { li { grid-template-columns: minmax(0, 1fr) auto auto; } .dt { display: none; } }
</style>

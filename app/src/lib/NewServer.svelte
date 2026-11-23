<script lang="ts">
  import { ArrowLeft, Check, Search } from 'lucide-svelte'
  import { GAMES, SOFTWARE, call, cmpVersion, type Host, type Catalog, type Conn, type Game, type Server, type Software } from './api'

  // affiches officielles, © leurs éditeurs (mêmes fichiers que le site)
  import minecraft from '../assets/games/minecraft.webp'
  import fivem from '../assets/games/fivem.webp'
  import valheim from '../assets/games/valheim.webp'
  import terraria from '../assets/games/terraria.webp'
  const POSTERS: Record<Game, string> = { minecraft, fivem, valheim, terraria }

  let { conn, servers, oncreated, oncancel }: {
    conn: Conn
    servers: Server[]
    oncreated: (s: Server) => void
    oncancel: () => void
  } = $props()

  let game = $state<Game>('minecraft')
  let query = $state('')
  let cat = $state('') // vide = tous
  const games = Object.entries(GAMES) as [Game, (typeof GAMES)[Game]][]
  const cats = [...new Set(games.flatMap(([, g]) => g.cats))]
  // sans accents ni majuscules
  const fold = (t: string) => t.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase()
  let shown = $derived(
    games.filter(([, g]) => (!cat || g.cats.includes(cat)) && fold([g.label, g.hint, ...g.cats].join(' ')).includes(fold(query.trim()))),
  )
  let name = $state('')
  let ram = $state(4)
  let eula = $state(false)
  // un agent plus ancien installerait Vanilla sans rien dire
  let agent = '999'
  call<Host>(conn, '/api/system').then((h) => (agent = h.agent ?? '0')).catch(() => {})
  let busy = $state(false)
  let error = $state('')

  // Minecraft
  let sw = $state<Software>('vanilla')
  let version = $state('')
  let seed = $state('')
  let levelType = $state('default')
  let hardcore = $state(false)
  let onlineMode = $state(true)
  // modpack : .mrpack Modrinth choisi dans la liste, ou lien d'un server pack .zip (CurseForge…)
  type Pack = { title: string; url: string; icon?: string }
  let pack = $state<Pack | null>(null)
  let packQ = $state('')
  let packLink = $state('')
  let packHits = $state<{ id: string; title: string; author: string; icon: string | null }[]>([])
  let packError = $state('')
  let packTimer: ReturnType<typeof setTimeout>
  function packSearch() {
    clearTimeout(packTimer)
    packTimer = setTimeout(async () => {
      const q = packQ.trim()
      if (!q) return (packHits = [])
      try {
        const qs = new URLSearchParams({ query: q, limit: '6', facets: JSON.stringify([['project_type:modpack']]) })
        const r = await fetch(`https://api.modrinth.com/v2/search?${qs}`).then((r) => r.json())
        if (q === packQ.trim()) packHits = r.hits.map((h: any) => ({ id: h.project_id, title: h.title, author: h.author, icon: h.icon_url }))
      } catch { packError = 'Recherche Modrinth indisponible.' }
    }, 300)
  }
  async function pickPack(h: (typeof packHits)[number]) {
    packError = ''
    try {
      const qs = new URLSearchParams({ loaders: JSON.stringify(['neoforge', 'forge', 'fabric']) })
      const v = (await fetch(`https://api.modrinth.com/v2/project/${h.id}/version?${qs}`).then((r) => r.json()))[0]
      const f = v?.files.find((f: any) => f.primary) ?? v?.files[0]
      const loader = (['neoforge', 'forge', 'fabric'] as Software[]).find((l) => v.loaders.includes(l))
      if (!f || !loader) throw new Error()
      pack = { title: `${h.title} ${v.version_number}`, url: f.url, icon: h.icon ?? undefined }
      sw = loader
      version = v.game_versions[0]
      loadVersions()
    } catch { packError = "Ce modpack n'a pas de version Fabric, Forge ou NeoForge (Quilt n'est pas géré)." }
  }
  function useLink() {
    const u = packLink.trim()
    if (!u.startsWith('https://')) return (packError = 'Le lien doit commencer par https://')
    packError = ''
    pack = { title: decodeURIComponent(u.split('/').pop()?.split('?')[0] || u), url: u }
  }
  const LEVELS = { default: 'Normal', flat: 'Plat', largeBiomes: 'Grands biomes', amplified: 'Amplifié' }
  // FiveM
  let channel = $state('recommended')
  let txadmin = $state(true)
  let framework = $state<'vanilla' | 'esx' | 'qbcore'>('vanilla')
  let licenseKey = $state('')
  let maxPlayers = $state(48)
  let dbUrl = $state('mysql://root@localhost/fivem')

  let catalog = $state<Catalog | null>(null)
  let catalogError = $state('')
  call<Catalog>(conn, '/api/catalog').then((c) => (catalog = c)).catch((e) => (catalogError = (e as Error).message))

  // versions du logiciel choisi
  let mcVersions = $state<Catalog['minecraft'] | null>(null)
  let mcError = $state('')
  async function loadVersions() {
    const want = sw
    const keep = version
    mcVersions = null
    version = ''
    mcError = ''
    try {
      const l = await call<Catalog['minecraft']>(conn, `/api/catalog/${want}`)
      if (want !== sw) return // un autre choix entre-temps
      mcVersions = l
      version = (l.find((v) => v.id === keep) ?? l[0])?.id ?? ''
    } catch (e) {
      if (want === sw) mcError = (e as Error).message
    }
  }
  loadVersions()
  // nom facultatif : minecraft1, fivem2… premier numéro libre
  let autoName = $derived.by(() => {
    const taken = new Set(servers.map((s) => s.name.toLowerCase()))
    let n = 1
    while (taken.has(`${game}${n}`)) n++
    return `${game}${n}`
  })
  let keyOk = $derived(/^[A-Za-z0-9_]*$/.test(licenseKey.trim()))

  // premier port libre à partir du port par défaut du jeu
  let port = $derived.by(() => {
    const used = new Set(servers.flatMap((s) => [s.port, s.admin_port]))
    let p = GAMES[game].port
    while (used.has(p) || (game === 'fivem' && txadmin && used.has(p + 10000))) p++
    return p
  })

  // étapes affichées une par une ; « framework » seulement pour FiveM sans txAdmin
  let steps = $derived(
    game === 'fivem' && !txadmin
      ? ['Jeu', 'Version', 'Framework', 'Finaliser']
      : game === 'minecraft' ? ['Jeu', 'Version', 'Monde', 'Finaliser'] : ['Jeu', 'Version', 'Finaliser'],
  )
  let step = $state(0)
  let current = $derived(steps[Math.min(step, steps.length - 1)])
  let last = $derived(step >= steps.length - 1)
  let canNext = $derived(
    current === 'Jeu' ? shown.some(([k]) => k === game) : current === 'Version' ? game !== 'minecraft' || !!version : current !== 'Framework' || keyOk,
  )
  const fwLabel: Record<string, string> = { vanilla: 'Vanilla', esx: 'ESX Legacy', qbcore: 'QBCore' }

  function back() {
    if (step === 0) oncancel()
    else step--
  }

  async function submit(e: SubmitEvent) {
    e.preventDefault()
    if (!last) {
      step++
      return
    }
    name = name.trim() || autoName
    if (game === 'minecraft' && cmpVersion(agent, '0.4.18') < 0) {
      error = "Mets d'abord Foyer Serveur à jour : cette version ignore ces choix."
      return
    }
    busy = true
    error = ''
    try {
      const s = await call<Server>(conn, '/api/servers', {
        method: 'POST',
        body: JSON.stringify(
          game === 'minecraft'
            ? { name, game, port, ram_mb: ram * 1024, eula, software: sw, version, seed: seed.trim(), level_type: levelType, hardcore, online_mode: onlineMode, pack: pack?.url }
            : {
                name, game, port, ram_mb: ram * 1024, version: channel, txadmin,
                ...(txadmin ? {} : { framework, license_key: licenseKey.trim(), max_players: maxPlayers, db_url: framework === 'vanilla' ? undefined : dbUrl }),
              },
        ),
      })
      oncreated(s)
    } catch (err) {
      error = (err as Error).message
    } finally {
      busy = false
    }
  }
</script>

<header>
  <button class="btn icon" onclick={back} aria-label="Retour"><ArrowLeft size={16} /></button>
  <h1>Nouveau serveur</h1>
</header>

<div class="steps" class:wide={current === 'Jeu'}>
  <div class="steps-label"><span>{current}</span><span class="steps-count">{step + 1} / {steps.length}</span></div>
  <div class="track" role="progressbar" aria-label="Progression" aria-valuemin="1" aria-valuemax={steps.length} aria-valuenow={step + 1} aria-valuetext={`Étape ${step + 1} sur ${steps.length} : ${current}`}>
    <div class="fill" style:width={`${((step + 1) / steps.length) * 100}%`}></div>
  </div>
</div>

<form onsubmit={submit} class:wide={current === 'Jeu'}>
  {#if current === 'Jeu'}
    <div class="bar">
      <div class="cats" role="group" aria-label="Catégories">
        {#each ['', ...cats] as c}
          <button type="button" class="cat" aria-pressed={cat === c} onclick={() => (cat = c)}>{c || 'Tous'}</button>
        {/each}
      </div>
      <div class="search">
        <Search size={14} />
        <input class="input" type="search" placeholder="Rechercher" bind:value={query} aria-label="Rechercher un jeu" spellcheck="false" onkeydown={(e) => e.key === 'Enter' && e.preventDefault()} />
      </div>
    </div>
    {#if shown.length}
      <fieldset class="posters" aria-label="Jeu">
        {#each shown as [key, g] (key)}
          <label class="poster" class:on={game === key} class:off={g.soon}>
            <input type="radio" name="game" value={key} bind:group={game} disabled={g.soon} />
            <span class="art">
              <img src={POSTERS[key]} alt="" draggable="false" />
              {#if g.soon}<span class="tag">Bientôt</span>{:else if game === key}<span class="tick"><Check size={14} strokeWidth={3} /></span>{/if}
            </span>
            <strong>{g.label}</strong>
            <small>{g.hint}</small>
            <small class="min" title="Configuration minimale du serveur">{g.min}</small>
          </label>
        {/each}
      </fieldset>
    {:else}
      <div class="empty"><Search size={20} /><p>Aucun jeu trouvé.</p></div>
    {/if}
  {:else if current === 'Version'}
    {#if catalogError && game === 'fivem'}<p class="note">Liste des versions indisponible ({catalogError}). Les choix par défaut seront utilisés.</p>{/if}
    {#if game === 'minecraft'}
      <fieldset>
        <legend>Logiciel</legend>
        <div class="games">
          {#each Object.entries(SOFTWARE) as [k, s] (k)}
            <label class="game" class:on={sw === k}>
              <input type="radio" name="software" value={k} bind:group={sw} onchange={loadVersions} />
              <span><strong>{s.label}</strong><small>{s.hint}</small></span>
            </label>
          {/each}
        </div>
      </fieldset>
      <label class="field">
        <span>Version de Minecraft</span>
        <select class="input" bind:value={version} disabled={!mcVersions}>
          {#if !mcVersions}<option value="">{mcError ? 'Indisponible' : 'Chargement…'}</option>{/if}
          {#each mcVersions ?? [] as v (v.id)}<option value={v.id}>{v.id}</option>{/each}
        </select>
        {#if SOFTWARE[sw].addons}<small>Vérifie d'abord la version demandée par tes {SOFTWARE[sw].addons}.</small>{/if}
      </label>
      {#if mcError}<p class="err-text" role="alert">Liste des versions indisponible : {mcError} <button type="button" class="btn" onclick={loadVersions}>Réessayer</button></p>{/if}
      <fieldset>
        <legend>Modpack <em>facultatif</em></legend>
        {#if pack}
          <div class="pack">
            {#if pack.icon}<img src={pack.icon} alt="" width="32" height="32" />{/if}
            <strong>{pack.title}</strong>
            <button type="button" class="btn" onclick={() => (pack = null)}>Retirer</button>
          </div>
        {:else}
          <div class="search">
            <Search size={14} />
            <input class="input" type="search" placeholder="Chercher sur Modrinth" bind:value={packQ} oninput={packSearch} aria-label="Chercher un modpack" spellcheck="false" onkeydown={(e) => e.key === 'Enter' && e.preventDefault()} />
          </div>
          {#each packHits as h (h.id)}
            <button type="button" class="pack hit" onclick={() => pickPack(h)}>
              {#if h.icon}<img src={h.icon} alt="" width="32" height="32" />{/if}
              <span><strong>{h.title}</strong><small>par {h.author}</small></span>
            </button>
          {/each}
          <label class="field">
            <span>Ou lien d'un server pack .zip</span>
            <div class="row">
              <input class="input" type="url" placeholder="https://…/Server-Files.zip" bind:value={packLink} spellcheck="false" onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), useLink())} />
              <button type="button" class="btn" disabled={!packLink.trim()} onclick={useLink}>Utiliser</button>
            </div>
            <small>CurseForge (All The Mods…) : page du pack, onglet Files, « Server Pack », clic droit sur Download, copier le lien. Choisis ensuite le même logiciel et la même version que le pack.</small>
          </label>
        {/if}
        {#if packError}<p class="err-text" role="alert">{packError}</p>{/if}
      </fieldset>
    {:else}
      <label class="field">
        <span>Build FiveM</span>
        <select class="input" bind:value={channel}>
          {#each catalog?.fivem ?? [{ id: 'recommended', build: '' }, { id: 'optional', build: '' }, { id: 'latest', build: '' }] as c}
            <option value={c.id}>{({ recommended: 'Recommandé, le plus sûr', optional: 'Optionnel', latest: 'Dernier, les nouveautés' } as Record<string, string>)[c.id]}{c.build ? ` (build ${c.build})` : ''}</option>
          {/each}
        </select>
      </label>
      <fieldset>
        <legend>Gestion du serveur</legend>
        <div class="games">
          <label class="game" class:on={txadmin}>
            <input type="radio" name="tx" value={true} bind:group={txadmin} />
            <span><strong>Avec txAdmin</strong><small>Panneau web, recettes ESX/QBCore installées par txAdmin.</small></span>
          </label>
          <label class="game" class:on={!txadmin}>
            <input type="radio" name="tx" value={false} bind:group={txadmin} />
            <span><strong>Sans txAdmin</strong><small>Foyer prépare server.cfg et le framework.</small></span>
          </label>
        </div>
      </fieldset>
      {#if txadmin}
        <p class="note">Au premier démarrage, la console affiche un code PIN. Ouvre txAdmin dans le navigateur, puis choisis une recette (ESX, QBCore…) et colle ta clé de licence.</p>
      {/if}
    {/if}
  {:else if current === 'Monde'}
    <p class="note">À choisir maintenant : difficile à changer une fois le monde créé.</p>
    <label class="field">
      <span>Type de monde</span>
      <select class="input" bind:value={levelType}>
        {#each Object.entries(LEVELS) as [k, l] (k)}<option value={k}>{l}</option>{/each}
      </select>
    </label>
    <label class="field">
      <span>Graine <em>facultatif</em></span>
      <input class="input" bind:value={seed} maxlength="100" placeholder="Au hasard" spellcheck="false" autocomplete="off" />
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={hardcore} />
      <span>Hardcore : une seule vie</span>
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={onlineMode} />
      <span>Comptes Minecraft officiels seulement{#if !onlineMode}<small class="warn">N'importe qui pourra prendre le pseudo d'un autre. Changer plus tard vide les inventaires.</small>{/if}</span>
    </label>
  {:else if current === 'Framework'}
    <fieldset>
      <legend>Framework</legend>
      <div class="games three">
        {#each [['vanilla', 'Ressources de base, rien d’autre.'], ['esx', 'RP le plus répandu.'], ['qbcore', 'RP moderne, très modulaire.']] as [k, hint]}
          <label class="game" class:on={framework === k}>
            <input type="radio" name="fw" value={k} bind:group={framework} />
            <span><strong>{fwLabel[k]}</strong><small>{hint}</small></span>
          </label>
        {/each}
      </div>
    </fieldset>
    <label class="field">
      <span>Clé de licence Cfx.re</span>
      <input class="input" type="password" bind:value={licenseKey} placeholder="cfxk_…" autocomplete="off" spellcheck="false" aria-invalid={!keyOk} />
      {#if !keyOk}<small class="warn">Lettres, chiffres et _ seulement.</small>{:else}<small>Gratuite sur <a href="https://portal.cfx.re/servers/registration-keys" target="_blank" rel="noreferrer">portal.cfx.re</a>. Sans clé, le serveur ne démarre pas.</small>{/if}
    </label>
    <label class="field">
      <span>Joueurs maximum</span>
      <input class="input" type="number" min="1" max="2048" bind:value={maxPlayers} />
      <small>48 sans abonnement Cfx.re.</small>
    </label>
    {#if framework !== 'vanilla'}
      <label class="field">
        <span>Base de données MySQL / MariaDB</span>
        <input class="input" bind:value={dbUrl} spellcheck="false" />
        <small>Requise par {fwLabel[framework]}. Importe le fichier .sql indiqué dans la console avant le premier démarrage.</small>
      </label>
    {/if}
  {:else}
    <label class="field">
      <span>Nom <em>facultatif</em></span>
      <input class="input" bind:value={name} maxlength="60" placeholder={autoName} />
    </label>
    <label class="field">
      <span>Mémoire réservée : {ram} Go</span>
      <input type="range" min="1" max="12" bind:value={ram} />
      {#if game === 'minecraft' && SOFTWARE[sw].addons === 'mods' && ram < 4}<small class="warn">Peu pour des mods : 4 Go ou plus.</small>{/if}
    </label>
    <dl class="recap">
      <dt>Jeu</dt><dd>{GAMES[game].label}</dd>
      {#if game === 'minecraft'}
        <dt>Version</dt><dd>{SOFTWARE[sw].label} {version}</dd>
        <dt>Monde</dt><dd>{LEVELS[levelType as keyof typeof LEVELS]}{seed.trim() ? `, graine ${seed.trim()}` : ''}{hardcore ? ', hardcore' : ''}</dd>
        {#if !onlineMode}<dt>Comptes</dt><dd>Non officiels acceptés</dd>{/if}
      {:else}
        <dt>Build</dt><dd>{channel}</dd>
        <dt>Mode</dt><dd>{txadmin ? 'txAdmin' : fwLabel[framework]}</dd>
      {/if}
      <dt>Port</dt><dd>{port}{#if game === 'fivem' && txadmin}, txAdmin {port + 10000}{/if} (réseau local)</dd>
    </dl>
    {#if game === 'minecraft'}
      <label class="check">
        <input type="checkbox" bind:checked={eula} required />
        <span>J'accepte le <a href="https://aka.ms/MinecraftEULA" target="_blank" rel="noreferrer">contrat de licence (EULA) de Minecraft</a>.</span>
      </label>
    {/if}
  {/if}

  {#if error}<p class="err-text" role="alert">{error}</p>{/if}
  <div class="foot">
    <button type="button" class="btn" onclick={back}>{step === 0 ? 'Annuler' : 'Précédent'}</button>
    {#if last}
      <button class="btn primary" disabled={busy || !keyOk}>{busy ? 'Création…' : 'Créer et installer'}</button>
    {:else}
      <button class="btn primary" disabled={!canNext}>Suivant</button>
    {/if}
  </div>
</form>
<style>
  header { display: flex; align-items: center; gap: 12px; margin-bottom: 24px; }
  form { display: grid; gap: 20px; max-width: 560px; }
  fieldset { border: 0; padding: 0; margin: 0; display: grid; gap: 8px; }
  legend { color: var(--muted); font-size: 0.89rem; margin-bottom: 6px; }
  .games { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 8px; }
  .game {
    display: flex; align-items: center; gap: 12px; padding: 8px 4px; cursor: pointer;
    background: none; border: 0; opacity: 0.6; transition: opacity 150ms var(--ease);
  }
  .game:hover { opacity: 0.85; }
  .game.on { opacity: 1; }
  .game.on strong { color: var(--primary); }
  .game input { position: absolute; opacity: 0; pointer-events: none; }
  .game:has(input:focus-visible) { outline: 2px solid var(--primary); outline-offset: 2px; border-radius: var(--r); }
  .game > span { display: grid; }
  .game small { color: var(--muted); font-size: 0.85rem; }
  .steps { margin: 0 0 24px; display: grid; gap: 8px; max-width: 560px; }
  .steps-label { display: flex; justify-content: space-between; font-size: 0.85rem; color: var(--ink); }
  .steps-count { color: var(--muted); font-variant-numeric: tabular-nums; }
  .track { height: 6px; border-radius: 999px; background: var(--line); overflow: hidden; }
  .fill { height: 100%; border-radius: inherit; background: var(--primary); transition: width 0.35s cubic-bezier(0.2, 0.8, 0.2, 1); }
  @media (prefers-reduced-motion: reduce) { .fill { transition: none; } }
  .wide { max-width: 860px; }
  .bar { display: flex; flex-wrap: wrap; gap: 10px 16px; align-items: center; justify-content: space-between; }
  .cats { display: flex; gap: 6px; flex-wrap: wrap; }
  .cat { height: 1.9rem; padding: 0 0.8rem; border-radius: 999px; border: 1px solid var(--line-soft); background: none; color: var(--muted); cursor: pointer; font-size: 0.85rem; transition: background 120ms, color 120ms, border-color 120ms; }
  .cat:hover { color: var(--ink); background: var(--surface); }
  .cat[aria-pressed='true'] { background: var(--primary-soft); border-color: color-mix(in srgb, var(--primary) 40%, transparent); color: var(--primary); }
  .search { position: relative; width: 180px; }
  .search :global(svg) { position: absolute; left: 11px; top: 50%; translate: 0 -50%; color: var(--faint); pointer-events: none; }
  .search .input { width: 100%; height: 2rem; padding-left: 32px; font-size: 0.89rem; }
  .posters { grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 20px; }
  .poster { position: relative; display: flex; flex-direction: column; cursor: pointer; }
  .poster input { position: absolute; opacity: 0; pointer-events: none; }
  .art { position: relative; aspect-ratio: 2 / 3; margin-bottom: 10px; border-radius: var(--r); overflow: hidden; outline: 2px solid transparent; outline-offset: 3px; transition: outline-color 150ms var(--ease); }
  .art img { display: block; width: 100%; height: 100%; object-fit: cover; transition: scale 250ms var(--ease), filter 150ms; }
  .poster:not(.off):hover img { scale: 1.04; }
  .poster:not(.off):hover .art { outline-color: var(--line); }
  .poster.on .art, .poster:has(input:focus-visible) .art { outline-color: var(--primary); }
  .poster:has(input:focus-visible) .art { outline-width: 3px; }
  .poster.off { cursor: not-allowed; }
  .poster.off img { filter: grayscale(0.75) brightness(0.6); }
  .poster.off strong { color: var(--muted); }
  .poster.on strong { color: var(--primary); }
  .poster small { color: var(--muted); font-size: 0.85rem; line-height: 1.4; }
  .poster .min { margin-top: 4px; font-size: 0.78rem; font-variant-numeric: tabular-nums; }
  .tag { position: absolute; top: 10px; left: 10px; padding: 2px 9px; border-radius: 999px; background: rgb(26 24 23 / 0.85); color: #f5ead8; font-size: 0.75rem; font-weight: 500; }
  .tick { position: absolute; bottom: 10px; right: 10px; display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; background: var(--primary); color: var(--primary-ink); }
  .empty { display: grid; justify-items: center; gap: 10px; padding: 40px 16px; border: 1px dashed var(--line); border-radius: var(--r-lg); color: var(--muted); }
  .recap { display: grid; grid-template-columns: max-content 1fr; gap: 6px 16px; margin: 0; padding: 12px 2px; border: 0; border-radius: 0; background: none; }
  .recap dt { color: var(--muted); }
  .recap dd { margin: 0; }
  .three { grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); }
  .field em { font-style: normal; color: var(--faint); font-size: 0.85em; margin-left: 4px; }
  .field small { color: var(--muted); font-size: 0.85rem; }
  .field small.warn { color: var(--warn, oklch(0.75 0.15 70)); }
  input[type='range'] { accent-color: var(--primary); }
  .check { display: flex; gap: 10px; align-items: flex-start; }
  .check small { display: block; font-size: 0.85rem; color: var(--warn, oklch(0.75 0.15 70)); }
  .check input { accent-color: var(--primary); margin-top: 4px; }
  a { color: var(--primary); }
  .note { color: var(--muted); max-width: 65ch; }
  .note strong { color: var(--ink); font-variant-numeric: tabular-nums; }
  .foot { display: flex; gap: 8px; justify-content: flex-end; }
  .pack { display: flex; align-items: center; gap: 10px; }
  .pack img { border-radius: 6px; }
  .pack span { display: flex; flex-direction: column; }
  .pack.hit { width: 100%; padding: 6px 8px; border: 0; background: none; border-radius: 8px; text-align: left; color: inherit; font: inherit; cursor: pointer; }
  .pack.hit:hover, .pack.hit:focus-visible { background: color-mix(in srgb, var(--primary) 10%, transparent); }
  .row { display: flex; gap: 8px; }
  .row .input { flex: 1; }
</style>

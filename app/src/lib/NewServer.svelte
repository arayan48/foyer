<script lang="ts">
  import { ArrowLeft, Check, Search } from 'lucide-svelte'
  import { GAMES, call, cmpVersion, type Catalog, type Conn, type Game, type Server } from './api'

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
  let busy = $state(false)
  let error = $state('')

  // Minecraft
  let version = $state('') // vide = dernière stable
  let java = $state(0) // 0 = conseillé
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

  // de la plus récente à la plus ancienne (l'agent les renvoie dans le désordre)
  let mcVersions = $derived([...(catalog?.minecraft ?? [])].sort((a, b) => cmpVersion(b.id, a.id)))
  let recommendedJava = $derived(mcVersions.find((v) => v.id === (version || mcVersions[0]?.id))?.java ?? 25)
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
      : ['Jeu', 'Version', 'Finaliser'],
  )
  let step = $state(0)
  let current = $derived(steps[Math.min(step, steps.length - 1)])
  let last = $derived(step >= steps.length - 1)
  let canNext = $derived(
    current === 'Jeu' ? shown.some(([k]) => k === game) : current !== 'Framework' || keyOk,
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
    busy = true
    error = ''
    try {
      const s = await call<Server>(conn, '/api/servers', {
        method: 'POST',
        body: JSON.stringify(
          game === 'minecraft'
            ? { name, game, port, ram_mb: ram * 1024, eula, version: version || undefined, java: java || undefined }
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
    {#if catalogError}<p class="note">Liste des versions indisponible ({catalogError}). Les choix par défaut seront utilisés.</p>{/if}
    {#if game === 'minecraft'}
      <label class="field">
        <span>Version de Minecraft</span>
        <select class="input" bind:value={version} disabled={!catalog}>
          <option value="">La plus récente{mcVersions[0] ? ` (${mcVersions[0].id})` : ''}</option>
          {#each mcVersions as v}<option value={v.id}>{v.id}</option>{/each}
        </select>
        <small>Les joueurs doivent avoir la même version. Plugins et mods : onglet Version du serveur, après l'installation.</small>
      </label>
      <label class="field">
        <span>Java</span>
        <select class="input" bind:value={java}>
          <option value={0}>Automatique (Java {recommendedJava})</option>
          {#each (catalog?.java ?? [8, 11, 17, 21, 25]).filter((j) => j !== recommendedJava) as j}<option value={j}>Java {j}</option>{/each}
        </select>
        {#if java && java < recommendedJava}<small class="warn">Trop ancien pour cette version : le serveur risque de ne pas démarrer.</small>{:else}<small>Téléchargé une fois, partagé entre serveurs.</small>{/if}
      </label>
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
    </label>
    <dl class="recap">
      <dt>Jeu</dt><dd>{GAMES[game].label}</dd>
      {#if game === 'minecraft'}
        <dt>Version</dt><dd>{version || mcVersions[0]?.id || 'La plus récente'}, Java {java || recommendedJava}</dd>
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
  .check input { accent-color: var(--primary); margin-top: 4px; }
  a { color: var(--primary); }
  .note { color: var(--muted); max-width: 65ch; }
  .note strong { color: var(--ink); font-variant-numeric: tabular-nums; }
  .foot { display: flex; gap: 8px; justify-content: flex-end; }
</style>

<script lang="ts">
  import { ArrowLeft } from 'lucide-svelte'
  import GameMark from './GameMark.svelte'
  import { GAMES, call, type Catalog, type Conn, type Game, type Server } from './api'

  let { conn, servers, oncreated, oncancel }: {
    conn: Conn
    servers: Server[]
    oncreated: (s: Server) => void
    oncancel: () => void
  } = $props()

  let game = $state<Game>('minecraft')
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

  let recommendedJava = $derived(catalog?.minecraft.find((v) => v.id === (version || catalog?.minecraft[0]?.id))?.java ?? 25)
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
  let canNext = $derived(current !== 'Framework' || keyOk)
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

<ol class="steps" aria-label="Étapes">
  {#each steps as label, i}
    <li class:done={i < step} class:on={i === step} aria-current={i === step ? 'step' : undefined}>
      <span class="dot">{i + 1}</span>{label}
    </li>
  {/each}
</ol>

<form onsubmit={submit}>
  {#if current === 'Jeu'}
    <fieldset>
      <legend>Quel jeu ?</legend>
      <div class="games">
        {#each Object.entries(GAMES) as [key, g]}
          <label class="game" class:on={game === key}>
            <input type="radio" name="game" value={key} bind:group={game} />
            <GameMark game={key as Game} size={44} />
            <span><strong>{g.label}</strong><small>{g.hint}</small></span>
          </label>
        {/each}
      </div>
    </fieldset>
  {:else if current === 'Version'}
    {#if catalogError}<p class="note">Liste des versions indisponible ({catalogError}). Les choix par défaut seront utilisés.</p>{/if}
    {#if game === 'minecraft'}
      <label class="field">
        <span>Version de Minecraft</span>
        <select class="input" bind:value={version} disabled={!catalog}>
          <option value="">Dernière stable{catalog ? ` (${catalog.minecraft[0]?.id})` : ''}</option>
          {#each catalog?.minecraft ?? [] as v}<option value={v.id}>{v.id}</option>{/each}
        </select>
        <small>Les joueurs doivent avoir la même version. Serveur Paper (plugins).</small>
      </label>
      <label class="field">
        <span>Java</span>
        <select class="input" bind:value={java}>
          <option value={0}>Conseillé (Java {recommendedJava})</option>
          {#each catalog?.java ?? [8, 11, 17, 21, 25] as j}<option value={j}>Java {j}{j === recommendedJava ? ' (conseillé)' : ''}</option>{/each}
        </select>
        {#if java && java < recommendedJava}<small class="warn">Trop ancien pour cette version : le serveur risque de ne pas démarrer.</small>{:else}<small>Téléchargé une fois, partagé entre serveurs.</small>{/if}
      </label>
    {:else}
      <label class="field">
        <span>Build FiveM</span>
        <select class="input" bind:value={channel}>
          {#each catalog?.fivem ?? [{ id: 'recommended', build: '' }, { id: 'optional', build: '' }, { id: 'latest', build: '' }] as c}
            <option value={c.id}>{({ recommended: 'Recommended, le plus sûr', optional: 'Optional', latest: 'Latest, les nouveautés' } as Record<string, string>)[c.id]}{c.build ? ` (build ${c.build})` : ''}</option>
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
      <span>Nom</span>
      <input class="input" bind:value={name} maxlength="60" required placeholder={game === 'minecraft' ? 'Survie entre potes' : 'RP de la ville'} />
    </label>
    <label class="field">
      <span>Mémoire réservée : {ram} Go</span>
      <input type="range" min="1" max="12" bind:value={ram} />
    </label>
    <dl class="recap">
      <dt>Jeu</dt><dd>{GAMES[game].label}</dd>
      {#if game === 'minecraft'}
        <dt>Version</dt><dd>{version || 'Dernière stable'}, Java {java || recommendedJava}</dd>
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
      <button class="btn primary" disabled={busy || !name.trim() || !keyOk}>{busy ? 'Création…' : 'Créer et installer'}</button>
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
    display: flex; align-items: center; gap: 12px; padding: 12px; border-radius: var(--r); cursor: pointer;
    background: var(--surface); border: 1px solid var(--line); transition: border-color 150ms var(--ease), background 150ms;
  }
  .game:hover { background: var(--surface-2); }
  .game.on { border-color: var(--primary); background: oklch(0.72 0.18 357 / 0.08); }
  .game input { position: absolute; opacity: 0; pointer-events: none; }
  .game:has(input:focus-visible) { outline: 2px solid var(--primary); outline-offset: 2px; }
  .game span { display: grid; }
  .game small { color: var(--muted); font-size: 0.85rem; }
  .steps { list-style: none; display: flex; gap: 16px; padding: 0; margin: 0 0 24px; color: var(--muted); font-size: 0.89rem; flex-wrap: wrap; }
  .steps li { display: flex; align-items: center; gap: 8px; }
  .steps li.on { color: var(--ink); }
  .dot { display: grid; place-items: center; width: 22px; height: 22px; border-radius: 50%; border: 1px solid var(--line); font-variant-numeric: tabular-nums; font-size: 0.8rem; }
  .steps li.on .dot { border-color: var(--primary); color: var(--primary); }
  .steps li.done .dot { background: var(--primary); border-color: var(--primary); color: var(--bg, #000); }
  .recap { display: grid; grid-template-columns: max-content 1fr; gap: 6px 16px; margin: 0; padding: 12px; border: 1px solid var(--line); border-radius: var(--r); background: var(--surface); }
  .recap dt { color: var(--muted); }
  .recap dd { margin: 0; }
  .three { grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); }
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

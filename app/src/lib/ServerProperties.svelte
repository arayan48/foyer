<script module lang="ts">
  import { SvelteSet } from 'svelte/reactivity'
  // serveurs à redémarrer, gardés quand on change d'onglet
  const pending = new SvelteSet<string>()
</script>

<script lang="ts">
  import { untrack } from 'svelte'
  import { Earth, Gauge, RotateCw, Swords, Users } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import SaveBar from './SaveBar.svelte'
  import Confirm from './Confirm.svelte'
  import { call, cmpVersion, software, type Conn, type Server } from './api'

  let { conn, server }: { conn: Conn; server: Server } = $props()

  const pages = [
    { id: 'game', label: 'Partie', icon: Swords },
    { id: 'world', label: 'Monde', icon: Earth },
    { id: 'access', label: 'Accès', icon: Users },
    { id: 'perf', label: 'Performances', icon: Gauge },
  ]
  let page = $state('game')

  // valeurs par défaut de Minecraft quand la clé manque dans le fichier
  const DEF = {
    motd: 'A Minecraft Server', 'level-seed': '', 'level-type': 'default', 'spawn-protection': '16',
    gamemode: 'survival', 'force-gamemode': 'false', difficulty: 'easy', hardcore: 'false', pvp: 'true', 'enable-command-block': 'false', 'allow-flight': 'false',
    'max-players': '20', 'white-list': 'false', 'online-mode': 'true', 'player-idle-timeout': '0',
    'view-distance': '10', 'simulation-distance': '10', 'entity-broadcast-range-percentage': '100',
  }
  type Props = typeof DEF
  type Key = keyof Props
  let saved = $state<Props>({ ...DEF })
  let f = $state<Props>({ ...DEF })
  let ready = $state(false)
  let hasWorld = $state(false)
  let busy = $state(false)
  let error = $state('')
  const restart = $derived(pending.has(server.id))
  let ask = $state(false)
  const dirty = $derived(JSON.stringify(f) !== JSON.stringify(saved))
  const url = `/api/servers/${server.id}/properties`

  const mc = software(server).mc
  const since = (v: string) => !mc || cmpVersion(mc, v) >= 0
  // devenus des règles de jeu en 1.21.9 : l'agent les change par la console
  const oldRules = !since('1.21.9')
  const RULES: Partial<Record<Key, string>> = oldRules ? {} : { pvp: 'pvp', 'enable-command-block': 'command-blocks' }
  let hasRules = $state(false)

  // nombres avant 1.14, noms ensuite
  const NAMES: Partial<Record<Key, string[]>> = {
    gamemode: ['survival', 'creative', 'adventure', 'spectator'],
    difficulty: ['peaceful', 'easy', 'normal', 'hard'],
  }
  const LEVELS: Record<string, string> = { default: 'Normal', flat: 'Plat', largeBiomes: 'Grands biomes', amplified: 'Amplifié' }
  function read(k: Key, v: string) {
    if (NAMES[k]) return NAMES[k][Number(v)] ?? v
    if (k !== 'level-type') return v
    const id = v.replace(/^minecraft:/, '').replace('large_biomes', 'largeBiomes')
    if (id === 'normal') return 'default'
    return Object.keys(LEVELS).find((l) => l.toLowerCase() === id.toLowerCase()) ?? v
  }
  // depuis 1.19, Minecraft ne lit plus que les identifiants
  const IDS: Record<string, string> = { default: 'minecraft:normal', flat: 'minecraft:flat', largeBiomes: 'minecraft:large_biomes', amplified: 'minecraft:amplified' }
  function write(k: Key, v: string) {
    if (k === 'level-type' && since('1.19')) return IDS[v] ?? v
    const n = NAMES[k]?.indexOf(v) ?? -1
    return n >= 0 && !since('1.14') ? String(n) : v
  }

  let worldDir = ''
  async function findWorld() {
    const l = await call<{ name: string; dir: boolean }[]>(conn, `/api/servers/${server.id}/files?path=`)
    hasWorld = l.some((x) => x.dir && x.name === worldDir)
  }

  async function load() {
    error = ''
    try {
      const p = await call<Record<string, string>>(conn, url)
      const v = { ...DEF }
      for (const k of Object.keys(DEF) as Key[]) if (p[k] !== undefined) v[k] = read(k, p[k])
      if (!oldRules) {
        // un agent plus ancien n'a pas cette route
        const r = await call<Record<string, boolean>>(conn, `/api/servers/${server.id}/rules`).catch(() => null)
        hasRules = !!r
        for (const [k, name] of Object.entries(RULES)) if (r) v[k as Key] = String(r[name])
      }
      worldDir = p['level-name'] || 'world'
      await findWorld()
      saved = v
      f = { ...v }
      ready = true
    } catch (e) {
      error = (e as Error).message
    }
  }
  load()

  // case à cocher ↔ "true"/"false" du fichier
  const bool = (k: Key) => ({ get: () => f[k] === 'true', set: (v: boolean) => (f[k] = String(v)) })
  const pvp = bool('pvp'), hardcore = bool('hardcore'), white = bool('white-list'), online = bool('online-mode')
  const force = bool('force-gamemode'), blocks = bool('enable-command-block'), flight = bool('allow-flight')

  // le redémarrage demandé est fait dès que le serveur quitte l'état « en ligne »
  $effect(() => {
    // le monde naît au premier démarrage
    if (server.status && ready) untrack(() => findWorld().catch(() => {}))
    if (server.status !== 'running') untrack(() => pending.delete(server.id))
  })

  // réglages rapides : affichage, simulation, entités
  const PRESETS: [string, string, string, string][] = [
    ['Léger', '6', '4', '70'],
    ['Équilibré', '10', '8', '100'],
    ['Lointain', '16', '10', '100'],
  ]
  const simOn = since('1.18'), entOn = since('1.16')
  const isPreset = ([, v, sim, ent]: (typeof PRESETS)[number]) =>
    f['view-distance'] === v && (!simOn || f['simulation-distance'] === sim) && (!entOn || f['entity-broadcast-range-percentage'] === ent)
  function preset([, v, sim, ent]: (typeof PRESETS)[number]) {
    f['view-distance'] = v
    if (simOn) f['simulation-distance'] = sim
    if (entOn) f['entity-broadcast-range-percentage'] = ent
  }
  const view = $derived(Number(f['view-distance']))
  const chunks = (n: number) => `${n} chunks`

  function save() {
    error = ''
    if (hasWorld && f['online-mode'] !== saved['online-mode']) ask = true
    else send()
  }

  async function send() {
    busy = true
    error = ''
    const next = { ...f }
    try {
      const body: Record<string, string> = {}
      for (const k of Object.keys(next) as Key[]) if (next[k] !== saved[k] && !RULES[k]) body[k] = write(k, next[k].trim().replace(/\r/g, ''))
      // sans enforce-whitelist, les joueurs déjà connectés restent quand on active la liste
      if (body['white-list']) body['enforce-whitelist'] = body['white-list']
      for (const [k, name] of Object.entries(RULES) as [Key, string][]) {
        if (next[k] === saved[k]) continue
        await call(conn, `/api/servers/${server.id}/rules/${name}/${next[k]}`, { method: 'POST' })
        saved[k] = next[k]
      }
      const file = Object.keys(body).length > 0
      if (file) await call(conn, url, { method: 'PUT', body: JSON.stringify(body) })
      saved = next
      if (file && server.status === 'running') pending.add(server.id)
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }

  async function doRestart() {
    busy = true
    error = ''
    try {
      await call(conn, `/api/servers/${server.id}/restart`, { method: 'POST' })
      pending.delete(server.id)
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }
</script>

{#snippet slider(k: Key, label: string, min: number, max: number, unit: (n: number) => string, hint = '', step = 1)}
  {@const n = Number(f[k])}
  <label class="row"><div><strong>{label}</strong>{#if hint}<small>{hint}</small>{/if}</div>
    <div class="slide"><input type="range" {min} max={Math.max(max, Number(saved[k]) || 0)} {step} value={n} oninput={(e) => (f[k] = e.currentTarget.value)} aria-valuetext={unit(n)} /><output>{unit(n)}</output></div>
  </label>
{/snippet}
{#snippet pick(k: Key, label: string, options: [string, string][])}
  <label class="row"><div><strong>{label}</strong></div><select class="input" bind:value={f[k]}>
    {#each options as [v, l] (v)}<option value={v}>{l}</option>{/each}
    {#if !options.some(([v]) => v === saved[k])}<option value={saved[k]}>{saved[k]}</option>{/if}
  </select></label>
{/snippet}
{#snippet toggle(b: { get: () => boolean; set: (v: boolean) => void }, label: string, hint = '', disabled = false)}
  <label class="row"><div><strong>{label}</strong>{#if hint}<small>{hint}</small>{/if}</div><input type="checkbox" class="switch" {disabled} bind:checked={b.get, b.set} /></label>
{/snippet}

<SubPages {pages} bind:page label="Propriétés Minecraft">
  {#if restart && !dirty}
    <p class="restart" role="status">Redémarre le serveur pour appliquer. <button class="btn" disabled={busy} onclick={doRestart}><RotateCw size={14} /> Redémarrer</button></p>
  {/if}
  <fieldset disabled={!ready}>
    {#if page === 'game'}
      <h2>Partie</h2>
      <div class="card">
        <label class="row"><div><strong>Mode de jeu</strong></div><select class="input" bind:value={f.gamemode}><option value="survival">Survie</option><option value="creative">Créatif</option><option value="adventure">Aventure</option><option value="spectator">Spectateur</option></select></label>
        {@render toggle(force, 'Imposer le mode de jeu', 'Remis à chaque connexion.')}
        <label class="row"><div><strong>Difficulté</strong>{#if f.hardcore === 'true'}<small>Difficile imposé par le hardcore.</small>{/if}</div><select class="input" bind:value={f.difficulty} disabled={f.hardcore === 'true'}><option value="peaceful">Paisible</option><option value="easy">Facile</option><option value="normal">Normale</option><option value="hard">Difficile</option></select></label>
        {#if oldRules || hasRules}
          {@render toggle(pvp, 'Combat entre joueurs')}
          {@render toggle(blocks, 'Blocs de commande')}
        {/if}
        {@render toggle(flight, 'Autoriser le vol', "À activer si un mod fait voler, sinon le joueur est expulsé.")}
      </div>

    {:else if page === 'world'}
      <h2>Monde</h2>
      <div class="card">
        <label class="row"><div><strong>Type de monde</strong></div><select class="input" bind:value={f['level-type']} disabled={hasWorld}>
          {#each Object.entries(LEVELS) as [k, l] (k)}<option value={k}>{l}</option>{/each}
          {#if !(f['level-type'] in LEVELS)}<option value={f['level-type']}>{f['level-type']}</option>{/if}
        </select></label>
        <label class="row"><div><strong>Graine</strong></div><input class="input" bind:value={f['level-seed']} placeholder="Au hasard" maxlength="100" spellcheck="false" disabled={hasWorld} /></label>
        {@render toggle(hardcore, 'Hardcore', 'Une seule vie.', hasWorld)}
        {@render pick('spawn-protection', 'Protection du point de départ', [['0', 'Aucune'], ['8', '8 blocs'], ['16', '16 blocs'], ['32', '32 blocs'], ['64', '64 blocs']])}
      </div>
      {#if hasWorld}<p class="note">Type, graine et hardcore sont fixés à la création du monde.</p>{/if}

    {:else if page === 'access'}
      <h2>Accès</h2>
      <div class="card">
        <label class="row"><div><strong>Message d'accueil</strong><small>Affiché dans la liste des serveurs du jeu, deux lignes.</small></div><textarea class="input" rows="2" bind:value={f.motd} maxlength="120"></textarea></label>
        {@render slider('max-players', 'Joueurs maximum', 1, 100, (n) => `${n}`)}
        {@render toggle(white, 'Liste blanche', 'Seuls les joueurs ajoutés peuvent entrer.')}
        {@render toggle(online, 'Comptes officiels seulement', f['online-mode'] === 'true' ? '' : "N'importe qui peut prendre le pseudo d'un autre.")}
        {@render pick('player-idle-timeout', 'Expulser les inactifs', [['0', 'Jamais'], ['5', 'Après 5 min'], ['10', 'Après 10 min'], ['15', 'Après 15 min'], ['30', 'Après 30 min'], ['60', 'Après 1 h']])}
      </div>

    {:else if page === 'perf'}
      <h2>Performances</h2>
      <div class="presets" role="group" aria-label="Réglages rapides">
        {#each PRESETS as p (p[0])}<button type="button" class="btn" aria-pressed={isPreset(p)} onclick={() => preset(p)}>{p[0]}</button>{/each}
      </div>
      <div class="card">
        {@render slider('view-distance', "Distance d'affichage", 3, 32, chunks, `${view * 16} blocs, jusqu'à ${(2 * view + 1) ** 2} chunks par joueur.`)}
        {#if simOn}{@render slider('simulation-distance', 'Distance de simulation', 3, 32, chunks, Number(f['simulation-distance']) > view ? "Inutile au-delà de la distance d'affichage." : 'Zone où les monstres et les cultures vivent.')}{/if}
        {#if entOn}{@render slider('entity-broadcast-range-percentage', 'Distance des créatures', 10, 200, (n) => `${n} %`, 'À partir de quand un joueur les voit.', 10)}{/if}
      </div>
      {#if view > 16}<p class="note warn">Très lourd : réservé à un serveur puissant avec peu de joueurs.</p>{/if}
    {/if}
  </fieldset>
  {#if error && !ready}<button class="btn" onclick={load}>Réessayer</button>{/if}
</SubPages>

<SaveBar {dirty} {busy} {error} onsave={save} onreset={() => ((f = { ...saved }), (error = ''))} />
<Confirm
  bind:open={ask}
  title={f['online-mode'] === 'true' ? 'Revenir aux comptes officiels ?' : 'Accepter les comptes non officiels ?'}
  text="Chaque joueur repartira avec un inventaire vide et devra être remis opérateur ou en liste blanche."
  action="Changer"
  onconfirm={send}
/>

<style>
  .row { border-top: 0; padding-inline: 0; }
  fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
  textarea { font: inherit; height: auto; padding-block: 8px; resize: none; }
  .note { color: var(--muted); font-size: 0.89rem; margin-top: 10px; }
  .warn { color: var(--warn, oklch(0.75 0.15 70)); }
  .slide { display: flex; align-items: center; gap: 12px; }
  .slide input { flex: 1; min-width: 0; accent-color: var(--primary); cursor: pointer; }
  .slide input:focus-visible { outline: 2px solid var(--primary); outline-offset: 4px; }
  .slide output { min-width: 6.5em; text-align: right; font-variant-numeric: tabular-nums; }
  .presets { display: flex; gap: 8px; margin: 12px 0 4px; }
  .presets [aria-pressed='true'] { background: var(--primary-soft); color: var(--primary); }
  .restart { display: flex; align-items: center; gap: 12px; margin-bottom: 14px; }
</style>

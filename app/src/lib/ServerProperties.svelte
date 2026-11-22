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
    'view-distance': '10', 'simulation-distance': '10',
  }
  type Props = typeof DEF
  type Key = keyof Props
  let saved = $state<Props>({ ...DEF })
  let f = $state<Props>({ ...DEF })
  let ready = $state(false)
  let hasWorld = $state(false)
  let busy = $state(false)
  let error = $state('')
  let restart = $state(false)
  let ask = $state(false)
  const dirty = $derived(JSON.stringify(f) !== JSON.stringify(saved))
  const url = `/api/servers/${server.id}/properties`

  const mc = software(server).mc
  const since = (v: string) => !mc || cmpVersion(mc, v) >= 0
  // devenus des règles de jeu en 1.21.9 : le fichier n'est plus lu
  const oldRules = !since('1.21.9')

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
  const write = (k: Key, v: string) => (NAMES[k] && !since('1.14') ? String(NAMES[k].indexOf(v)) : v)

  async function load() {
    error = ''
    try {
      const p = await call<Record<string, string>>(conn, url)
      const v = { ...DEF }
      for (const k of Object.keys(DEF) as Key[]) if (p[k] !== undefined) v[k] = read(k, p[k])
      const world = encodeURIComponent(p['level-name'] || 'world')
      hasWorld = await call(conn, `/api/servers/${server.id}/files?path=${world}`).then(() => true, () => false)
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
    if (server.status !== 'running') untrack(() => (restart = false))
  })

  const NUM: [Key, number, number, string][] = [
    ['view-distance', 3, 32, "Distance d'affichage"],
    ['simulation-distance', 3, 32, 'Distance de simulation'],
    ['max-players', 1, 1000, 'Joueurs maximum'],
    ['spawn-protection', 0, 1000, 'Protection du point de départ'],
    ['player-idle-timeout', 0, 1440, 'Inactivité'],
  ]

  function save() {
    error = ''
    for (const [k, min, max, label] of NUM) {
      if (f[k] === saved[k]) continue
      const x = Number(f[k])
      if (!f[k].trim() || !Number.isInteger(x) || x < min || x > max) return void (error = `${label} : entre ${min} et ${max}`)
    }
    if (hasWorld && f['online-mode'] !== saved['online-mode']) ask = true
    else send()
  }

  async function send() {
    busy = true
    error = ''
    const next = { ...f }
    try {
      const body: Record<string, string> = {}
      for (const k of Object.keys(next) as Key[]) if (next[k] !== saved[k]) body[k] = write(k, next[k].trim().replace(/\r/g, ''))
      // sans enforce-whitelist, les joueurs déjà connectés restent quand on active la liste
      if (body['white-list']) body['enforce-whitelist'] = body['white-list']
      await call(conn, url, { method: 'PUT', body: JSON.stringify(body) })
      saved = next
      restart = server.status === 'running'
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
      restart = false
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }
</script>

{#snippet num(k: Key, label: string, hint = '')}
  <label class="row"><div><strong>{label}</strong>{#if hint}<small>{hint}</small>{/if}</div><input class="input" inputmode="numeric" bind:value={f[k]} /></label>
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
        {#if oldRules}
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
        {@render num('spawn-protection', 'Protection du point de départ', 'En blocs. 0 = désactivée.')}
      </div>
      {#if hasWorld}<p class="note">Type, graine et hardcore sont fixés à la création du monde.</p>{/if}

    {:else if page === 'access'}
      <h2>Accès</h2>
      <div class="card">
        <label class="row"><div><strong>Message d'accueil</strong><small>Affiché dans la liste des serveurs du jeu, deux lignes.</small></div><textarea class="input" rows="2" bind:value={f.motd} maxlength="120"></textarea></label>
        {@render num('max-players', 'Joueurs maximum')}
        {@render toggle(white, 'Liste blanche', 'Seuls les joueurs ajoutés peuvent entrer.')}
        {@render toggle(online, 'Comptes officiels seulement', f['online-mode'] === 'true' ? '' : "N'importe qui peut prendre le pseudo d'un autre.")}
        {@render num('player-idle-timeout', 'Expulser les inactifs', 'En minutes. 0 = jamais.')}
      </div>

    {:else if page === 'perf'}
      <h2>Performances</h2>
      <div class="card">
        {@render num('view-distance', "Distance d'affichage", 'De 3 à 32. Plus bas = moins de mémoire.')}
        {#if since('1.18')}{@render num('simulation-distance', 'Distance de simulation', 'De 3 à 32. Plus bas = moins de processeur.')}{/if}
      </div>
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
  fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
  textarea { font: inherit; height: auto; padding-block: 8px; resize: none; }
  .note { color: var(--muted); font-size: 0.89rem; margin-top: 10px; }
  .restart { display: flex; align-items: center; gap: 12px; margin-bottom: 14px; }
</style>

<script lang="ts">
  import { Earth, Swords, Users } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import SaveBar from './SaveBar.svelte'
  import { call, type Conn, type Server } from './api'

  let { conn, server }: { conn: Conn; server: Server } = $props()

  const pages = [
    { id: 'world', label: 'Monde', icon: Earth },
    { id: 'game', label: 'Partie', icon: Swords },
    { id: 'access', label: 'Accès', icon: Users },
  ]
  let page = $state('world')

  // valeurs par défaut de Minecraft quand la clé manque dans le fichier
  const DEF = { motd: 'A Minecraft Server', 'level-seed': '', 'view-distance': '10', gamemode: 'survival', difficulty: 'easy', pvp: 'true', hardcore: 'false', 'max-players': '20', 'white-list': 'false', 'online-mode': 'true' }
  type Props = typeof DEF
  let saved = $state<Props>({ ...DEF })
  let f = $state<Props>({ ...DEF })
  let loading = $state(true)
  let busy = $state(false)
  let error = $state('')
  const dirty = $derived(JSON.stringify(f) !== JSON.stringify(saved))
  const url = `/api/servers/${server.id}/properties`

  call<Record<string, string>>(conn, url)
    .then((p) => {
      const v = { ...DEF }
      for (const k of Object.keys(DEF) as (keyof Props)[]) if (p[k] !== undefined) v[k] = p[k]
      saved = v
      f = { ...v }
    })
    .catch((e) => (error = (e as Error).message))
    .finally(() => (loading = false))

  // case à cocher ↔ "true"/"false" du fichier
  const bool = (k: keyof Props) => ({ get: () => f[k] === 'true', set: (v: boolean) => (f[k] = String(v)) })
  const pvp = bool('pvp'), hardcore = bool('hardcore'), white = bool('white-list'), online = bool('online-mode')

  async function save() {
    const n = (k: keyof Props, min: number, max: number, label: string) => {
      const x = Number(f[k])
      if (!Number.isInteger(x) || x < min || x > max) throw new Error(`${label} : entre ${min} et ${max}`)
    }
    busy = true
    error = ''
    try {
      n('view-distance', 3, 32, "Distance d'affichage")
      n('max-players', 1, 1000, 'Joueurs maximum')
      const body: Record<string, string> = {}
      for (const k of Object.keys(f) as (keyof Props)[]) if (f[k] !== saved[k]) body[k] = f[k]
      // sans enforce-whitelist, les joueurs déjà connectés restent quand on active la liste
      if (body['white-list']) body['enforce-whitelist'] = body['white-list']
      await call(conn, url, { method: 'PUT', body: JSON.stringify(body) })
      saved = { ...f }
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }
</script>

<SubPages {pages} bind:page label="Propriétés Minecraft">
  <fieldset disabled={loading}>
    {#if page === 'world'}
      <h2>Monde</h2>
      <div class="card">
        <label class="row"><div><strong>Message d'accueil</strong><small>Affiché dans la liste des serveurs du jeu.</small></div><input class="input" bind:value={f.motd} maxlength="59" /></label>
        <label class="row"><div><strong>Graine du monde</strong><small>Vide = au hasard. Pour un nouveau monde seulement.</small></div><input class="input" bind:value={f['level-seed']} /></label>
        <label class="row"><div><strong>Distance d'affichage</strong><small>En tronçons. Plus haut = plus lourd.</small></div><input class="input" inputmode="numeric" bind:value={f['view-distance']} /></label>
      </div>

    {:else if page === 'game'}
      <h2>Partie</h2>
      <div class="card">
        <label class="row"><div><strong>Mode de jeu</strong></div><select class="input" bind:value={f.gamemode}><option value="survival">Survie</option><option value="creative">Créatif</option><option value="adventure">Aventure</option><option value="spectator">Spectateur</option></select></label>
        <label class="row"><div><strong>Difficulté</strong></div><select class="input" bind:value={f.difficulty}><option value="peaceful">Paisible</option><option value="easy">Facile</option><option value="normal">Normale</option><option value="hard">Difficile</option></select></label>
        <label class="row"><div><strong>Combat entre joueurs</strong></div><input type="checkbox" class="switch" bind:checked={pvp.get, pvp.set} /></label>
        <label class="row"><div><strong>Mode hardcore</strong><small>Une seule vie.</small></div><input type="checkbox" class="switch" bind:checked={hardcore.get, hardcore.set} /></label>
      </div>

    {:else if page === 'access'}
      <h2>Accès</h2>
      <div class="card">
        <label class="row"><div><strong>Joueurs maximum</strong></div><input class="input" inputmode="numeric" bind:value={f['max-players']} /></label>
        <label class="row"><div><strong>Liste blanche</strong><small>Seuls les joueurs ajoutés peuvent entrer.</small></div><input type="checkbox" class="switch" bind:checked={white.get, white.set} /></label>
        <label class="row"><div><strong>Comptes officiels seulement</strong><small>À laisser activé.</small></div><input type="checkbox" class="switch" bind:checked={online.get, online.set} /></label>
      </div>
    {/if}
  </fieldset>
</SubPages>

<SaveBar {dirty} {busy} {error} onsave={save} onreset={() => ((f = { ...saved }), (error = ''))} />

<style>
  fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
</style>

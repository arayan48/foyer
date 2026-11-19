<script lang="ts">
  import { IdCard, Cpu, Power, TriangleAlert, Trash2 } from 'lucide-svelte'
  import SaveBar from './SaveBar.svelte'
  import Confirm from './Confirm.svelte'
  import SubPages from './SubPages.svelte'
  import { call, patchServer, type Conn, type Host, type Server } from './api'

  let { conn, server, ondelete }: { conn: Conn; server: Server; ondelete: (s: Server) => void } = $props()

  // copie figée à l'ouverture : la liste se rafraîchit toutes les 2 s, il ne faut pas écraser la saisie
  const base = () => ({ name: server.name, ram: Math.max(1, Math.round(server.ram_mb / 1024)), auto_start: server.auto_start, auto_restart: server.auto_restart, cpu_limit: server.cpu_limit ?? 0, cores: [...(server.cores ?? [])], ram_cap: server.ram_cap ?? false })
  let saved = $state(base())
  let f = $state(base())
  let busy = $state(false)
  let error = $state('')
  let confirmDelete = $state(false)
  const pages = [
    { id: 'server', label: 'Serveur', icon: IdCard },
    { id: 'resources', label: 'Ressources', icon: Cpu },
    { id: 'startup', label: 'Démarrage', icon: Power },
    { id: 'danger', label: 'Zone sensible', icon: TriangleAlert, danger: true },
  ]
  let page = $state('server')
  // cœurs logiques de la machine, pour la grille des cœurs
  let threads = $state(0)
  call<Host>(conn, '/api/system').then((h) => (threads = Math.min(h.threads ?? 0, 64))).catch(() => {})
  const toggleCore = (c: number) => (f.cores = f.cores.includes(c) ? f.cores.filter((x) => x !== c) : [...f.cores, c].sort((a, b) => a - b))
  const dirty = $derived(JSON.stringify(f) !== JSON.stringify(saved))

  async function save() {
    busy = true
    error = ''
    try {
      await patchServer(conn, server.id, { name: f.name, ram_mb: f.ram * 1024, auto_start: f.auto_start, auto_restart: f.auto_restart, cpu_limit: f.cpu_limit, cores: f.cores, ram_cap: f.ram_cap })
      saved = { ...f, cores: [...f.cores] }
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }
</script>

<SubPages {pages} bind:page label="Réglages généraux" onselect={() => (confirmDelete = false)}>
    {#if page === 'server'}
      <h2>Serveur</h2>
      <div class="card">
        <label class="row">
          <div><strong>Nom</strong><small>{f.name.length} / 60 caractères.</small></div>
          <input class="input" bind:value={f.name} maxlength="60" />
        </label>
        <div class="row">
          <div><strong>Jeu</strong></div>
          <div class="val">{server.game === 'minecraft' ? 'Minecraft' : 'FiveM'}</div>
        </div>
        <div class="row">
          <div><strong>Port</strong><small>{server.game === 'minecraft' ? "Se change dans l'onglet Réseau." : 'Se change dans txAdmin.'}</small></div>
          <div class="val">{server.port}</div>
        </div>
      </div>

    {:else if page === 'resources'}
      <h2>Ressources</h2>
      <p class="lead">Ce que le serveur a le droit d'utiliser. S'applique au prochain démarrage.</p>
      <h3>Mémoire</h3>
      <div class="card">
        <div class="row">
          <div><strong>Mémoire réservée</strong><small>{server.game === 'minecraft' ? 'Donnée à Java au lancement.' : 'Sert au plafond ci-dessous.'}</small></div>
          <div class="val"><input type="range" min="1" max="16" bind:value={f.ram} aria-label="Mémoire réservée" /><output class="num">{f.ram} Go</output></div>
        </div>
        <label class="row">
          <div><strong>Ne jamais dépasser</strong><small>{server.game === 'minecraft' ? `Plafond à ${f.ram} Go plus la marge de Java.` : `Plafond à ${f.ram} Go.`} Au-delà, le système arrête le serveur.</small></div>
          <input type="checkbox" class="switch" bind:checked={f.ram_cap} />
        </label>
      </div>
      <h3>Processeur</h3>
      <div class="card">
        <div class="row">
          <div><strong>Part maximale</strong><small>Du processeur de toute la machine.</small></div>
          <div class="seg" role="radiogroup" aria-label="Part maximale du processeur">
            {#each [0, 25, 50, 75] as n}
              <button type="button" role="radio" aria-checked={f.cpu_limit === n} onclick={() => (f.cpu_limit = n)}>{n === 0 ? 'Libre' : `${n} %`}</button>
            {/each}
          </div>
        </div>
        {#if threads > 1}
          <div class="row wide">
            <div><strong>Cœurs autorisés</strong><small>{f.cores.length === 0 ? `Tous (${threads}). Choisis-en pour réserver ce serveur à certains cœurs.` : `${f.cores.length} sur ${threads}.`}</small></div>
            <div class="cores" role="group" aria-label="Cœurs autorisés">
              {#each Array.from({ length: threads }, (_, i) => i) as c}
                <button type="button" aria-pressed={f.cores.includes(c)} onclick={() => toggleCore(c)}>{c + 1}</button>
              {/each}
              {#if f.cores.length > 0}<button type="button" class="all" onclick={() => (f.cores = [])}>Tous</button>{/if}
            </div>
          </div>
        {/if}
      </div>

    {:else if page === 'startup'}
      <h2>Démarrage</h2>
      <div class="card">
        <label class="row">
          <div><strong>Démarrer avec la machine</strong><small>Lance le serveur dès que Foyer démarre.</small></div>
          <input type="checkbox" class="switch" bind:checked={f.auto_start} />
        </label>
        <label class="row">
          <div><strong>Relancer après un crash</strong><small>3 essais en 10 minutes, puis Foyer abandonne.</small></div>
          <input type="checkbox" class="switch" bind:checked={f.auto_restart} />
        </label>
      </div>

    {:else}
      <h2>Zone sensible</h2>
      <p class="lead">Actions définitives. Rien ne peut être annulé.</p>
      <div class="card danger">
        <div class="row">
          <div><strong>Supprimer le serveur</strong><small>Efface le serveur, son monde et tous ses fichiers.</small></div>
          <div class="val"><button class="btn danger" onclick={() => (confirmDelete = true)}><Trash2 size={14} /> Supprimer…</button></div>
        </div>
      </div>
      <Confirm bind:open={confirmDelete} title="Supprimer « {server.name} » ?" text="Le monde et tous les fichiers partent. Aucune sauvegarde n'est faite, c'est définitif." onconfirm={() => ondelete(server)} />
    {/if}
</SubPages>

<SaveBar {dirty} {busy} {error} onsave={save} onreset={() => { f = { ...saved, cores: [...saved.cores] }; error = '' }} />

<style>
  .val input[type='range'] { width: 150px; }
  .num { min-width: 46px; text-align: right; color: var(--ink); font-weight: 500; }

  .seg { justify-self: end; display: inline-flex; border: 1px solid var(--line); border-radius: 8px; overflow: hidden; }
  .seg button { height: 30px; padding: 0 12px; border: 0; border-left: 1px solid var(--line); background: none; color: var(--muted); font-size: 0.85rem; cursor: pointer; font-variant-numeric: tabular-nums; }
  .seg button:first-child { border-left: 0; }
  .seg button:hover { color: var(--ink); background: var(--surface-2); }
  .seg button[aria-checked='true'] { background: var(--surface-2); color: var(--ink); box-shadow: inset 0 -2px 0 var(--primary); }

  .cores { display: flex; flex-wrap: wrap; gap: 6px; }
  .cores button { min-width: 34px; height: 30px; padding: 0 8px; border-radius: 6px; border: 1px solid var(--line); background: none; color: var(--muted); font-size: 0.82rem; font-variant-numeric: tabular-nums; cursor: pointer; }
  .cores button:hover { color: var(--ink); border-color: oklch(0.42 0.012 265); }
  .cores button[aria-pressed='true'] { background: var(--primary); border-color: transparent; color: var(--primary-ink); }
  .cores .all { color: var(--primary); }

  .danger { border-color: color-mix(in oklch, var(--err) 40%, var(--line)); }

</style>

<script lang="ts">
  import { IdCard, Cpu, Power, TriangleAlert, Trash2 } from 'lucide-svelte'
  import SaveBar from './SaveBar.svelte'
  import { patchServer, type Conn, type Server } from './api'

  let { conn, server, ondelete }: { conn: Conn; server: Server; ondelete: (s: Server) => void } = $props()

  // copie figée à l'ouverture : la liste se rafraîchit toutes les 2 s, il ne faut pas écraser la saisie
  const base = () => ({ name: server.name, ram: Math.max(1, Math.round(server.ram_mb / 1024)), auto_start: server.auto_start, auto_restart: server.auto_restart })
  let saved = $state(base())
  let f = $state(base())
  let busy = $state(false)
  let error = $state('')
  let confirmDelete = $state(false)
  const dirty = $derived(JSON.stringify(f) !== JSON.stringify(saved))

  async function save() {
    busy = true
    error = ''
    try {
      await patchServer(conn, server.id, { name: f.name, ram_mb: f.ram * 1024, auto_start: f.auto_start, auto_restart: f.auto_restart })
      saved = { ...f }
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }
</script>

<div class="card">
  <div class="card-head"><IdCard size={18} /><div><h2>Identité</h2><p>Comment ce serveur s'affiche dans Foyer.</p></div></div>
  <label class="row">
    <div><strong>Nom</strong><small>60 caractères maximum.</small></div>
    <input class="input" bind:value={f.name} maxlength="60" />
  </label>
</div>

<div class="card">
  <div class="card-head"><Cpu size={18} /><div><h2>Ressources</h2><p>S'applique au prochain démarrage.</p></div></div>
  <label class="row">
    <div><strong>Mémoire réservée</strong><small>{f.ram} Go pour le jeu.</small></div>
    <input type="range" min="1" max="16" bind:value={f.ram} />
  </label>
  <label class="row">
    <div><strong>Limite processeur <span class="soon">Bientôt</span></strong><small>Part maximale du processeur que le jeu peut prendre.</small></div>
    <input type="range" min="10" max="100" value="100" disabled />
  </label>
  <label class="row">
    <div><strong>Cœurs dédiés <span class="soon">Bientôt</span></strong><small>Réserver certains cœurs à ce serveur.</small></div>
    <select class="input" disabled><option>Tous les cœurs</option></select>
  </label>
</div>

<div class="card">
  <div class="card-head"><Power size={18} /><div><h2>Démarrage</h2><p>Ce que fait Foyer quand la machine s'allume ou que le jeu plante.</p></div></div>
  <label class="row">
    <div><strong>Démarrer avec la machine</strong><small>Lance le serveur dès que Foyer démarre.</small></div>
    <input type="checkbox" class="switch" bind:checked={f.auto_start} />
  </label>
  <label class="row">
    <div><strong>Relancer en cas de crash</strong><small>3 essais maximum en 10 minutes, puis Foyer abandonne.</small></div>
    <input type="checkbox" class="switch" bind:checked={f.auto_restart} />
  </label>
</div>

<div class="card danger">
  <div class="card-head"><TriangleAlert size={18} /><div><h2>Zone sensible</h2><p>Supprime le serveur, son monde et tous ses fichiers. C'est définitif.</p></div></div>
  <div class="row">
    {#if confirmDelete}
      <div><strong>Supprimer « {server.name} » ?</strong><small>Aucune sauvegarde n'est faite.</small></div>
      <div class="val"><button class="btn" onclick={() => (confirmDelete = false)}>Annuler</button><button class="btn danger" onclick={() => ondelete(server)}><Trash2 size={14} /> Supprimer</button></div>
    {:else}
      <div><strong>Supprimer le serveur</strong></div>
      <div class="val"><button class="btn danger" onclick={() => (confirmDelete = true)}><Trash2 size={14} /> Supprimer…</button></div>
    {/if}
  </div>
</div>

<SaveBar {dirty} {busy} {error} onsave={save} onreset={() => { f = { ...saved }; error = '' }} />

<style>
  .danger .card-head :global(svg) { color: var(--err); }
</style>

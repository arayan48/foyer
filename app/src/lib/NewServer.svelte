<script lang="ts">
  import { ArrowLeft } from 'lucide-svelte'
  import GameMark from './GameMark.svelte'
  import { GAMES, call, type Conn, type Game, type Server } from './api'

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

  // premier port libre à partir du port par défaut du jeu
  let port = $derived.by(() => {
    const used = new Set(servers.flatMap((s) => [s.port, s.admin_port]))
    let p = GAMES[game].port
    while (used.has(p) || (game === 'fivem' && used.has(p + 10000))) p++
    return p
  })

  async function submit(e: SubmitEvent) {
    e.preventDefault()
    busy = true
    error = ''
    try {
      const s = await call<Server>(conn, '/api/servers', {
        method: 'POST',
        body: JSON.stringify({ name, game, port, ram_mb: ram * 1024, eula }),
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
  <button class="btn icon" onclick={oncancel} aria-label="Retour"><ArrowLeft size={16} /></button>
  <h1>Nouveau serveur</h1>
</header>

<form onsubmit={submit}>
  <fieldset>
    <legend>Jeu</legend>
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

  <label class="field">
    <span>Nom</span>
    <input class="input" bind:value={name} maxlength="60" required placeholder={game === 'minecraft' ? 'Survie entre potes' : 'RP de la ville'} />
  </label>

  {#if game === 'minecraft'}
    <label class="field">
      <span>Mémoire réservée : {ram} Go</span>
      <input type="range" min="1" max="12" bind:value={ram} />
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={eula} required />
      <span>J'accepte le <a href="https://aka.ms/MinecraftEULA" target="_blank" rel="noreferrer">contrat de licence (EULA) de Minecraft</a>.</span>
    </label>
  {:else}
    <p class="note">Au premier démarrage, la console affiche un code PIN. Ouvre ensuite txAdmin dans le navigateur pour finir la configuration (clé de licence Cfx.re, ressources).</p>
  {/if}

  <p class="note">Port du jeu : <strong>{port}</strong>{#if game === 'fivem'}, txAdmin : <strong>{port + 10000}</strong>{/if}. Ouvert sur le réseau local uniquement.</p>

  {#if error}<p class="err-text" role="alert">{error}</p>{/if}
  <div class="foot">
    <button type="button" class="btn" onclick={oncancel}>Annuler</button>
    <button class="btn primary" disabled={busy || !name.trim()}>{busy ? 'Création…' : 'Créer et installer'}</button>
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
  input[type='range'] { accent-color: var(--primary); }
  .check { display: flex; gap: 10px; align-items: flex-start; }
  .check input { accent-color: var(--primary); margin-top: 4px; }
  a { color: var(--primary); }
  .note { color: var(--muted); max-width: 65ch; }
  .note strong { color: var(--ink); font-variant-numeric: tabular-nums; }
  .foot { display: flex; gap: 8px; justify-content: flex-end; }
</style>

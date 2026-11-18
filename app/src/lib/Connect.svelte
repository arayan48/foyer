<script lang="ts">
  import { Flame } from 'lucide-svelte'
  import { call, normalizeUrl, AuthError, type Conn } from './api'

  let { onconnect }: { onconnect: (c: Conn) => void } = $props()
  let url = $state('')
  let token = $state('')
  let busy = $state(false)
  let error = $state('')

  // Agent installé sur ce même PC : connexion directe, sans saisie
  $effect(() => {
    import('@tauri-apps/api/core')
      .then(({ invoke }) => invoke<string | null>('local_token'))
      .then(async (t) => {
        if (!t) return
        const c = { url: 'http://127.0.0.1:7878', token: t }
        await call(c, '/api/system')
        onconnect(c)
      })
      .catch(() => {}) // hors Tauri ou agent arrêté : formulaire normal
  })

  async function submit(e: SubmitEvent) {
    e.preventDefault()
    busy = true
    error = ''
    const c = { url: normalizeUrl(url), token: token.trim() }
    try {
      await call(c, '/api/system')
      onconnect(c)
    } catch (err) {
      error = err instanceof AuthError ? 'Jeton incorrect.' : `Agent injoignable à ${c.url}. Vérifie qu'il tourne et que le port est ouvert.`
    } finally {
      busy = false
    }
  }
</script>

<main>
  <form onsubmit={submit}>
    <div class="mark"><Flame size={22} /></div>
    <h1>Connexion à un serveur</h1>
    <p class="lead">Entre l'adresse de la machine où tourne l'agent Foyer, et son jeton.</p>
    <label class="field">
      <span>Adresse</span>
      <input class="input" bind:value={url} placeholder="192.168.1.240" required autocomplete="off" spellcheck="false" />
    </label>
    <label class="field">
      <span>Jeton</span>
      <input class="input mono" type="password" bind:value={token} placeholder="Fichier C:\Foyer\token sur le serveur" required />
    </label>
    {#if error}<p class="err-text" role="alert">{error}</p>{/if}
    <button class="btn primary big" disabled={busy}>{busy ? 'Connexion…' : 'Se connecter'}</button>
  </form>
</main>

<style>
  main { height: 100%; display: grid; place-items: center; padding: 24px; }
  form { width: min(380px, 100%); display: grid; gap: 16px; }
  .mark {
    width: 44px; height: 44px; border-radius: 12px; display: grid; place-items: center;
    background: oklch(0.72 0.18 357 / 0.14); color: var(--primary);
  }
  .lead { color: var(--muted); margin-top: -8px; }
  .mono { font-family: ui-monospace, 'SF Mono', Consolas, monospace; }
  .big { height: 40px; justify-content: center; margin-top: 4px; }
</style>

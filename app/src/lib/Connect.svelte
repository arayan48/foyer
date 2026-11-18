<script lang="ts">
  import { Flame, ShieldCheck } from 'lucide-svelte'
  import { call, probe, inTauri, normalizeUrl, AuthError, type Conn } from './api'

  let { onconnect }: { onconnect: (c: Conn) => void } = $props()
  let url = $state('')
  let token = $state('')
  let busy = $state(false)
  let error = $state('')
  /** empreinte lue, en attente de confirmation par l'utilisateur */
  let pending = $state<Conn | null>(null)

  // Agent installé sur ce même PC : connexion directe, l'empreinte est lue dans son dossier
  $effect(() => {
    import('@tauri-apps/api/core')
      .then(({ invoke }) => invoke<[string, string] | null>('local_agent'))
      .then(async (a) => {
        if (!a) return
        const c = { url: 'https://127.0.0.1:7878', token: a[0], fp: a[1] }
        await call(c, '/api/system')
        onconnect(c)
      })
      .catch(() => {}) // hors Tauri ou agent arrêté : formulaire normal
  })

  async function tryConn(c: Conn) {
    try {
      await call(c, '/api/system')
      onconnect(c)
    } catch (err) {
      pending = null
      error = err instanceof AuthError ? err.message.startsWith('Jeton') ? 'Jeton incorrect.' : err.message : `Agent injoignable à ${c.url}. Vérifie qu'il tourne et que le port est ouvert.`
    }
  }

  async function submit(e: SubmitEvent) {
    e.preventDefault()
    busy = true
    error = ''
    const c: Conn = { url: normalizeUrl(url), token: token.trim() }
    try {
      if (!inTauri) return await tryConn(c)
      c.fp = await probe(c.url)
      pending = c
    } catch {
      error = `Agent injoignable à ${c.url}. Vérifie qu'il tourne et que le port est ouvert.`
    } finally {
      busy = false
    }
  }

  async function confirm() {
    busy = true
    await tryConn(pending!)
    busy = false
  }
</script>

<main>
  {#if pending}
    <div class="form">
      <div class="mark"><ShieldCheck size={22} /></div>
      <h1>Vérifie l'empreinte</h1>
      <p class="lead">Elle doit être identique à celle du serveur : fichier <code>C:\Foyer\fingerprint</code>, ou au démarrage de l'agent. Sinon, quelqu'un se fait peut-être passer pour lui.</p>
      <p class="fp mono">{pending.fp}</p>
      <div class="row">
        <button class="btn" onclick={() => (pending = null)} disabled={busy}>Annuler</button>
        <button class="btn primary" onclick={confirm} disabled={busy}>{busy ? 'Connexion…' : 'Elle est identique'}</button>
      </div>
    </div>
  {:else}
  <form class="form" onsubmit={submit}>
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
  {/if}
</main>

<style>
  main { height: 100%; display: grid; place-items: center; padding: 24px; }
  .form { width: min(380px, 100%); display: grid; gap: 16px; }
  .fp { padding: 12px; border-radius: 8px; background: var(--surface-2); word-break: break-all; line-height: 1.6; font-size: 13px; }
  .row { display: flex; gap: 8px; justify-content: flex-end; }
  .mark {
    width: 44px; height: 44px; border-radius: 12px; display: grid; place-items: center;
    background: oklch(0.72 0.18 357 / 0.14); color: var(--primary);
  }
  .lead { color: var(--muted); margin-top: -8px; }
  .mono { font-family: ui-monospace, 'SF Mono', Consolas, monospace; }
  .big { height: 40px; justify-content: center; margin-top: 4px; }
</style>

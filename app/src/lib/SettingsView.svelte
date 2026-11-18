<script lang="ts">
  import { LogOut, RefreshCw } from 'lucide-svelte'
  import { applySettings, loadSettings } from './settings'
  import type { Conn } from './api'

  let { conn, onlogout }: { conn: Conn; onlogout: () => void } = $props()

  const inApp = '__TAURI_INTERNALS__' in window
  let s = $state(loadSettings())
  let autostart = $state(false)
  let version = $state('')
  let upd = $state<'idle' | 'checking' | 'none' | 'found' | 'installing' | 'error'>('idle')
  let newVersion = $state('')
  let install: (() => Promise<void>) | null = null

  $effect(() => {
    if (!inApp) return
    import('@tauri-apps/api/app').then(({ getVersion }) => getVersion()).then((v) => (version = v))
    import('@tauri-apps/plugin-autostart').then(({ isEnabled }) => isEnabled()).then((v) => (autostart = v))
  })

  function toggleTray() {
    s.closeToTray = !s.closeToTray
    applySettings(s)
  }

  async function toggleAutostart() {
    const a = await import('@tauri-apps/plugin-autostart')
    autostart ? await a.disable() : await a.enable()
    autostart = await a.isEnabled()
  }

  async function checkUpdate() {
    upd = 'checking'
    try {
      const { check } = await import('@tauri-apps/plugin-updater')
      const u = await check()
      if (!u) return (upd = 'none')
      newVersion = u.version
      install = async () => {
        await u.downloadAndInstall()
        const { relaunch } = await import('@tauri-apps/plugin-process')
        await relaunch()
      }
      upd = 'found'
    } catch {
      upd = 'error'
    }
  }

  async function doInstall() {
    upd = 'installing'
    try {
      await install?.()
    } catch {
      upd = 'error'
    }
  }
</script>

<header><h1>Paramètres</h1></header>

<section>
  <h2>Fenêtre</h2>
  <div class="opt">
    <div>
      <strong>Rester en arrière-plan à la fermeture</strong>
      <p>La croix cache la fenêtre. Foyer reste dans la barre des tâches, près de l'horloge. Pour quitter : clic droit sur l'icône, puis « Quitter Foyer ».</p>
    </div>
    <button class="switch" role="switch" aria-checked={s.closeToTray} aria-label="Rester en arrière-plan à la fermeture" onclick={toggleTray} disabled={!inApp}><span></span></button>
  </div>
  <div class="opt">
    <div>
      <strong>Lancer Foyer au démarrage de l'ordinateur</strong>
      <p>Foyer démarre caché dans la barre, prêt à recevoir les alertes.</p>
    </div>
    <button class="switch" role="switch" aria-checked={autostart} aria-label="Lancer Foyer au démarrage" onclick={toggleAutostart} disabled={!inApp}><span></span></button>
  </div>
  {#if !inApp}<p class="hint">Ces réglages ne marchent que dans l'application installée.</p>{/if}
</section>

<section>
  <h2>Mises à jour</h2>
  <div class="opt">
    <div>
      <strong>Version {version || '—'}</strong>
      <p>
        {#if upd === 'checking'}Recherche…
        {:else if upd === 'none'}Foyer est à jour.
        {:else if upd === 'found'}Foyer {newVersion} est disponible.
        {:else if upd === 'installing'}Installation, Foyer va redémarrer…
        {:else if upd === 'error'}Impossible de vérifier. Réessaie plus tard.
        {:else}Foyer vérifie tout seul au lancement puis toutes les 6 h.{/if}
      </p>
    </div>
    {#if upd === 'found' || upd === 'installing'}
      <button class="btn primary" onclick={doInstall} disabled={upd === 'installing'}>Mettre à jour</button>
    {:else}
      <button class="btn" onclick={checkUpdate} disabled={!inApp || upd === 'checking'}><RefreshCw size={14} /> Vérifier</button>
    {/if}
  </div>
</section>

<section>
  <h2>Connexion</h2>
  <div class="opt">
    <div>
      <strong>Agent : <span class="num">{conn.url}</span></strong>
      <p>Se déconnecter pour piloter un autre serveur.</p>
    </div>
    <button class="btn danger" onclick={onlogout}><LogOut size={14} /> Se déconnecter</button>
  </div>
</section>

<style>
  header { margin-bottom: 24px; }
  section { max-width: 680px; margin-bottom: 28px; }
  h2 { font-size: 0.89rem; color: var(--muted); font-weight: 500; margin-bottom: 8px; }
  .opt { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 14px 16px; background: var(--surface); border-radius: var(--r); }
  .opt + .opt { margin-top: 2px; }
  .opt p { color: var(--muted); margin-top: 2px; max-width: 60ch; }
  .opt strong { font-weight: 600; }
  .num { font-variant-numeric: tabular-nums; font-weight: 500; }
  .hint { color: var(--muted); font-size: 0.85rem; margin-top: 8px; }
  .switch {
    flex: none; width: 40px; height: 24px; border-radius: 12px; border: 0; padding: 3px; cursor: pointer;
    background: var(--line); transition: background 200ms var(--ease);
  }
  .switch span { display: block; width: 18px; height: 18px; border-radius: 50%; background: var(--ink); transition: transform 200ms var(--ease); }
  .switch[aria-checked='true'] { background: var(--primary); }
  .switch[aria-checked='true'] span { transform: translateX(16px); background: var(--primary-ink); }
  .switch:disabled { opacity: 0.45; cursor: not-allowed; }
</style>

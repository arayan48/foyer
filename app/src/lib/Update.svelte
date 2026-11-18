<script lang="ts">
  import { Download } from 'lucide-svelte'

  let version = $state('')
  let busy = $state(false)
  let error = $state('')
  let install: (() => Promise<void>) | null = null

  // vérifie au lancement puis toutes les 6 h ; hors Tauri (navigateur), ne fait rien
  $effect(() => {
    const check = async () => {
      try {
        const { check } = await import('@tauri-apps/plugin-updater')
        const u = await check()
        if (!u) return
        version = u.version
        install = async () => {
          await u.downloadAndInstall()
          const { relaunch } = await import('@tauri-apps/plugin-process')
          await relaunch()
        }
      } catch {}
    }
    check()
    const t = setInterval(check, 6 * 3600_000)
    return () => clearInterval(t)
  })

  async function go() {
    busy = true
    error = ''
    try {
      await install?.()
    } catch (e) {
      error = 'Échec de la mise à jour, réessaie plus tard.'
      busy = false
    }
  }
</script>

{#if version}
  <div class="upd" role="status">
    <span>Foyer {version} est disponible.</span>
    {#if error}<span class="err-text">{error}</span>{/if}
    <button class="btn primary" onclick={go} disabled={busy}><Download size={14} /> {busy ? 'Installation…' : 'Mettre à jour'}</button>
  </div>
{/if}

<style>
  .upd {
    position: fixed; right: 20px; bottom: 20px; z-index: var(--z-toast);
    display: flex; align-items: center; gap: 12px; padding: 10px 10px 10px 16px;
    background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r);
    animation: up 200ms var(--ease);
  }
  @keyframes up { from { opacity: 0; transform: translateY(8px); } }
</style>

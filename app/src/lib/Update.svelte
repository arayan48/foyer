<script lang="ts">
  import { RotateCw } from 'lucide-svelte'
  import type { Update } from '@tauri-apps/plugin-updater'

  let version = $state('')
  let progress = $state(0) // 0 à 100
  let ready = $state(false)
  let busy = $state(false)
  let error = $state('')
  let pending: Update | null = null

  // vérifie au lancement puis toutes les 6 h, télécharge en fond ; hors Tauri (navigateur), ne fait rien
  $effect(() => {
    const check = async () => {
      if (pending) return
      try {
        const { check } = await import('@tauri-apps/plugin-updater')
        const u = await check()
        if (!u) return
        pending = u
        version = u.version
        let total = 0
        let done = 0
        await u.download((e) => {
          if (e.event === 'Started') total = e.data.contentLength ?? 0
          else if (e.event === 'Progress' && total) progress = Math.min(100, ((done += e.data.chunkLength) / total) * 100)
        })
        ready = true
      } catch {
        pending = null
        version = ''
      }
    }
    check()
    const t = setInterval(check, 6 * 3600_000)
    return () => clearInterval(t)
  })

  async function restart() {
    busy = true
    error = ''
    try {
      await pending!.install() // installateur sans fenêtre (installMode « quiet »)
      const { relaunch } = await import('@tauri-apps/plugin-process')
      await relaunch()
    } catch {
      error = 'Échec, réessaie plus tard.'
      busy = false
    }
  }
</script>

{#if version}
  <div class="upd" role="status">
    {#if ready}
      <span>Foyer {version} est prêt.</span>
      {#if error}<span class="err-text">{error}</span>{/if}
      <button class="btn primary" onclick={restart} disabled={busy}><RotateCw size={14} /> {busy ? 'Redémarrage…' : 'Redémarrer'}</button>
    {:else}
      <span>Téléchargement de Foyer {version}…</span>
      <span class="bar" aria-label="Progression {Math.round(progress)} %"><span style:width="{progress}%"></span></span>
    {/if}
  </div>
{/if}

<style>
  .upd {
    position: fixed; right: 20px; bottom: 20px; z-index: var(--z-toast);
    display: flex; align-items: center; gap: 12px; min-height: 52px; padding: 10px 10px 10px 16px;
    background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r);
    animation: up 200ms var(--ease);
  }
  .bar { width: 120px; height: 4px; border-radius: 2px; background: var(--surface); overflow: hidden; margin-right: 6px; }
  .bar span { display: block; height: 100%; background: var(--primary); transition: width 300ms var(--ease); }
  @keyframes up { from { opacity: 0; transform: translateY(8px); } }
</style>

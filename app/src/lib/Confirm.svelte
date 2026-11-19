<script lang="ts">
  import { TriangleAlert } from 'lucide-svelte'

  // open = true ouvre la fenêtre ; Échap ou Annuler la ferment sans rien faire
  let { open = $bindable(false), title, text, action = 'Supprimer', onconfirm }: {
    open: boolean
    title: string
    text: string
    action?: string
    onconfirm: () => void
  } = $props()

  let dlg: HTMLDialogElement
  $effect(() => {
    if (open && !dlg.open) dlg.showModal()
    else if (!open && dlg.open) dlg.close()
  })
</script>

<dialog class="dialog" bind:this={dlg} onclose={() => (open = false)} onclick={(e) => e.target === dlg && dlg.close()} aria-labelledby="confirm-title">
  <div class="body">
    <span class="ico"><TriangleAlert size={18} /></span>
    <div>
      <h2 id="confirm-title">{title}</h2>
      <p>{text}</p>
    </div>
  </div>
  <div class="foot">
    <button class="btn" onclick={() => dlg.close()}>Annuler</button>
    <button class="btn danger solid" onclick={() => { dlg.close(); onconfirm() }}>{action}</button>
  </div>
</dialog>

<style>
  .body { display: flex; gap: 14px; padding: 22px 22px 18px; }
  .ico { display: grid; place-items: center; flex: none; width: 36px; height: 36px; border-radius: 10px; color: var(--err); background: oklch(0.7 0.18 25 / 0.14); }
  h2 { font-size: 1rem; margin-bottom: 4px; overflow-wrap: anywhere; }
  p { color: var(--muted); font-size: 0.9rem; }
  .foot { display: flex; justify-content: flex-end; gap: 8px; padding: 14px 22px; background: var(--side); border-top: 1px solid var(--line-soft); }
</style>

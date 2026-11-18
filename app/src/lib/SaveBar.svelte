<script lang="ts">
  let { dirty, busy, error, onsave, onreset }: { dirty: boolean; busy: boolean; error: string; onsave: () => void; onreset: () => void } = $props()
</script>

{#if dirty || error}
  <div class="savebar">
    <span class="msg" class:err={!!error} role={error ? 'alert' : undefined}>{error || 'Modifications non enregistrées'}</span>
    <button class="btn" onclick={onreset} disabled={busy}>Annuler</button>
    <button class="btn primary" onclick={onsave} disabled={busy || !dirty}>{busy ? 'Enregistrement…' : 'Enregistrer'}</button>
  </div>
{/if}

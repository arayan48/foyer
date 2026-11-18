<script lang="ts">
  import type { Component } from 'svelte'
  let { icon: Icon, title, text, version }: { icon: Component<any>; title: string; text: string; version: string } = $props()
</script>

<div class="empty">
  <span class="ic"><Icon size={22} /></span>
  <h2>{title}</h2>
  <p>{text}</p>
  <span class="soon">Prévu pour la {version}</span>
</div>

<style>
  .empty { display: grid; justify-items: center; gap: 8px; text-align: center; padding: 64px 24px; border: 1px dashed var(--line); border-radius: var(--r); }
  .ic { display: grid; place-items: center; width: 48px; height: 48px; border-radius: 12px; background: var(--surface); color: var(--primary); margin-bottom: 4px; }
  p { color: var(--muted); max-width: 46ch; }
</style>

<script lang="ts">
  import type { Component, Snippet } from 'svelte'

  type Page = { id: string; label: string; icon: Component<any>; danger?: boolean }
  let { pages, page = $bindable(), label, onselect, children }: { pages: Page[]; page: string; label: string; onselect?: () => void; children: Snippet } = $props()
</script>

<div class="layout">
  <nav aria-label={label}>
    {#each pages as p (p.id)}
      <button class:danger-link={p.danger} aria-current={page === p.id ? 'page' : undefined} onclick={() => { page = p.id; onselect?.() }}>
        <p.icon size={16} />{p.label}
      </button>
    {/each}
  </nav>
  <div class="content">{@render children()}</div>
</div>

<style>
  .layout { display: grid; grid-template-columns: 180px minmax(0, 1fr); gap: 32px; align-items: start; }
  nav { display: grid; gap: 2px; position: sticky; top: 0; }
  nav button {
    display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 10px; border: 0; border-radius: 8px;
    background: none; color: var(--muted); text-align: left; cursor: pointer; transition: background 120ms var(--ease), color 120ms;
  }
  nav button:hover { background: var(--surface); color: var(--ink); }
  nav button[aria-current='page'] { background: var(--surface-2); color: var(--ink); }
  nav button[aria-current='page'] :global(svg) { color: var(--primary); }
  nav .danger-link { margin-top: 12px; }
  nav .danger-link[aria-current='page'] :global(svg) { color: var(--err); }

  .content :global(h2) { font-size: 1.125rem; }
  .content :global(.lead) { color: var(--muted); margin: 2px 0 18px; }
  .content :global(h3) { font-size: 0.89rem; font-weight: 500; color: var(--muted); margin: 22px 0 8px; }
  .content :global(h3:first-of-type) { margin-top: 0; }

  @media (max-width: 760px) {
    .layout { grid-template-columns: 1fr; gap: 16px; }
    nav { position: static; display: flex; overflow-x: auto; }
    nav .danger-link { margin-top: 0; }
  }
</style>

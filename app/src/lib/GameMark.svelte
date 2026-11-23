<script lang="ts">
  import type { Game } from './api'
  let { game, size = 40 }: { game: Game; size?: number } = $props()
</script>

<!-- Minecraft : bloc d'herbe dessiné pour Foyer (pas le logo officiel). FiveM : logo de simple-icons (CC0), marque de Cfx.re. -->
<span class="mark {game}" style:width="{size}px" style:height="{size}px" aria-hidden="true">
  {#if game === 'minecraft'}
    <svg viewBox="0 0 24 24" width={size * 0.62} height={size * 0.62}>
      <path d="M12 2 22 7 12 12 2 7Z" fill="#7cc84a" />
      <path d="M2 7 12 12V22L2 17Z" fill="#8a5a32" />
      <path d="M12 12 22 7V17L12 22Z" fill="#6a4426" />
      <path d="M2 7 12 12V14.6L2 9.6Z" fill="#5fa832" />
      <path d="M12 12 22 7V9.6L12 14.6Z" fill="#4b8a26" />
      <path d="M4 11.2 6 12.2V13.6L4 12.6ZM8 14.6 10 15.6V17L8 16ZM5 16 7 17V18.4L5 17.4Z" fill="#a8774a" />
      <path d="M15 15.5 17 14.5V15.9L15 16.9ZM19 12.5 21 11.5V12.9L19 13.9ZM17 18.5 19 17.5V18.9L17 19.9Z" fill="#553519" />
      <path d="M8 6 10 7 8 8 6 7ZM14 4.5 16 5.5 14 6.5 12 5.5Z" fill="#9ad866" />
    </svg>
  {:else if game === 'fivem'}
    <svg viewBox="0 0 24 24" width={size * 0.56} height={size * 0.56}><path fill="currentColor" d="M22.4 24h-5.225c-.117 0-.455-1.127-1.026-3.375-1.982-6.909-3.124-10.946-3.417-12.12l3.37-3.325h.099c.454 1.42 2.554 7.676 6.299 18.768ZM12.342 7.084h-.048a3.382 3.385 0 0 1-.098-.492v-.098a102.619 102.715 0 0 1 3.272-3.275c.13.196.196.356.196.491v.05a140.694 140.826 0 0 1-3.322 3.324ZM5.994 10.9h-.05c.67-2.12 1.076-3.209 1.223-3.275L14.492.343c.08 0 .258.524.533 1.562zm1.37-4.014h-.05C8.813 2.342 9.612.048 9.71 0h4.495v.05a664.971 664.971 0 0 1-6.841 6.839Zm-2.69 7.874h-.05c.166-.798.554-1.418 1.174-1.855a312.918 313.213 0 0 1 5.71-5.717h.05c-.117.672-.375 1.175-.781 1.52zM1.598 24l-.098-.05c1.399-4.172 2.148-6.322 2.248-6.45l6.74-6.694v.05C10.232 11.88 8.974 16.263 6.73 24Z" /></svg>
  {:else}
    <strong style:font-size="{size * 0.42}px">{game[0].toUpperCase()}</strong>
  {/if}
</span>

<style>
  .mark { background: var(--muted, oklch(0.3 0.02 60)); font-family: Unbounded, sans-serif; display: grid; place-items: center; border-radius: 10px; flex: none; }
  .minecraft { background: oklch(0.3 0.05 140); box-shadow: inset 0 0 0 1px oklch(0.42 0.07 140); }
  .valheim { background: oklch(0.28 0.03 240); color: oklch(0.85 0.04 240); font-family: Unbounded, sans-serif; }
  .terraria { background: oklch(0.3 0.06 150); color: oklch(0.88 0.1 140); font-family: Unbounded, sans-serif; }
  .fivem { background: oklch(0.27 0.06 10); color: #f40552; box-shadow: inset 0 0 0 1px oklch(0.4 0.1 10); }
</style>

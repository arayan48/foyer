<script lang="ts">
  import { Box, CarFront } from 'lucide-svelte'
  import type { Game } from './api'
  let { game, size = 40 }: { game: Game; size?: number } = $props()
</script>

<span class="mark {game}" style:width="{size}px" style:height="{size}px" aria-hidden="true">
  {#if game === 'minecraft'}<Box size={size * 0.5} />{:else}<CarFront size={size * 0.5} />{/if}
</span>

<style>
  .mark { display: grid; place-items: center; border-radius: 10px; flex: none; }
  .minecraft { background: oklch(0.4 0.09 140); color: oklch(0.92 0.08 140); }
  .fivem { background: oklch(0.42 0.11 55); color: oklch(0.94 0.07 70); }
</style>

<script lang="ts">
  let { size = 28 }: { size?: number } = $props()
</script>

<!-- marque Foyer : une flamme dans une tuile -->
<svg viewBox="0 0 32 32" width={size} height={size} aria-hidden="true">
  <defs>
    <linearGradient id="foyer-bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ffb26b" /><stop offset="1" stop-color="#e8643a" /></linearGradient>
  </defs>
  <rect width="32" height="32" rx="9" fill="url(#foyer-bg)" />
  <path d="M16.4 6.5c.5 3.2 4.6 5 4.6 10a5 5 0 0 1-10 0c0-2.2 1.2-3.7 2.4-4.6.1 1.6.8 2.7 1.9 3.1-.6-3.3.3-6.4 1.1-8.5Z" fill="#2a1408" opacity=".88" />
  <path d="M16 18.2c1.3 1 2.2 2 2.2 3.3a2.2 2.2 0 0 1-4.4 0c0-1.3.9-2.3 2.2-3.3Z" fill="#ffd9a8" />
</svg>

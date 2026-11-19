<script lang="ts">
  let { size = 28 }: { size?: number } = $props()
</script>

<!-- marque Foyer : le toit et la tuile, sur fond encre (comme l'icône de l'appli) -->
<svg viewBox="0 0 100 100" width={size} height={size} aria-hidden="true">
  <rect width="100" height="100" rx="24" fill="#201e1d" />
  <g transform="translate(20 20) scale(0.6)">
    <path d="M21 90V39A18 18 0 0 1 39 21H90" fill="none" stroke="#f5ead8" stroke-width="22" />
    <rect x="46" y="46" width="44" height="44" rx="12" fill="#c67139" />
  </g>
</svg>

<!-- Tile of an event or an exercise in Program: it opens its edit dialog, the pen telling it can be edited. -->
<template>
  <button class="tile">
    <slot />
    <span class="tile-pen" aria-hidden="true">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
      </svg>
    </span>
  </button>
</template>

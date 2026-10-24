<!-- Tile of an event or an exercise in Program: it opens its edit dialog, the pen telling it can be edited. -->
<script setup>
import PenIcon from './PenIcon.vue';
</script>

<template>
  <button class="tile">
    <slot />
    <span class="tile-pen"><PenIcon /></span>
  </button>
</template>

<script setup>
import { vTooltip } from 'floating-vue';

// Infobulle d'un contrôle (souvent la raison de sa désactivation), portée par un conteneur :
// un contrôle désactivé ne reçoit pas le survol. error la borde de rouge.
defineProps({ tip: String, error: Boolean });
</script>

<template>
  <span v-tooltip="{ content: tip, disabled: !tip, placement: 'top', popperClass: error ? 'error' : '' }" class="tip"><slot /></span>
</template>

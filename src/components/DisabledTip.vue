<script setup>
import { vTooltip } from '../tooltip.js';

// Tooltip of a control (often the reason it is disabled), carried by a container:
// a disabled control does not receive hover. error gives it a red border.
defineProps({ tip: String, error: Boolean });
</script>

<template>
  <span v-tooltip="{ content: tip, disabled: !tip, placement: 'top', popperClass: error ? 'error' : '' }" class="tip"><slot /></span>
</template>

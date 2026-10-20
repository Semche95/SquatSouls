<script setup>
import { vTooltip } from '../ui/tooltip.js';

// Stopwatch of a timed exercise, in ember like the timer seal in play; `label` tells how it is timed.
defineProps({ label: { type: String, required: true } });
</script>

<template>
  <span v-tooltip="{ content: label, placement: 'top' }" class="timer-icon" role="img" :aria-label="label">
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="14" r="8" />
      <path d="M12 14V10M10 2h4M12 2v4" />
    </svg>
  </span>
</template>

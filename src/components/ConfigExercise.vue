<script setup>
import { computed } from 'vue';
import { t } from '../i18n/i18n.js';
import { timerSummary } from '../config/itemSummary.js';
import { vTooltip } from '../ui/tooltip.js';
import ProgramTile from './ProgramTile.vue';
import TimerIcon from './TimerIcon.vue';

// Tile of an exercise: its switch, its name, then its range, its unit, its timer and its own recovery.
// A click outside the switch opens ExerciseDialog, through the `edit` event; the switch is the `enabled` model.
const props = defineProps({ ex: { type: Object, required: true } });
defineEmits(['edit']);
const enabled = defineModel('enabled', { type: Boolean, required: true });

const timedTip = computed(() => timerSummary(props.ex));
</script>

<template>
  <div class="tile-slot" :class="{ 'cfg-off': !enabled }">
    <ProgramTile @click="$emit('edit')">
      <strong class="tile-name">{{ ex.name.trim() || t('common.unnamed') }}</strong>
      <span class="tile-foot">
        <span class="tile-count">{{ t('config.exercises.rangeValue', { min: ex.min, max: ex.max }) }}</span> <span class="tile-meta">{{ ex.unit }} <TimerIcon v-if="timedTip" :label="timedTip" /></span>
      </span>
      <span v-if="ex.recovery !== undefined" class="tile-meta">{{ t('config.exercises.recoverySummary', { n: ex.recovery }) }}</span>
    </ProgramTile>
    <input
      v-model="enabled"
      v-tooltip="{ content: t('config.exercises.enabled'), placement: 'top' }"
      type="checkbox"
      class="tile-switch"
      :aria-label="t('config.exercises.enabled')"
    />
  </div>
</template>

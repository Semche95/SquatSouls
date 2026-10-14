<script setup>
import { computed, onMounted, ref } from 'vue';
import { MAX_NAME, MAX_UNIT } from '../configDraft.js';
import { t } from '../i18n.js';
import { vReveal } from '../reveal.js';
import { vTooltip } from '../tooltip.js';
import CountInput from './CountInput.vue';
import DisabledTip from './DisabledTip.vue';

// ex belongs to the draft: it is edited in place. A fresh exercise opens with its name selected.
const props = defineProps({ ex: Object, recovery: Number, removable: Boolean, fresh: Boolean });
defineEmits(['remove']);
// Time units of a timed exercise, after the "not timed" choice.
const TIME_UNITS = ['seconds', 'minutes'];

const open = ref(props.fresh);
const nameInput = ref(null);
onMounted(() => {
  if (!props.fresh) return;
  nameInput.value.focus();
  nameInput.value.select();
});

// Settings only visible once open, recalled in the header: the timer as an icon, the own recovery as text.
const timedTip = computed(() => props.ex.timed && t(`config.exercises.timedSummary.${props.ex.timed}`));
</script>

<template>
  <div class="cfg-event cfg-exercise" :class="{ open, 'cfg-off': !ex.enabled }">
    <div class="cfg-exercise-head">
      <input v-model="ex.enabled" type="checkbox" :aria-label="t('config.exercises.enabled')" />
      <button class="cfg-event-head" :aria-expanded="open" @click="open = !open">
        <strong>{{ ex.name || t('common.unnamed') }}</strong>
        <span v-if="ex.recovery !== undefined" class="muted">{{ t('config.exercises.recoverySummary', { n: ex.recovery }) }}</span>
        <span v-if="timedTip" v-tooltip="{ content: timedTip, placement: 'top' }" class="cfg-timed-icon" role="img" :aria-label="timedTip">⏱&#xFE0E;</span>
      </button>
      <span class="cfg-range">
        <label>
          <span>{{ t('config.exercises.min') }}</span>
          <CountInput v-model="ex.min" :aria-label="t('config.exercises.minimum')" />
        </label>
        <label>
          <span>{{ t('config.exercises.max') }}</span>
          <CountInput v-model="ex.max" :aria-label="t('config.exercises.maximum')" />
        </label>
        <span class="muted">{{ ex.unit }}</span>
      </span>
    </div>
    <template v-if="open">
      <div class="field">
        <span>{{ t('common.name') }}</span>
        <input ref="nameInput" v-model="ex.name" :maxlength="MAX_NAME" :aria-label="t('config.exercises.name')" />
        <DisabledTip :tip="removable ? '' : t('config.exercises.lastExercise')">
          <button class="ghost danger" :disabled="!removable" @click="$emit('remove')">{{ t('common.remove') }}</button>
        </DisabledTip>
      </div>
      <div v-reveal class="cfg-params">
        <label>
          <span>{{ t('config.exercises.unit') }}</span>
          <input v-model="ex.unit" :maxlength="MAX_UNIT" :placeholder="t('config.exercises.unitPlaceholder')" />
        </label>
        <label>
          <span>{{ t('config.exercises.timed') }}</span>
          <select v-model="ex.timed" :aria-label="t('config.exercises.timed')">
            <option :value="undefined">{{ t('config.exercises.notTimed') }}</option>
            <option v-for="unit in TIME_UNITS" :key="unit" :value="unit">{{ t(`config.exercises.${unit}`) }}</option>
          </select>
        </label>
        <label>
          <span>{{ t('config.exercises.recoveryColumn') }}</span>
          <CountInput v-model="ex.recovery" optional :placeholder="String(recovery)" :aria-label="t('config.exercises.recoveryLabel')" />
        </label>
      </div>
    </template>
  </div>
</template>

<script setup>
import { MAX_UNIT } from '../configDraft.js';
import { t } from '../i18n.js';
import CountInput from './CountInput.vue';
import EditDialog from './EditDialog.vue';
import EditRow from './EditRow.vue';

// Edit dialog of an exercise of the draft: included or not, range, unit, timer, own recovery and,
// when timed, the other side. `recovery` is the one of all exercises.
const props = defineProps({ ex: { type: Object, required: true }, recovery: Number, removable: Boolean, fresh: Boolean });
defineEmits(['remove']);
const open = defineModel({ type: Boolean, required: true });

// Timer choices: not timed (no `timed`), then the time units.
const TIMERS = [
  { value: undefined, label: 'notTimed' },
  { value: 'seconds', label: 'seconds' },
  { value: 'minutes', label: 'minutes' },
];

// Only a timed exercise is done on each side.
function setTimed(value) {
  props.ex.timed = value;
  if (!value && props.ex.eachSide) props.ex.eachSide = false;
}
</script>

<template>
  <EditDialog
    v-model="open"
    :item="ex"
    :title="t('config.exercises.title')"
    :name-label="t('config.exercises.name')"
    :name-needed="t('config.exercises.nameNeeded')"
    :remove="t('config.exercises.remove')"
    :remove-tip="removable ? '' : t('config.exercises.lastExercise')"
    :fresh="fresh"
    @remove="$emit('remove')"
  >
    <template #aside>
      <label class="check edit-aside">
        {{ t('config.exercises.included') }}
        <input v-model="ex.enabled" type="checkbox" />
      </label>
    </template>
    <EditRow :label="t('config.exercises.range')">
      <span class="edit-range">
        <span class="muted">{{ t('config.exercises.min') }}</span>
        <CountInput v-model="ex.min" :aria-label="t('config.exercises.minimum')" />
        <span class="muted">{{ t('config.exercises.max') }}</span>
        <CountInput v-model="ex.max" :aria-label="t('config.exercises.maximum')" />
      </span>
    </EditRow>
    <EditRow :label="t('config.exercises.unit')" for-id="ex-unit">
      <input id="ex-unit" v-model="ex.unit" class="edit-unit" :maxlength="MAX_UNIT" :placeholder="t('config.exercises.unitPlaceholder')" />
    </EditRow>
    <EditRow :label="t('config.exercises.timed')">
      <div class="segmented" role="radiogroup" :aria-label="t('config.exercises.timed')">
        <button v-for="timer in TIMERS" :key="timer.label" role="radio" :aria-checked="ex.timed === timer.value" :class="{ active: ex.timed === timer.value }" @click="setTimed(timer.value)">
          {{ t(`config.exercises.${timer.label}`) }}
        </button>
      </div>
    </EditRow>
    <EditRow :label="t('config.exercises.recoveryColumn')" :hint="t('config.exercises.ownRecoveryHint')" for-id="ex-recovery">
      <CountInput id="ex-recovery" v-model="ex.recovery" optional :placeholder="String(recovery)" />
    </EditRow>
    <EditRow v-if="ex.timed" :label="t('config.exercises.eachSide')" :hint="t('config.exercises.eachSideHint')" for-id="ex-each-side">
      <input id="ex-each-side" v-model="ex.eachSide" type="checkbox" />
    </EditRow>
  </EditDialog>
</template>

<script setup>
import { computed } from 'vue';
import { BOUNDS, MAX_UNIT, rangeBounds } from '../configDraft.js';
import { t } from '../i18n.js';
import { shareExercise } from '../share.js';
import { useConfig } from '../composables/useConfig.js';
import CountStepper from './CountStepper.vue';
import DisabledTip from './DisabledTip.vue';
import EditDialog from './EditDialog.vue';
import EditRow from './EditRow.vue';

// Edit dialog of an exercise of the draft: included or not, range, unit, timer, own recovery and,
// when timed, the other side, and Share. `recovery` is the one of all exercises.
const props = defineProps({ ex: { type: Object, required: true }, recovery: Number, fresh: Boolean });
defineEmits(['remove']);
const open = defineModel({ type: Boolean, required: true });

// Timer choices: not timed (no `timed`), then the time units.
const TIMERS = [
  { value: undefined, label: 'notTimed' },
  { value: 'seconds', label: 'seconds' },
  { value: 'minutes', label: 'minutes' },
];

const range = computed(() => rangeBounds(props.ex));

// The saved exercise is shared: a new one has nothing to share before Save.
const { config, copy } = useConfig();
const saved = computed(() => config.value.exercises.find((ex) => ex.id === props.ex.id));

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
    :fresh="fresh"
    @remove="$emit('remove')"
  >
    <template #aside>
      <label class="check edit-aside">
        {{ t('config.exercises.included') }}
        <input v-model="ex.enabled" type="checkbox" />
      </label>
    </template>
    <template #actions>
      <DisabledTip :tip="saved ? '' : t('config.exercises.shareUnsaved')">
        <button class="ghost" :disabled="!saved" @click="copy(shareExercise(saved))">{{ t('common.share') }}</button>
      </DisabledTip>
    </template>
    <EditRow :label="t('config.exercises.range')">
      <span class="edit-range">
        <span class="muted">{{ t('config.exercises.min') }}</span>
        <CountStepper v-model="ex.min" :bounds="range.min" :aria-label="t('config.exercises.minimum')" />
        <span class="muted">{{ t('config.exercises.max') }}</span>
        <CountStepper v-model="ex.max" :bounds="range.max" :aria-label="t('config.exercises.maximum')" />
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
    <EditRow :label="t('config.exercises.recoveryLabel')" :hint="t('config.exercises.ownRecoveryHint')" for-id="ex-recovery">
      <CountStepper id="ex-recovery" v-model="ex.recovery" optional :bounds="BOUNDS.seconds" :base="+recovery || 0" :placeholder="String(recovery)" :unit="t('fields.secondsUnit')" />
    </EditRow>
    <EditRow v-if="ex.timed" :label="t('config.exercises.eachSide')" :hint="t('config.exercises.eachSideHint')" for-id="ex-each-side">
      <input id="ex-each-side" v-model="ex.eachSide" type="checkbox" />
    </EditRow>
  </EditDialog>
</template>

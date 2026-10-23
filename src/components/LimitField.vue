<script setup>
import { computed } from 'vue';
import { t } from '../i18n/i18n.js';
import { BOUNDS } from '../config/configDraft.js';
import { vSelect } from '../ui/select.js';
import { useConfig } from '../composables/useConfig.js';
import CountStepper from './CountStepper.vue';

// Limit of the session, chosen in Play next to the intensity: its kind, then its value once a kind is chosen.
const LIMIT_KINDS = ['none', 'exercises', 'playTime', 'trainingTime'];
// Value a kind starts from when chosen.
const START = { exercises: 20, playTime: 60, trainingTime: 30 };

const { limitKind, limitValue } = useConfig();
const kind = computed({
  get: () => limitKind.value,
  set: (value) => {
    limitKind.value = value;
    limitValue.value = START[value];
  },
});
</script>

<template>
  <div class="field play-limit">
    <label for="play-limit-kind">{{ t('play.limit') }}</label>
    <div class="row">
      <select id="play-limit-kind" v-model="kind" v-select>
        <option v-for="k in LIMIT_KINDS" :key="k" :value="k">{{ t(`play.limitKinds.${k}`) }}</option>
      </select>
      <CountStepper
        v-if="kind !== 'none'"
        id="play-limit"
        v-model="limitValue"
        :bounds="BOUNDS.amount"
        :unit="kind === 'exercises' ? '' : t('play.minutes')"
        :aria-label="t(`play.limitKinds.${kind}`)"
      />
    </div>
  </div>
</template>

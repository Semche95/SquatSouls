<script setup>
import { computed } from 'vue';
import { t } from '../i18n/i18n.js';
import { useSessionLimit } from '../composables/useSessionLimit.js';
import DisabledTip from './DisabledTip.vue';

// Progress of the session towards its limit, next to the jokers; the tooltip says what it counts.
const { limit, used, max, reached } = useSessionLimit();

const text = computed(() =>
  limit.value.kind === 'exercises'
    ? t('play.limitExercises', { used: used.value, n: max.value }, max.value)
    : t('play.limitTime', { used: Math.floor(used.value / 60), max: limit.value.value }),
);
</script>

<template>
  <DisabledTip v-if="limit" :tip="reached ? t('play.sessionLimitTip') : t(`play.limitTips.${limit.kind}`, { program: t('app.tabs.program') })">
    <span class="session-limit" :class="{ full: reached }">
      {{ text }}
      <span class="gauge"><span :style="{ width: `${Math.min(100, (100 * used) / max)}%` }"></span></span>
    </span>
  </DisabledTip>
</template>

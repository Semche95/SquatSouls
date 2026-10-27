<script setup>
import { daySummary, formatDay, formatDuration, formatTime, sessionSummary } from '../i18n/format.js';
import { t } from '../i18n/i18n.js';
import { useCopy } from '../composables/useCopy.js';

defineProps({ days: { type: Array, required: true } });

const { copied, copy } = useCopy();
</script>

<template>
  <ul class="days">
    <li v-for="d in days" :key="d.date" :class="{ off: !d.sets }">
      <span class="day">
        {{ formatDay(d.date) }}
        <button v-if="d.sessions?.length > 1" class="link" @click="copy(d.date, daySummary(d))">
          {{ t(copied === d.date ? 'history.copiedSummary' : 'history.copyDay') }}
        </button>
      </span>
      <span v-if="!d.sets" class="muted">{{ t('history.rest') }}</span>
      <span v-else class="day-body">
        <span v-for="se in d.sessions" :key="se.start + se.game" class="day-session">
          <span class="day-games">
            {{ formatTime(se.start) }} · {{ se.game }} · {{ formatDuration(se.duration) }}
            <button class="link" @click="copy(se.start + se.game, sessionSummary(se))">
              {{ t(copied === se.start + se.game ? 'history.copiedSummary' : 'history.copy') }}
            </button>
          </span>
          <span class="day-totals">
            <span v-for="total in se.totals" :key="total.exercise + total.unit" class="chip">
              {{ total.exercise }} {{ total.amount }} {{ total.unit }}
            </span>
          </span>
        </span>
      </span>
    </li>
  </ul>
</template>

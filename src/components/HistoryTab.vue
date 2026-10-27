<script setup>
import { computed, onMounted, ref } from 'vue';
import { formatTime, sessionSummary } from '../i18n/format.js';
import { t } from '../i18n/i18n.js';
import { useApi } from '../composables/useApi.js';
import { useConfig } from '../composables/useConfig.js';
import { useCopy } from '../composables/useCopy.js';
import { useSession } from '../composables/useSession.js';
import FullHistory from './FullHistory.vue';
import HistoryDays from './HistoryDays.vue';

const { call } = useApi();
const { config } = useConfig();
const { session } = useSession();
const { copied, copy } = useCopy();

const history = ref([]);
// Full history, shown large over the History tab.
const full = ref(false);
// Days with at least one set, ever: a count that only goes up.
const activeDays = ref(0);
// Game set on the session's first event, absent before.
const gameName = computed(() => config.value?.games.find((g) => g.id === session.value.gameId)?.name);

// The current session lasts until now, without the time the app was closed; its events follow the order of the
// game's events.
function currentSummary() {
  const s = session.value;
  const events = config.value.games.find((g) => g.id === s.gameId)?.events ?? [];
  return sessionSummary({
    game: gameName.value,
    duration: Math.floor(Date.now() / 1000) - s.countedFrom,
    events: events.flatMap((ev) => {
      const count = s.countedTriggers.find((tr) => tr.gameId === s.gameId && tr.eventId === ev.id)?.count;
      return count ? [{ event: ev.name, count }] : [];
    }),
    totals: s.totals,
    jokers: { earned: s.jokersEarned, spent: s.jokersSpent },
  });
}

onMounted(async () => {
  try {
    [history.value, activeDays.value] = await Promise.all([call('get_history', { days: 7 }), call('count_active_days')]);
  } catch {
    history.value = [];
  }
});
</script>

<template>
  <main class="panel summary">
    <section class="group current">
      <header class="group-head">
        <h2 class="group-title">{{ t('history.current') }}</h2>
        <span class="muted"><template v-if="gameName">{{ gameName }} · </template>{{ session.startedAt ? t('history.since', { time: formatTime(session.startedAt) }) : t('history.notStarted') }}</span>
        <button v-if="session.sets" class="ghost" @click="copy('current', currentSummary())">
          {{ t(copied === 'current' ? 'history.copiedSummary' : 'history.copy') }}
        </button>
      </header>
      <p v-if="!session.sets" class="empty">{{ t(session.startedAt ? 'history.underway' : 'history.waiting') }}</p>
      <div v-else class="session-detail">
        <div class="session-totals">
          <h2>{{ t('history.totals') }}</h2>
          <ul class="totals">
            <li v-for="total in session.totals" :key="total.exercise + total.unit">
              <span class="total-amount">{{ total.amount }} <small>{{ total.unit }}</small></span>
              <strong>{{ total.exercise }}</strong>
              <span class="muted">{{ t('history.sets', total.sets) }}</span>
            </li>
          </ul>
        </div>
        <div>
          <h2>{{ t('common.setsDone') }}</h2>
          <ul class="log">
            <li v-for="(e, i) in session.log" :key="i">
              <span class="muted">{{ formatTime(e.at) }}</span>
              <span>{{ e.amount }} {{ e.unit }} · {{ e.exercise }}</span>
              <span class="muted">{{ e.event }}</span>
            </li>
          </ul>
        </div>
      </div>
      <div v-if="session.jokersEarned || session.jokersSpent">
        <h2>{{ t('common.jokers') }}</h2>
        <p class="muted">{{ t('history.earned', session.jokersEarned) }} · {{ t('history.spent', session.jokersSpent) }}</p>
      </div>
    </section>

    <section class="group">
      <header class="group-head">
        <h2 class="group-title">{{ t('history.lastDays') }}</h2>
        <span class="muted">{{ t('history.bySession') }} · {{ t('history.activeDays', activeDays) }}</span>
        <button class="ghost" @click="full = true">{{ t('history.seeAll') }}</button>
      </header>
      <HistoryDays :days="history" />
    </section>
    <FullHistory v-if="full" :active-days="activeDays" @close="full = false" />
  </main>
</template>

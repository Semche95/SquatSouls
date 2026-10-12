<script setup>
import { computed, onMounted, ref } from 'vue';
import { formatDay, formatDuration, formatTime, sessionSummary } from '../format.js';
import { t } from '../i18n.js';
import { useApi } from '../composables/useApi.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useKeydown } from '../composables/useKeydown.js';

// How long a copy button reads "Copied".
const COPIED_MS = 2000;

const { error, call } = useApi();
const { config } = useConfig();
const { session } = useSession();

const history = ref([]);
// Full history, shown large over the Summary tab (null when closed).
const fullHistory = ref(null);
// Game set on the session's first event, absent before.
const gameName = computed(() => config.value?.games.find((g) => g.id === session.value.gameId)?.name);
const jokersSpent = computed(() => session.value.skipped.reduce((sum, k) => sum + k.jokers, 0));
// Copy button that last copied its summary, '' after a while.
const copied = ref('');
let copiedTimer;

// The current session lasts until now; its events follow the order of the game's events.
function currentSummary() {
  const s = session.value;
  const events = config.value.games.find((g) => g.id === s.gameId)?.events ?? [];
  return sessionSummary({
    game: gameName.value,
    duration: Math.floor(Date.now() / 1000) - s.startedAt,
    events: events.flatMap((ev) => {
      const count = s.triggers.find((tr) => tr.gameId === s.gameId && tr.eventId === ev.id)?.count;
      return count ? [{ event: ev.name, count }] : [];
    }),
    totals: s.totals,
    jokers: { earned: s.jokersEarned, spent: jokersSpent.value },
  });
}

async function copySummary(key, text) {
  try {
    await navigator.clipboard.writeText(text);
    copied.value = key;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => (copied.value = ''), COPIED_MS);
  } catch (e) {
    error.value = t('common.copyFailed', { error: String(e) });
  }
}

async function openFullHistory() {
  try {
    fullHistory.value = await call('get_full_history');
  } catch {
    /* error already shown */
  }
}

useKeydown((e) => {
  if (fullHistory.value && e.code === 'Escape') fullHistory.value = null;
});

onMounted(async () => {
  try {
    history.value = await call('get_history', { days: 7 });
  } catch {
    history.value = [];
  }
});
</script>

<template>
  <main class="panel summary">
    <section class="group current">
      <header class="group-head">
        <h2 class="group-title">{{ t('summary.current') }}</h2>
        <span class="muted"><template v-if="gameName">{{ gameName }} · </template>{{ session.startedAt ? t('summary.since', { time: formatTime(session.startedAt) }) : t('summary.notStarted') }}</span>
        <button v-if="session.sets" class="ghost" @click="copySummary('current', currentSummary())">
          {{ t(copied === 'current' ? 'summary.copiedSummary' : 'summary.copy') }}
        </button>
      </header>
      <p v-if="!session.sets" class="empty">{{ t('summary.noSets') }}</p>
      <div v-else class="session-detail">
        <div class="session-totals">
          <h2>{{ t('summary.totals') }}</h2>
          <ul class="totals">
            <li v-for="total in session.totals" :key="total.exercise + total.unit">
              <span class="total-amount">{{ total.amount }} <small>{{ total.unit }}</small></span>
              <strong>{{ total.exercise }}</strong>
              <span class="muted">{{ t('summary.sets', total.sets) }}</span>
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
      <div v-if="session.jokersEarned || session.skipped.length">
        <h2>{{ t('common.jokers') }}</h2>
        <p class="muted">{{ t('summary.earned', session.jokersEarned) }} · {{ t('summary.spent', jokersSpent) }}</p>
        <ul v-if="session.skipped.length" class="log">
          <li v-for="(k, i) in session.skipped" :key="i">
            <span class="muted">{{ formatTime(k.at) }}</span>
            <span>{{ k.items.map((it) => `${it.amount} ${it.unit} · ${it.name}`).join(', ') }}</span>
            <span class="muted">{{ k.event }} ({{ t(k.halved ? 'common.halved' : 'summary.skipped') }})</span>
          </li>
        </ul>
      </div>
    </section>

    <section class="group" :class="{ full: fullHistory }">
      <header class="group-head">
        <h2 class="group-title">{{ t(fullHistory ? 'summary.fullHistory' : 'summary.history') }}</h2>
        <span v-if="fullHistory" class="muted">{{ t('summary.activeDays', fullHistory.length) }}</span>
        <span v-else class="muted">{{ t('summary.lastDays') }}</span>
        <button v-if="fullHistory" class="ghost" @click="fullHistory = null">{{ t('summary.close') }}</button>
        <button v-else class="ghost" @click="openFullHistory">{{ t('summary.seeAll') }}</button>
      </header>
      <p v-if="fullHistory && !fullHistory.length" class="muted">{{ t('summary.noHistory') }}</p>
      <ul class="days">
        <li v-for="d in fullHistory ?? history" :key="d.date" :class="{ off: !d.sets }">
          <span class="day">{{ formatDay(d.date) }}</span>
          <span v-if="!d.sets" class="muted">{{ t('summary.rest') }}</span>
          <span v-else class="day-body">
            <span v-for="se in d.sessions" :key="se.start + se.game" class="day-session">
              <span class="day-games">
                {{ formatTime(se.start) }} · {{ se.game }} · {{ formatDuration(se.duration) }}
                <button class="link" @click="copySummary(se.start + se.game, sessionSummary(se))">
                  {{ t(copied === se.start + se.game ? 'summary.copiedSummary' : 'summary.copy') }}
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
    </section>
  </main>
</template>

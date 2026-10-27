<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { byName } from '../config/configDraft.js';
import { formatMonth } from '../i18n/format.js';
import { t } from '../i18n/i18n.js';
import { useApi } from '../composables/useApi.js';
import { useKeydown } from '../composables/useKeydown.js';
import { vSelect } from '../ui/select.js';
import HistoryDays from './HistoryDays.vue';

// Days of the full history loaded at a time.
const FULL_HISTORY_DAYS = 60;

const emit = defineEmits(['close']);

const { call } = useApi();

// Game whose sessions are shown, '' for every game.
const game = ref('');
// Month (YYYY-MM) the days start from, '' for the most recent.
const month = ref('');
// Games of the history, as they were named then.
const games = ref([]);
// Months in which a session of the game started, most recent first.
const months = ref([]);
// Days with at least one set of the game, ever.
const activeDays = ref(0);
// Days shown, null until the first page is loaded.
const days = ref(null);
const list = ref(null);
// End of the list: the older days load once it comes into view.
const end = ref(null);
// Page of days being loaded, null when none is.
let loading = null;
const observer = new IntersectionObserver(([entry]) => {
  if (entry.isIntersecting && !loading) loadDays();
});

// First day after the month chosen, the days shown being those before it.
const monthEnd = computed(() => {
  if (!month.value) return null;
  const [y, m] = month.value.split('-').map(Number);
  return m === 12 ? `${y + 1}-01-01` : `${y}-${String(m + 1).padStart(2, '0')}-01`;
});

// Loads the days older than those shown, or from the start of the list again with `reset`.
async function loadDays(reset = false) {
  const shown = reset ? [] : (days.value ?? []);
  const before = shown.at(-1)?.date ?? monthEnd.value;
  const page = call('get_full_history', { before, days: FULL_HISTORY_DAYS, game: game.value || null });
  loading = page;
  try {
    const loaded = await page;
    // Closed or filtered again meanwhile.
    if (loading !== page) return;
    days.value = [...shown, ...loaded];
    await nextTick();
    if (reset) list.value.scrollTop = 0;
    // Observing again checks right away whether the end is still in view.
    observer.disconnect();
    if (loaded.length === FULL_HISTORY_DAYS) observer.observe(end.value);
  } catch {
    /* error already shown */
  } finally {
    if (loading === page) loading = null;
  }
}

// The months and the count of the game chosen; a month it was not played in gives way to the most recent.
watch(game, async (chosen) => {
  try {
    const [m, count] = await Promise.all([call('get_history_months', { game: chosen || null }), call('count_active_days', { game: chosen || null })]);
    if (game.value !== chosen) return;
    [months.value, activeDays.value] = [m, count];
  } catch {
    /* error already shown */
  }
  // Changing the month loads its days.
  if (month.value && !months.value.includes(month.value)) month.value = '';
  else loadDays(true);
});

watch(month, () => loadDays(true));

useKeydown((e) => {
  if (e.code === 'Escape') emit('close');
});

onMounted(async () => {
  loadDays();
  try {
    const [names, m, count] = await Promise.all([call('get_history_games'), call('get_history_months', { game: null }), call('count_active_days', { game: null })]);
    games.value = byName(names.map((name) => ({ name }))).map((g) => g.name);
    [months.value, activeDays.value] = [m, count];
  } catch {
    /* error already shown */
  }
});

onUnmounted(() => {
  loading = null;
  observer.disconnect();
});
</script>

<template>
  <section v-if="days" ref="list" class="group full">
    <div class="full-head">
      <header class="group-head">
        <h2 class="group-title">{{ t('history.fullHistory') }}</h2>
        <span class="muted">{{ t('history.activeDays', activeDays) }}</span>
        <button class="ghost" @click="emit('close')">{{ t('history.close') }}</button>
      </header>
      <div v-if="months.length" class="history-filters">
        <label for="history-month" class="sr-only">{{ t('history.month') }}</label>
        <select id="history-month" v-model="month" v-select class="month-select">
          <option value="">{{ t('history.latest') }}</option>
          <option v-for="m in months" :key="m" :value="m">{{ formatMonth(m) }}</option>
        </select>
        <label for="history-game" class="sr-only">{{ t('history.game') }}</label>
        <select id="history-game" v-model="game" v-select>
          <option value="">{{ t('history.allGames') }}</option>
          <option v-for="name in games" :key="name" :value="name">{{ name }}</option>
        </select>
      </div>
    </div>
    <p v-if="!days.length" class="muted">{{ t('history.noHistory') }}</p>
    <HistoryDays :days="days" />
    <div ref="end"></div>
  </section>
</template>

<script setup>
import { nextTick, onMounted, onUnmounted, ref } from 'vue';
import { t } from '../i18n/i18n.js';
import { useApi } from '../composables/useApi.js';
import { useKeydown } from '../composables/useKeydown.js';
import HistoryDays from './HistoryDays.vue';

// Days of the full history loaded at a time.
const FULL_HISTORY_DAYS = 60;

defineProps({ activeDays: { type: Number, required: true } });
const emit = defineEmits(['close']);

const { call } = useApi();

// Days shown, null until the first page is loaded.
const days = ref(null);
// End of the list: the older days load once it comes into view.
const end = ref(null);
// Page of days being loaded, null when none is.
let loading = null;
const observer = new IntersectionObserver(([entry]) => {
  if (entry.isIntersecting && !loading) loadDays();
});

// Loads the days older than those shown.
async function loadDays() {
  const shown = days.value ?? [];
  const page = call('get_full_history', { before: shown.at(-1)?.date ?? null, days: FULL_HISTORY_DAYS });
  loading = page;
  try {
    const loaded = await page;
    // Closed meanwhile.
    if (loading !== page) return;
    days.value = [...shown, ...loaded];
    await nextTick();
    // Observing again checks right away whether the end is still in view.
    observer.disconnect();
    if (loaded.length === FULL_HISTORY_DAYS) observer.observe(end.value);
  } catch {
    /* error already shown */
  } finally {
    if (loading === page) loading = null;
  }
}

useKeydown((e) => {
  if (e.code === 'Escape') emit('close');
});

onMounted(loadDays);

onUnmounted(() => {
  loading = null;
  observer.disconnect();
});
</script>

<template>
  <section v-if="days" class="group full">
    <header class="group-head">
      <h2 class="group-title">{{ t('history.fullHistory') }}</h2>
      <span class="muted">{{ t('history.activeDays', activeDays) }}</span>
      <button class="ghost" @click="emit('close')">{{ t('history.close') }}</button>
    </header>
    <p v-if="!days.length" class="muted">{{ t('history.noHistory') }}</p>
    <HistoryDays :days="days" />
    <div ref="end"></div>
  </section>
</template>

import { computed, onScopeDispose, ref, watch } from 'vue';
import { defineShared } from './shared.js';

// Interval between two readings of the play time, against which a limit in play time applies.
const TICK_MS = 1000;

// Progress of the session towards its limit, as the backend counts it: the exercises asked, the play time without
// the time the app was closed, or the estimated training time. The backend refuses the sets beyond it.
export const [provideSessionLimit, useSessionLimit] = defineShared(({ session }) => {
  const limit = computed(() => session.value.limit ?? null);
  const playTime = computed(() => limit.value?.kind === 'playTime' && !!session.value.startedAt);
  const now = ref(Date.now());
  let ticker;
  watch(
    playTime,
    (on) => {
      clearInterval(ticker);
      now.value = Date.now();
      if (on) ticker = setInterval(() => (now.value = Date.now()), TICK_MS);
    },
    { immediate: true },
  );
  onScopeDispose(() => clearInterval(ticker));

  // Exercises asked, or seconds, and the limit in the same unit.
  const used = computed(() => {
    const s = session.value;
    switch (limit.value?.kind) {
      case 'exercises':
        return s.asked;
      case 'playTime':
        return playTime.value ? Math.max(0, Math.floor(now.value / 1000) - s.countedFrom) : 0;
      case 'trainingTime':
        return s.askedTime;
      default:
        return 0;
    }
  });
  const max = computed(() => (limit.value?.kind === 'exercises' ? limit.value.value : (limit.value?.value ?? 0) * 60));
  const reached = computed(() => !!limit.value && (used.value >= max.value || (limit.value.kind === 'trainingTime' && session.value.limitFull)));

  return { limit, used, max, reached };
});

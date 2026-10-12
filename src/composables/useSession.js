import { computed, onScopeDispose, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { clone } from '../configDraft.js';
import { t, tm } from '../i18n.js';
import { defineShared } from './shared.js';

// Refresh rate of the pause countdown.
const PAUSE_TICK_MS = 250;

export const [provideSession, useSession] = defineShared(({ call }, { config, draft, game, gameId, intensity, store }) => {
  const session = ref({ pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, jokersSpent: 0 });
  const notice = ref('');
  // Encouragement shown once the session ends, until the next one starts: index in `session.farewells`.
  const farewell = ref(-1);
  watch(() => session.value.startedAt, (started) => {
    if (started) farewell.value = -1;
  });
  // Pause after an exercise is done: no other can be marked done before it ends.
  const pauseEnd = ref(0);
  const now = ref(0);
  let ticker;
  // Set whose exercise started the pause: dismissing it stops the pause.
  let pauseDrawId = 0;
  const pauseLeft = computed(() => Math.max(0, Math.ceil((pauseEnd.value - now.value) / 1000)));

  // Exercise-specific pause, otherwise the global pause.
  function pauseAfter(item) {
    const { exercises, pause } = config.value;
    return exercises.find((ex) => ex.id === item.exerciseId)?.pause ?? pause;
  }

  function startPause(seconds, drawId) {
    skipPause();
    if (!seconds) return;
    pauseDrawId = drawId;
    now.value = Date.now();
    pauseEnd.value = now.value + seconds * 1000;
    ticker = setInterval(() => {
      now.value = Date.now();
      if (!pauseLeft.value) skipPause();
    }, PAUSE_TICK_MS);
  }

  function skipPause() {
    clearInterval(ticker);
    pauseEnd.value = 0;
    pauseDrawId = 0;
  }

  onScopeDispose(skipPause);

  watch(session, (s) => {
    if (s.gameId) {
      gameId.value = s.gameId;
      intensity.value = s.intensity;
    }
  });

  async function reload() {
    session.value = await invoke('get_session');
  }

  // Configuration saved right away and applied to the session; false if the backend refuses it.
  async function saveNow(cfg) {
    if (!(await store(cfg))) return false;
    await reload();
    return true;
  }

  // Confirmed action: `change` modifies the saved configuration, saved right away, then the
  // draft, which keeps its other pending changes; false if the backend refuses.
  async function saveChange(change) {
    const cfg = clone(config.value);
    change(cfg);
    if (!(await saveNow(cfg))) return false;
    change(draft.value);
    return true;
  }

  async function trigger(ev) {
    notice.value = '';
    try {
      const draw = await call('draw', { gameId: game.value.id, eventId: ev.id, intensity: intensity.value });
      if (draw.capped) {
        notice.value = t('play.capped', { event: draw.eventName, max: draw.maxPerSession });
      }
      session.value = await call('get_session');
    } catch {
      /* error already shown */
    }
  }

  // Done, joker or dismissal of a set: on failure, the queue is resynced.
  async function act(command, draw, args = {}) {
    try {
      session.value = await call(command, { drawId: draw.id, ...args });
      return true;
    } catch {
      await reload();
      return false;
    }
  }

  // Started set whose dismissal awaits confirmation: its done exercises will not count.
  const confirmDismiss = ref(0);

  function dismiss(draw) {
    if (draw.items.some((it) => it.done)) confirmDismiss.value = draw.id;
    else dismissNow(draw);
  }

  async function dismissNow(draw) {
    confirmDismiss.value = 0;
    if ((await act('dismiss', draw)) && draw.id === pauseDrawId) skipPause();
  }

  // Exercises of the set still to do.
  const remaining = (draw) => draw.items.filter((it) => !it.done);
  // Joker cost absent when the action is no longer possible.
  const canAfford = (cost) => cost != null && session.value.jokers >= cost;
  const setSkipCost = (draw) => remaining(draw).reduce((sum, it) => sum + it.skipCost, 0);

  async function complete(draw, item) {
    if (pauseLeft.value) return;
    if (await act('complete_exercise', draw, { exerciseId: item.exerciseId })) startPause(pauseAfter(item), draw.id);
  }

  function halve(draw, item) {
    if (canAfford(item.halveCost)) act('halve', draw, { exerciseId: item.exerciseId });
  }

  function skipExercise(draw, item) {
    if (canAfford(item.skipCost)) act('skip_exercise', draw, { exerciseId: item.exerciseId });
  }

  function skipSet(draw) {
    if (canAfford(setSkipCost(draw))) act('skip_set', draw);
  }

  async function startSession() {
    try {
      session.value = await call('start_session');
    } catch {
      /* error already shown */
    }
  }

  async function endSession() {
    notice.value = '';
    skipPause();
    try {
      session.value = await call('end_session');
      farewell.value = Math.floor(Math.random() * tm('session.farewells').length);
    } catch {
      await reload();
    }
  }

  async function cancelSession() {
    notice.value = '';
    skipPause();
    try {
      session.value = await call('cancel_session');
    } catch {
      /* error already shown */
    }
  }

  return {
    session, notice, farewell, pauseLeft, skipPause, reload, saveNow, saveChange, trigger, act, confirmDismiss, dismiss,
    dismissNow, remaining, canAfford, setSkipCost, complete, halve, skipExercise, skipSet, startSession, endSession, cancelSession,
  };
});

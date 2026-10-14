import { computed, onScopeDispose, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { clone } from '../configDraft.js';
import { t, tm } from '../i18n.js';
import { chime, recoveryEnd, startSignal } from '../sound.js';
import { defineShared } from './shared.js';

// Refresh rate of the countdowns.
const TICK_MS = 250;
// Seconds in each time unit of a timed exercise.
const UNIT_SECONDS = { seconds: 1, minutes: 60 };

// Countdown that can be held then resumed; `onEnd` runs when it reaches 0, right away for 0 s.
function countdown() {
  const end = ref(0);
  const seconds = ref(0);
  // Milliseconds left while held, 0 otherwise.
  const heldMs = ref(0);
  const now = ref(0);
  let ticker;
  let onEnd;
  const msLeft = computed(() => heldMs.value || Math.max(0, end.value - now.value));
  const left = computed(() => Math.ceil(msLeft.value / 1000));
  // Share still to wait, from 1 to 0.
  const share = computed(() => (seconds.value ? msLeft.value / (seconds.value * 1000) : 0));
  const held = computed(() => !!heldMs.value);

  function run(ms) {
    heldMs.value = 0;
    now.value = Date.now();
    end.value = now.value + ms;
    ticker = setInterval(() => {
      now.value = Date.now();
      if (msLeft.value) return;
      const then = onEnd;
      stop();
      then?.();
    }, TICK_MS);
  }

  function start(duration, then) {
    stop();
    if (!duration) return then?.();
    seconds.value = duration;
    onEnd = then;
    run(duration * 1000);
  }

  function stop() {
    clearInterval(ticker);
    end.value = 0;
    seconds.value = 0;
    heldMs.value = 0;
    onEnd = undefined;
  }

  function hold() {
    clearInterval(ticker);
    heldMs.value = msLeft.value;
  }

  function resume() {
    if (heldMs.value) run(heldMs.value);
  }

  return { left, share, held, start, stop, hold, resume };
}

export const [provideSession, useSession] = defineShared(({ call }, { config, draft, game, gameId, intensity, store }) => {
  const session = ref({ pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, jokersSpent: 0 });
  const notice = ref('');
  // Encouragement shown once the session ends, until the next one starts: index in `session.farewells`.
  const farewell = ref(-1);
  watch(() => session.value.startedAt, (started) => {
    if (started) farewell.value = -1;
  });
  // Recovery after an exercise is done: no other can be marked done before it ends.
  const recovery = countdown();
  const { left: recoveryLeft, share: recoveryShare } = recovery;
  // Set whose exercise started the recovery: dismissing it stops the recovery.
  let recoveryDrawId = 0;

  const exerciseOf = (item) => config.value.exercises.find((ex) => ex.id === item.exerciseId);
  // Exercise-specific recovery, otherwise the global recovery.
  const recoveryAfter = (item) => exerciseOf(item)?.recovery ?? config.value.recovery;
  // Time unit of a timed exercise, undefined otherwise.
  const timedUnit = (item) => exerciseOf(item)?.timed;

  function startRecovery(seconds, drawId) {
    recoveryDrawId = drawId;
    recovery.start(seconds, () => {
      recoveryDrawId = 0;
      if (config.value.sound) recoveryEnd();
    });
  }

  function skipRecovery() {
    recovery.stop();
    recoveryDrawId = 0;
  }

  // Timer of a timed exercise: its preparation, then its duration. Like the recovery, it blocks Done.
  const timer = countdown();
  const { left: timerLeft, share: timerShare, held: timerHeld } = timer;
  // Exercise timed, with its set and number, and whether the timer is still in its preparation.
  const timing = ref(null);

  function startTimer(draw, item) {
    const unit = timedUnit(item);
    if (!unit || recoveryLeft.value || timing.value) return;
    timing.value = { drawId: draw.id, exerciseId: item.exerciseId, amount: item.amount, ready: true };
    timer.start(config.value.timerPrep, () => {
      timing.value.ready = false;
      if (config.value.sound) startSignal();
      timer.start(item.amount * UNIT_SECONDS[unit], () => timerEnded(draw, item));
    });
  }

  function timerEnded(draw, item) {
    timing.value = null;
    if (config.value.sound) chime();
    if (config.value.timerAutoDone) complete(draw, item);
  }

  function stopTimer() {
    timer.stop();
    timing.value = null;
  }

  function toggleTimer() {
    if (timerHeld.value) timer.resume();
    else timer.hold();
  }

  onScopeDispose(() => {
    skipRecovery();
    stopTimer();
  });

  watch(session, (s) => {
    if (s.gameId) {
      gameId.value = s.gameId;
      intensity.value = s.intensity;
    }
  });

  // Intensity changed during the session: applied right away to the sets to do.
  watch(intensity, async (value) => {
    if (!session.value.gameId || value === session.value.intensity) return;
    try {
      session.value = await call('set_session_intensity', { intensity: value });
    } catch {
      await reload();
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
    if ((await act('dismiss', draw)) && draw.id === recoveryDrawId) skipRecovery();
  }

  // Exercises of the set still to do.
  const remaining = (draw) => draw.items.filter((it) => !it.done);
  // Joker cost absent when the action is no longer possible.
  const canAfford = (cost) => cost != null && session.value.jokers >= cost;
  const setSkipCost = (draw) => remaining(draw).reduce((sum, it) => sum + it.skipCost, 0);

  // The timer stops once its exercise is no longer the one to do as it was: set dismissed or skipped,
  // exercise halved or skipped, session cancelled.
  watch(session, (s) => {
    const timed = timing.value;
    if (!timed) return;
    const first = s.pending[0];
    const item = first?.id === timed.drawId ? remaining(first)[0] : undefined;
    if (item?.exerciseId !== timed.exerciseId || item.amount !== timed.amount) stopTimer();
  });

  async function complete(draw, item) {
    if (recoveryLeft.value || timing.value) return;
    if (await act('complete_exercise', draw, { exerciseId: item.exerciseId })) startRecovery(recoveryAfter(item), draw.id);
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
    skipRecovery();
    try {
      session.value = await call('end_session');
      farewell.value = Math.floor(Math.random() * tm('session.farewells').length);
    } catch {
      await reload();
    }
  }

  async function cancelSession() {
    notice.value = '';
    skipRecovery();
    try {
      session.value = await call('cancel_session');
    } catch {
      /* error already shown */
    }
  }

  return {
    session, notice, farewell, recoveryLeft, recoveryShare, skipRecovery, timing, timerLeft, timerShare, timerHeld, timedUnit, startTimer, stopTimer,
    toggleTimer, reload, saveNow, saveChange, trigger, act, confirmDismiss, dismiss, dismissNow, remaining, canAfford, setSkipCost,
    complete, halve, skipExercise, skipSet, startSession, endSession, cancelSession,
  };
});

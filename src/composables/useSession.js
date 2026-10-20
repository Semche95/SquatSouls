import { onScopeDispose, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { clone } from '../config/configDraft.js';
import { t, tm } from '../i18n/i18n.js';
import { chime, recoveryEnd, sideSignal, startSignal } from '../session/sound.js';
import { defineShared } from './shared.js';
import { useCountdown } from './useCountdown.js';

// Seconds in each time unit of a timed exercise.
const UNIT_SECONDS = { seconds: 1, minutes: 60 };
// Sides of an exercise timed on each side.
export const SIDES = 2;

export const [provideSession, useSession] = defineShared(({ call }, { config, draft, game, gameId, intensity, noExercise, store }) => {
  const session = ref({ pending: [], sets: 0, totals: [], log: [], triggers: [], countedTriggers: [], jokers: 0, jokersEarned: 0, jokersSpent: 0 });
  const notice = ref('');
  // Encouragement shown once a session ends with exercises done, however it ends (End the session, or a saved
  // configuration it can no longer be played on), until the next one starts: index in `session.farewells`.
  const farewell = ref(-1);
  watch(() => !!session.value.ended?.saved, (saved) => {
    farewell.value = saved ? Math.floor(Math.random() * tm('session.farewells').length) : -1;
  });
  // The recovery stops with the session, however it ends.
  watch(() => session.value.startedAt, (started) => started || skipRecovery());
  // Recovery after an exercise is done: no other can be marked done before it ends.
  const recovery = useCountdown();
  const { left: recoveryLeft, share: recoveryShare } = recovery;

  const exerciseOf = (item) => config.value.exercises.find((ex) => ex.id === item.exerciseId);
  // Exercise-specific recovery, otherwise the global recovery.
  const recoveryAfter = (item) => exerciseOf(item)?.recovery ?? config.value.recovery;
  // Time unit of a timed exercise, undefined otherwise.
  const timedUnit = (item) => exerciseOf(item)?.timed;

  // Exercise whose recovery runs: a saved configuration that changes its recovery resizes the one running, unless
  // it also ends the session, which stops the recovery.
  const recovering = ref(null);
  watch(
    () => (session.value.startedAt && recovering.value ? recoveryAfter(recovering.value) : null),
    (seconds) => seconds === null || recovery.resize(seconds),
  );

  function startRecovery(item) {
    recovering.value = item;
    recovery.start(recoveryAfter(item), () => {
      if (config.value.sound) recoveryEnd();
    });
  }

  function skipRecovery() {
    recovery.stop();
  }

  // Timer of a timed exercise: its preparation, then its duration; for an exercise on each side, once per side.
  // Like the recovery, it blocks Done.
  const timer = useCountdown();
  const { left: timerLeft, share: timerShare, held: timerHeld } = timer;
  // Exercise timed, with its set and number, whether the timer is still in its preparation and, on each side,
  // the side in progress.
  const timing = ref(null);
  // Exercise whose first side is done: once its timer is stopped, it resumes at the second side.
  let sideDone = null;
  const sameExercise = (a, b) => !!a && !!b && a.drawId === b.drawId && a.exerciseId === b.exerciseId && a.amount === b.amount;

  function startTimer(draw, item) {
    const unit = timedUnit(item);
    if (!unit || recoveryLeft.value || timing.value) return;
    const target = { drawId: draw.id, exerciseId: item.exerciseId, amount: item.amount };
    const side = exerciseOf(item).eachSide ? (sameExercise(sideDone, target) ? SIDES : 1) : undefined;
    runTimer(draw, item, target, item.amount * UNIT_SECONDS[unit], side);
  }

  function runTimer(draw, item, target, seconds, side) {
    timing.value = { ...target, ready: true, side };
    timer.start(config.value.timerPrep, () => {
      timing.value.ready = false;
      if (config.value.sound) startSignal();
      timer.start(seconds, () => {
        if (!side || side === SIDES) {
          timerEnded(draw, item);
          return;
        }
        sideDone = target;
        if (config.value.sound) sideSignal();
        runTimer(draw, item, target, seconds, side + 1);
      });
    });
  }

  function timerEnded(draw, item) {
    timing.value = null;
    sideDone = null;
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

  // The game of Play is the session's own once it has one, even left without event. Otherwise a remembered or chosen
  // game removed, or left without event, asks for the game again. The config is watched too, because an unknown
  // remembered game leaves `game` null both before and after it loads.
  watch([config, game, () => session.value.gameId], ([cfg, g, locked]) => {
    if (locked) gameId.value = locked;
    else if (cfg && !g?.events.length) gameId.value = '';
  });
  watch(session, (s) => {
    if (s.gameId) intensity.value = s.intensity;
  });

  // A session resumed at launch brings the intensity chosen in an earlier launch: once it ends, however it ends,
  // unless an intensity is chosen meanwhile, the intensity is the one a launch gives, normal unless remembered.
  let inherited = false;
  watch(() => session.value.startedAt, (started) => {
    if (started || !inherited) return;
    inherited = false;
    if (!config.value.rememberIntensity) intensity.value = 'normal';
  });

  async function resume() {
    const s = await call('get_session');
    inherited = !!s.gameId;
    session.value = s;
  }

  // Intensity changed during the session: applied right away to the sets to do.
  watch(intensity, async (value) => {
    if (!session.value.gameId || value === session.value.intensity) return;
    inherited = false;
    try {
      session.value = await call('set_session_intensity', { intensity: value });
    } catch {
      await reload();
    }
  });

  async function reload() {
    session.value = await invoke('get_session');
  }

  // Configuration saved right away and applied to the session; false if the backend refuses it. The configuration
  // and the session it leaves are applied together, so that nothing sees one without the other.
  async function saveNow(cfg) {
    const saved = await store(cfg);
    if (!saved) return false;
    const refitted = await invoke('get_session');
    config.value = saved;
    session.value = refitted;
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

  // Whatever triggers it (button, key), an event needs an included exercise, like the session's start.
  async function trigger(ev) {
    if (noExercise.value) return;
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

  // Started set whose stop awaits confirmation: the rest of the set is dropped, its done exercises stay counted.
  const confirmStop = ref(0);
  const started = (draw) => draw.items.some((it) => it.done);

  // A set not started is dismissed in one click, a started one stops after confirmation.
  function dismiss(draw) {
    if (started(draw)) confirmStop.value = draw.id;
    else dismissNow(draw);
  }

  function dismissNow(draw) {
    confirmStop.value = 0;
    return act('dismiss', draw);
  }

  // Exercises of the set still to do.
  const remaining = (draw) => draw.items.filter((it) => !it.done);
  // Joker cost absent when the action is no longer possible.
  const canAfford = (cost) => cost != null && session.value.jokers >= cost;
  const setSkipCost = (draw) => remaining(draw).reduce((sum, it) => sum + it.skipCost, 0);

  // The timer stops once its exercise is no longer the one to do as it was: set dismissed or skipped,
  // exercise halved or skipped, session cancelled. A side done is then forgotten.
  watch(session, (s) => {
    const first = s.pending[0];
    const item = first && remaining(first)[0];
    const current = item && { drawId: first.id, exerciseId: item.exerciseId, amount: item.amount };
    if (!sameExercise(sideDone, current)) sideDone = null;
    if (timing.value && !sameExercise(timing.value, current)) stopTimer();
  });

  async function complete(draw, item) {
    if (recoveryLeft.value || timing.value) return;
    if (await act('complete_exercise', draw, { exerciseId: item.exerciseId })) startRecovery(item);
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
      session.value = await call('start_session', { gameId: game.value.id, intensity: intensity.value });
    } catch {
      /* error already shown */
    }
  }

  async function endSession() {
    notice.value = '';
    try {
      session.value = await call('end_session');
    } catch {
      await reload();
    }
  }

  async function cancelSession() {
    notice.value = '';
    try {
      session.value = await call('cancel_session');
    } catch {
      /* error already shown */
    }
  }

  return {
    session, notice, farewell, recoveryLeft, recoveryShare, skipRecovery, timing, timerLeft, timerShare, timerHeld, timedUnit, startTimer, stopTimer,
    toggleTimer, resume, reload, saveNow, saveChange, trigger, act, confirmStop, started, dismiss, dismissNow, remaining, canAfford, setSkipCost,
    complete, halve, skipExercise, skipSet, startSession, endSession, cancelSession,
  };
});

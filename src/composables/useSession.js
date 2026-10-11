import { computed, onScopeDispose, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { defineShared } from './shared.js';

// Rafraîchissement du compte à rebours de la pause.
const PAUSE_TICK_MS = 250;

export const [provideSession, useSession] = defineShared(({ call }, { config, game, gameId, store }) => {
  const session = ref({ pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, skipped: [] });
  const notice = ref('');
  // Pause après un exercice fait : aucun autre ne peut être noté fait avant sa fin.
  const pauseEnd = ref(0);
  const now = ref(0);
  let ticker;
  // Série dont un exercice a lancé la pause : l'annuler arrête la pause.
  let pauseDrawId = 0;
  const pauseLeft = computed(() => Math.max(0, Math.ceil((pauseEnd.value - now.value) / 1000)));

  // Pause propre de l'exercice, la pause globale à défaut.
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
    if (s.gameId) gameId.value = s.gameId;
  });

  async function reload() {
    session.value = await invoke('get_session');
  }

  // Configuration enregistrée aussitôt et appliquée à la séance ; false si le backend la refuse.
  async function saveNow(cfg) {
    if (!(await store(cfg))) return false;
    await reload();
    return true;
  }

  async function trigger(ev) {
    notice.value = '';
    try {
      const draw = await call('draw', { gameId: game.value.id, eventId: ev.id });
      if (draw.capped) {
        notice.value = `${draw.eventName} : limite de ${draw.maxPerSession} atteinte pour la séance, pas de série.`;
      }
      session.value = await call('get_session');
    } catch {
      /* erreur déjà affichée */
    }
  }

  // Fait, joker ou annulation d'une série : en cas d'échec, on resynchronise la file.
  async function act(command, draw, args = {}) {
    try {
      session.value = await call(command, { drawId: draw.id, ...args });
      return true;
    } catch {
      await reload();
      return false;
    }
  }

  // Série commencée dont l'annulation attend confirmation : ses exercices faits ne compteront pas.
  const confirmDismiss = ref(0);

  function dismiss(draw) {
    if (draw.items.some((it) => it.done)) confirmDismiss.value = draw.id;
    else dismissNow(draw);
  }

  async function dismissNow(draw) {
    confirmDismiss.value = 0;
    if ((await act('dismiss', draw)) && draw.id === pauseDrawId) skipPause();
  }

  // Exercices de la série encore à faire.
  const remaining = (draw) => draw.items.filter((it) => !it.done);
  // Coût en jokers absent quand l'action n'est plus possible.
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

  async function newSession() {
    notice.value = '';
    skipPause();
    session.value = await call('new_session');
  }

  async function cancelSession() {
    notice.value = '';
    skipPause();
    try {
      session.value = await call('cancel_session');
    } catch {
      /* erreur déjà affichée */
    }
  }

  return {
    session, notice, pauseLeft, skipPause, reload, saveNow, trigger, act, confirmDismiss, dismiss,
    dismissNow, remaining, canAfford, setSkipCost, complete, halve, skipExercise, skipSet, newSession, cancelSession,
  };
});

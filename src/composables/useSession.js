import { computed, onScopeDispose, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { defineShared } from './shared.js';

// Rafraîchissement du compte à rebours de la pause.
const PAUSE_TICK_MS = 250;

export const [provideSession, useSession] = defineShared(({ call }, { config, game, gameId, store }) => {
  const session = ref({ pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, skipped: [] });
  const notice = ref('');
  // Pause après une série validée : la suivante ne peut pas être validée avant sa fin.
  const pauseEnd = ref(0);
  const now = ref(0);
  let ticker;
  const pauseLeft = computed(() => Math.max(0, Math.ceil((pauseEnd.value - now.value) / 1000)));

  // La plus longue des pauses des exercices de la série, la pause globale à défaut de pause propre.
  function pauseAfter(draw) {
    const { exercises, pause } = config.value;
    return Math.max(...draw.items.map((it) => exercises.find((ex) => ex.id === it.exerciseId)?.pause ?? pause));
  }

  function startPause(seconds) {
    skipPause();
    if (!seconds) return;
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
    const complete = command === 'complete';
    if (complete && pauseLeft.value) return;
    try {
      session.value = await call(command, { drawId: draw.id, ...args });
      if (complete) startPause(pauseAfter(draw));
    } catch {
      await reload();
    }
  }

  // Coût en jokers absent quand l'action n'est plus possible.
  const canAfford = (cost) => cost != null && session.value.jokers >= cost;
  const setSkipCost = (draw) => draw.items.reduce((sum, it) => sum + it.skipCost, 0);

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
    session, notice, pauseLeft, skipPause, reload, saveNow, trigger, act,
    canAfford, setSkipCost, halve, skipExercise, skipSet, newSession, cancelSession,
  };
});

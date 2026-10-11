import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { defineShared } from './shared.js';

export const [provideSession, useSession] = defineShared(({ call }, { game, gameId, store }) => {
  const session = ref({ pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, skipped: [] });
  const notice = ref('');

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
  async function act(command, draw) {
    try {
      session.value = await call(command, { drawId: draw.id });
    } catch {
      await reload();
    }
  }

  async function newSession() {
    notice.value = '';
    session.value = await call('new_session');
  }

  async function cancelSession() {
    notice.value = '';
    try {
      session.value = await call('cancel_session');
    } catch {
      /* erreur déjà affichée */
    }
  }

  return { session, notice, reload, saveNow, trigger, act, newSession, cancelSession };
});

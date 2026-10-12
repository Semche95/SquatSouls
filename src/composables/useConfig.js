import { computed, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { byName, clone, normalized, numberError } from '../configDraft.js';
import { errorText, t } from '../i18n.js';
import { defineShared } from './shared.js';

// Display duration of a confirmation.
const CONFIG_MSG_MS = 3000;

function findOr(games, id) {
  return games.find((g) => g.id === id) ?? byName(games)[0] ?? null;
}

export const [provideConfig, useConfig] = defineShared(() => {
  const config = ref(null);
  // Working copy edited in the Configuration tab, sent as is when saving.
  const draft = ref(null);
  const configError = ref('');
  // Transient confirmation (save, copy, import).
  const configMsg = ref('');
  let configMsgTimer;
  // Displayed tab, and configuration sub-tab.
  const tab = ref('play');
  const cfgTab = ref('games');
  const editedGameId = ref('');
  // Game creation: form on the right, nothing is added before confirmation.
  // A detected Steam game prefills it with its name and AppId.
  const newGame = ref(null);
  // Last chosen game, kept by the backend from one launch to the next (empty until one is chosen).
  const gameId = ref('');

  const game = computed(() => config.value?.games.find((g) => g.id === gameId.value) ?? null);
  const editedGame = computed(() => findOr(draft.value?.games ?? [], editedGameId.value));
  const dirty = computed(() => !!draft.value && JSON.stringify(draft.value) !== JSON.stringify(config.value));
  // First badly filled field of the draft, which blocks "Enregistrer" (empty if everything is valid).
  const draftError = computed(() => (draft.value ? numberError(normalized(draft.value)) : ''));

  // Remembered or current game removed: ask for the game again. The config is watched too, because an
  // unknown remembered game leaves `game` null both before and after it loads.
  watch([config, game], ([cfg, g]) => {
    if (cfg && !g) gameId.value = '';
  });
  watch(gameId, (id) => {
    if (id) invoke('set_last_game', { gameId: id }).catch(() => {});
  });
  // Keeps the "Jeu" list on the game actually edited (the first by name until one is chosen).
  watch(editedGame, (g) => {
    if (g && g.id !== editedGameId.value) editedGameId.value = g.id;
  });

  function applyConfigView(view) {
    config.value = view.config;
    draft.value = clone(view.config);
    configError.value = view.error ? errorText(view.error) : '';
  }

  // Saves `cfg` without touching the draft; returns false if the backend refuses.
  async function store(cfg) {
    const clean = normalized(cfg);
    configError.value = numberError(clean);
    if (configError.value) return false;
    try {
      config.value = (await invoke('save_config', { config: clean })).config;
      configError.value = '';
      return true;
    } catch (e) {
      configError.value = errorText(e);
      return false;
    }
  }

  function notify(msg) {
    configMsg.value = msg;
    clearTimeout(configMsgTimer);
    if (msg) configMsgTimer = setTimeout(() => (configMsg.value = ''), CONFIG_MSG_MS);
  }

  // Copies a shared text to the clipboard.
  async function copy(text) {
    try {
      await navigator.clipboard.writeText(text);
      notify(t('config.copied'));
    } catch (e) {
      configError.value = t('common.copyFailed', { error: String(e) });
    }
  }

  return { config, draft, configError, configMsg, notify, copy, tab, cfgTab, editedGameId, newGame, gameId, game, editedGame, dirty, draftError, applyConfigView, store };
});

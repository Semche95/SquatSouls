import { computed, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { byName, clone, normalized, fieldError, withTabDraft } from '../configDraft.js';
import { errorText, t } from '../i18n.js';
import { defineShared } from './shared.js';

// Display duration of a confirmation.
const CONFIG_MSG_MS = 3000;

function findOr(games, id) {
  return games.find((g) => g.id === id) ?? byName(games)[0] ?? null;
}

export const [provideConfig, useConfig] = defineShared(() => {
  const config = ref(null);
  // Working copy edited in the Program and Settings tabs, each tab saving only its own fields.
  const draft = ref(null);
  // Error of the saved configuration, kept until closed, or of an action.
  const configError = ref('');
  // Draft when an action failed: its error no longer applies once the draft changes (edited or discarded).
  let failedDraft = null;
  function fail(msg) {
    failedDraft = msg ? JSON.stringify(draft.value) : null;
    configError.value = msg;
  }
  // Transient confirmation (save, copy, import).
  const configMsg = ref('');
  let configMsgTimer;
  // Displayed tab, and Program sub-tab.
  const tab = ref('play');
  const cfgTab = ref('games');
  const editedGameId = ref('');
  // Game creation: form on the right, nothing is added before confirmation.
  // A detected Steam game prefills it with its name and AppId.
  const newGame = ref(null);
  // Last chosen game, kept by the backend from one launch to the next (empty until one is chosen).
  const gameId = ref('');
  // Intensity of the next session, normal at each launch unless the configuration remembers it.
  const intensity = ref('normal');

  const noGame = computed(() => !config.value?.games.length);
  // A session is only played with an included exercise, on a game with an event.
  const noExercise = computed(() => !config.value?.exercises.some((ex) => ex.enabled));
  const playable = computed(() => !!config.value?.games.some((g) => g.events.length));
  const game = computed(() => config.value?.games.find((g) => g.id === gameId.value) ?? null);
  const editedGame = computed(() => findOr(draft.value?.games ?? [], editedGameId.value));
  // Configuration the Program tab, or the Settings tab, would save: the other tab's pending changes stay in the draft.
  const tabDraft = (settings) => draft.value && withTabDraft(config.value, draft.value, settings);
  function tabDirty(settings) {
    const cfg = tabDraft(settings);
    return !!cfg && JSON.stringify(cfg) !== JSON.stringify(config.value);
  }
  // Sharing and importing work on the saved program: not while Program has pending changes, which a share
  // would leave out and an import would have to merge. Why they are not possible, '' when they are.
  const shareBlocked = computed(() => (tabDirty(false) ? t('config.saveFirst') : ''));

  watch(gameId, (id) => {
    if (id) invoke('set_last_game', { gameId: id }).catch(() => {});
  });
  // Saved only while remembered, so that turning it on keeps the current intensity.
  watch([intensity, () => config.value?.rememberIntensity], ([value, remember]) => {
    if (remember) invoke('set_intensity', { intensity: value }).catch(() => {});
  });
  watch(draft, (d) => failedDraft !== null && JSON.stringify(d) !== failedDraft && fail(''), { deep: true });
  // Keeps the Program game select on the game actually edited (the first by name until one is chosen).
  watch(editedGame, (g) => {
    if (g && g.id !== editedGameId.value) editedGameId.value = g.id;
  });

  function applyConfigView(view) {
    config.value = view.config;
    draft.value = clone(view.config);
    fail('');
    configError.value = view.error ? errorText(view.error) : '';
  }

  // Saves `cfg` without applying it nor touching the draft; returns the saved configuration, null if the backend
  // refuses.
  async function store(cfg) {
    const clean = normalized(cfg);
    fail(fieldError(clean));
    if (configError.value) return null;
    try {
      const saved = (await invoke('save_config', { config: clean })).config;
      fail('');
      return saved;
    } catch (e) {
      fail(errorText(e));
      return null;
    }
  }

  function notify(msg) {
    configMsg.value = msg;
    clearTimeout(configMsgTimer);
    if (msg) configMsgTimer = setTimeout(() => (configMsg.value = ''), CONFIG_MSG_MS);
  }

  // Copies a shared text to the clipboard.
  async function copy(text) {
    if (shareBlocked.value) return;
    try {
      await navigator.clipboard.writeText(text);
      notify(t('config.copied'));
    } catch (e) {
      fail(t('common.copyFailed', { error: String(e) }));
    }
  }

  return { config, draft, configError, configMsg, notify, copy, tab, cfgTab, editedGameId, newGame, gameId, intensity, noGame, noExercise, playable, game, editedGame, tabDraft, tabDirty, shareBlocked, applyConfigView, store };
});

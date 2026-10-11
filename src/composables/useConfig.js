import { computed, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { byName, clone, normalized, numberError } from '../configDraft.js';
import { defineShared } from './shared.js';

// Durée d'affichage d'une confirmation.
const CONFIG_MSG_MS = 3000;

function findOr(games, id) {
  return games.find((g) => g.id === id) ?? byName(games)[0] ?? null;
}

export const [provideConfig, useConfig] = defineShared(() => {
  const config = ref(null);
  // Copie de travail éditée dans l'onglet Configuration, envoyée telle quelle à l'enregistrement.
  const draft = ref(null);
  const configError = ref('');
  // Confirmation passagère (enregistrement, copie, import).
  const configMsg = ref('');
  let configMsgTimer;
  const cfgTab = ref('jeux');
  const editedGameId = ref('');
  // Création d'un jeu : formulaire à droite, rien n'est ajouté avant validation.
  const newGame = ref(null);
  // Dernier jeu choisi, gardé par le backend d'un lancement à l'autre (vide tant qu'aucun n'est choisi).
  const gameId = ref('');

  const game = computed(() => config.value?.games.find((g) => g.id === gameId.value) ?? null);
  const editedGame = computed(() => findOr(draft.value?.games ?? [], editedGameId.value));
  const dirty = computed(() => !!draft.value && JSON.stringify(draft.value) !== JSON.stringify(config.value));

  // Jeu mémorisé ou en cours supprimé : on redemande le jeu.
  watch(game, (g) => {
    if (config.value && !g) gameId.value = '';
  });
  watch(gameId, (id) => {
    if (id) invoke('set_last_game', { gameId: id }).catch(() => {});
  });
  // Garde la liste « Jeu » sur le jeu réellement édité (le premier par nom tant qu'aucun n'est choisi).
  watch(editedGame, (g) => {
    if (g && g.id !== editedGameId.value) editedGameId.value = g.id;
  });

  function applyConfigView(view) {
    config.value = view.config;
    draft.value = clone(view.config);
    configError.value = view.error ?? '';
  }

  // Enregistre `cfg` sans toucher au brouillon ; renvoie false si le backend refuse.
  async function store(cfg) {
    const clean = normalized(cfg);
    configError.value = numberError(clean);
    if (configError.value) return false;
    try {
      config.value = (await invoke('save_config', { config: clean })).config;
      configError.value = '';
      return true;
    } catch (e) {
      configError.value = String(e);
      return false;
    }
  }

  function notify(msg) {
    configMsg.value = msg;
    clearTimeout(configMsgTimer);
    if (msg) configMsgTimer = setTimeout(() => (configMsg.value = ''), CONFIG_MSG_MS);
  }

  // Copie un texte partagé dans le presse-papiers.
  async function copy(text) {
    try {
      await navigator.clipboard.writeText(text);
      notify('Copié dans le presse-papiers, prêt à partager.');
    } catch (e) {
      configError.value = `Copie impossible : ${e}`;
    }
  }

  return { config, draft, configError, configMsg, notify, copy, cfgTab, editedGameId, newGame, gameId, game, editedGame, dirty, applyConfigView, store };
});

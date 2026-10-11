import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { clone } from "../configDraft.js";
import { defineShared } from "./shared.js";

function findOr(games, id) {
  return games.find((g) => g.id === id) ?? games[0] ?? null;
}

export const [provideConfig, useConfig] = defineShared(() => {
  const config = ref(null);
  // Copie de travail éditée dans l'onglet Configuration, envoyée telle quelle à l'enregistrement.
  const draft = ref(null);
  const configError = ref("");
  const cfgTab = ref("jeux");
  const editedGameId = ref("");
  // Création d'un jeu : formulaire à droite, rien n'est ajouté avant validation.
  const newGame = ref(null);
  // Dernier jeu choisi, gardé par le backend d'un lancement à l'autre.
  const gameId = ref("");

  const game = computed(() => findOr(config.value?.games ?? [], gameId.value));
  const editedGame = computed(() => findOr(draft.value?.games ?? [], editedGameId.value));
  const dirty = computed(() => !!draft.value && JSON.stringify(draft.value) !== JSON.stringify(config.value));

  watch(game, (g) => {
    if (g && g.id !== gameId.value) gameId.value = g.id;
  });
  watch(gameId, (id) => {
    if (id) invoke("set_last_game", { gameId: id }).catch(() => {});
  });
  // Garde la liste « Jeu » sur le jeu réellement édité (le premier tant qu'aucun n'est choisi).
  watch(editedGame, (g) => {
    if (g && g.id !== editedGameId.value) editedGameId.value = g.id;
  });

  function applyConfigView(view) {
    config.value = view.config;
    draft.value = clone(view.config);
    configError.value = view.error ?? "";
  }

  return { config, draft, configError, cfgTab, editedGameId, newGame, gameId, game, editedGame, dirty, applyConfigView };
});

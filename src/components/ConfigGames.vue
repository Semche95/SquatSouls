<script setup>
import { computed, nextTick, ref } from "vue";
import { clone, newId, sameName } from "../configDraft.js";
import { useConfig } from "../composables/useConfig.js";
import { useSession } from "../composables/useSession.js";
import ConfigEvent from "./ConfigEvent.vue";

const { config, draft, editedGameId, editedGame, newGame, store } = useConfig();
const { reload } = useSession();
// Jeu dont la suppression attend confirmation ; changer de jeu la referme.
const confirmRemove = ref(null);
const newGameName = ref(null);
const nameTaken = computed(() => config.value.games.some((g) => sameName(g.name, newGame.value.name)));

async function openNewGame() {
  newGame.value = { name: "", source: "" };
  await nextTick();
  newGameName.value.focus();
}

// Création et suppression validées par l'utilisateur sont enregistrées aussitôt,
// seules les autres modifications du brouillon attendent « Enregistrer ».
async function saveGames(games) {
  if (!(await store({ ...config.value, games }))) return false;
  await reload();
  return true;
}

async function createGame() {
  const { name, source } = newGame.value;
  const model = draft.value.games.find((g) => g.id === source);
  const game = { id: newId(name, draft.value.games), name: name.trim(), events: model ? clone(model.events) : [] };
  if (!model) addEvent(game);
  if (!(await saveGames([...config.value.games, game]))) return;
  draft.value.games.push(clone(config.value.games.at(-1)));
  editedGameId.value = game.id;
  newGame.value = null;
}

async function removeGame(game) {
  confirmRemove.value = null;
  if (!(await saveGames(config.value.games.filter((g) => g.id !== game.id)))) return;
  draft.value.games = draft.value.games.filter((g) => g !== game);
  editedGameId.value = draft.value.games[0]?.id ?? "";
}

function addEvent(game) {
  game.events.push({ id: newId("Nouvel événement", game.events), name: "Nouvel événement", draws: 1, jokers: 0 });
}
</script>

<template>
  <section class="group">
    <header class="group-head">
      <h2 class="group-title">Jeux</h2>
      <button class="primary" :disabled="!!newGame" @click="openNewGame">+ Ajouter un jeu</button>
    </header>
    <div class="cfg-games">
      <nav class="subtabs cfg-game-list" aria-label="Jeu à modifier">
        <button
          v-for="g in config.games"
          :key="g.id"
          :class="{ active: !newGame && g.id === editedGame?.id }"
          @click="editedGameId = g.id; newGame = null"
        >{{ g.name }}</button>
      </nav>
      <form v-if="newGame" class="cfg-game" @submit.prevent="createGame">
        <h3 class="group-title">Nouveau jeu</h3>
        <label class="field">
          <span>Nom</span>
          <input ref="newGameName" v-model="newGame.name" aria-label="Nom du nouveau jeu" />
        </label>
        <p v-if="nameTaken" class="muted">Un jeu porte déjà ce nom.</p>
        <label class="field">
          <span>Événements</span>
          <select v-model="newGame.source" aria-label="Événements du nouveau jeu">
            <option value="">Aucun (un événement à remplir)</option>
            <option v-for="g in draft.games" :key="g.id" :value="g.id">Copier depuis {{ g.name || "(sans nom)" }}</option>
          </select>
        </label>
        <div class="row">
          <button class="primary" :disabled="!newGame.name.trim() || nameTaken">Créer</button>
          <button type="button" class="ghost" @click="newGame = null">Annuler</button>
        </div>
      </form>
      <div v-else-if="editedGame" class="cfg-game">
        <div class="field">
          <span>Nom</span>
          <input v-model="editedGame.name" aria-label="Nom du jeu" />
          <button class="ghost danger" :disabled="draft.games.length < 2" @click="confirmRemove = editedGame">Supprimer ce jeu</button>
        </div>
        <div v-if="confirmRemove === editedGame" class="banner confirm">
          <p>Supprimer {{ editedGame.name || "ce jeu" }} et tous ses événements ?</p>
          <div class="card-actions">
            <button class="danger" @click="removeGame(editedGame)">Supprimer</button>
            <button class="ghost" @click="confirmRemove = null">Garder le jeu</button>
          </div>
        </div>

        <div class="cfg-events">
          <ConfigEvent v-for="(ev, i) in editedGame.events" :key="ev.id" :ev="ev" @remove="editedGame.events.splice(i, 1)" />
        </div>
        <button class="ghost" @click="addEvent(editedGame)">Ajouter un événement</button>
      </div>
    </div>
  </section>
</template>

<script setup>
import { clone, newId } from "../configDraft.js";
import { useConfig } from "../composables/useConfig.js";
import ConfigEvent from "./ConfigEvent.vue";

const { draft, editedGameId, editedGame, newGame } = useConfig();

function openNewGame() {
  newGame.value = { name: "", source: "" };
}

function createGame() {
  const games = draft.value.games;
  const { name, source } = newGame.value;
  const model = games.find((g) => g.id === source);
  const game = { id: newId(name, games), name: name.trim(), events: model ? clone(model.events) : [] };
  if (!model) addEvent(game);
  games.push(game);
  editedGameId.value = game.id;
  newGame.value = null;
}

function removeGame(game) {
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
      <button class="ghost" :disabled="!!newGame" @click="openNewGame">Ajouter un jeu</button>
    </header>
    <div class="cfg-games">
      <nav class="subtabs cfg-game-list" aria-label="Jeu à modifier">
        <button
          v-for="g in draft.games"
          :key="g.id"
          :class="{ active: !newGame && g.id === editedGame?.id }"
          @click="editedGameId = g.id; newGame = null"
        >{{ g.name || "(sans nom)" }}</button>
      </nav>
      <form v-if="newGame" class="cfg-game" @submit.prevent="createGame">
        <h3 class="group-title">Nouveau jeu</h3>
        <label class="field">
          <span>Nom</span>
          <input v-model="newGame.name" aria-label="Nom du nouveau jeu" />
        </label>
        <label class="field">
          <span>Événements</span>
          <select v-model="newGame.source" aria-label="Événements du nouveau jeu">
            <option value="">Aucun (un événement à remplir)</option>
            <option v-for="g in draft.games" :key="g.id" :value="g.id">Copier depuis {{ g.name || "(sans nom)" }}</option>
          </select>
        </label>
        <div class="row">
          <button class="primary" :disabled="!newGame.name.trim()">Créer</button>
          <button type="button" class="ghost" @click="newGame = null">Annuler</button>
        </div>
      </form>
      <div v-else-if="editedGame" class="cfg-game">
        <div class="field">
          <span>Nom</span>
          <input v-model="editedGame.name" aria-label="Nom du jeu" />
          <button class="ghost danger" :disabled="draft.games.length < 2" @click="removeGame(editedGame)">Supprimer ce jeu</button>
        </div>

        <div class="cfg-events">
          <ConfigEvent v-for="(ev, i) in editedGame.events" :key="ev.id" :ev="ev" @remove="editedGame.events.splice(i, 1)" />
        </div>
        <button class="ghost" @click="addEvent(editedGame)">Ajouter un événement</button>
      </div>
    </div>
  </section>
</template>

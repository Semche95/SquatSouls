<script setup>
import { computed, nextTick, ref } from 'vue';
import { byName, clone, newId, sameName } from '../configDraft.js';
import { plural } from '../format.js';
import { importGame, parseGame, shareGame } from '../share.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ConfigEvent from './ConfigEvent.vue';
import ShareImport from './ShareImport.vue';

const { config, draft, editedGameId, editedGame, newGame, notify, copy } = useConfig();
const { saveNow, saveChange } = useSession();
// Import of a shared game: form on the right, like creation.
const importing = ref(false);
// Game whose removal awaits confirmation; switching games closes it.
const confirmRemove = ref(null);
const newGameName = ref(null);
const nameTaken = computed(() => config.value.games.some((g) => sameName(g.name, newGame.value.name)));

function selectGame(id) {
  editedGameId.value = id;
  newGame.value = null;
  importing.value = false;
}

async function openNewGame() {
  importing.value = false;
  newGame.value = { name: '', source: '' };
  await nextTick();
  newGameName.value.focus();
}

// Creation, import and removal confirmed by the user are saved right away,
// only the other draft changes wait for "Enregistrer".
function saveGames(games) {
  return saveNow({ ...config.value, games });
}

// A detected Steam game once added leaves the ignored games list.
async function createGame() {
  const { name, source, steamAppId } = newGame.value;
  const model = draft.value.games.find((g) => g.id === source);
  const game = { id: newId(name, draft.value.games), name: name.trim(), events: model ? clone(model.events) : [] };
  if (steamAppId) game.steamAppId = steamAppId;
  if (!model) addEvent(game);
  const created = await saveChange((cfg) => {
    cfg.games.push(clone(game));
    cfg.ignoredGames = cfg.ignoredGames.filter((g) => g.appId !== steamAppId);
  });
  if (!created) return;
  editedGameId.value = game.id;
  newGame.value = null;
}

async function removeGame(game) {
  confirmRemove.value = null;
  if (!(await saveGames(config.value.games.filter((g) => g.id !== game.id)))) return;
  draft.value.games = draft.value.games.filter((g) => g !== game);
  editedGameId.value = '';
}

function openImport() {
  newGame.value = null;
  importing.value = true;
}

const activeCount = () => config.value.exercises.filter((ex) => ex.enabled).length;
const parseShared = (text) => parseGame(text, activeCount());
const takenName = (name) => config.value.games.some((g) => sameName(g.name, name));

function shareEdited() {
  copy(shareGame(config.value.games.find((g) => g.id === editedGame.value.id)));
}

async function importShared(game) {
  if (!(await saveGames(importGame(config.value.games, game)))) return;
  draft.value.games = importGame(draft.value.games, clone(game));
  selectGame(config.value.games.find((g) => sameName(g.name, game.name)).id);
  const count = activeCount();
  notify(game.clamped ? `${game.name} importé, tirages limités à ${plural(count, 'exercice')}.` : `${game.name} importé.`);
}

function addEvent(game) {
  game.events.push({ id: newId('Nouvel événement', game.events), name: 'Nouvel événement', draws: 1, jokers: 0 });
}
</script>

<template>
  <section class="group">
    <header class="group-head">
      <h2 class="group-title">Jeux</h2>
      <button class="primary" :disabled="!!newGame" @click="openNewGame">+ Ajouter un jeu</button>
      <button class="ghost" :disabled="importing" @click="openImport">Importer un jeu</button>
    </header>
    <div class="cfg-games">
      <nav class="subtabs cfg-game-list" aria-label="Jeu à modifier">
        <button
          v-for="g in byName(config.games)"
          :key="g.id"
          :class="{ active: !newGame && !importing && g.id === editedGame?.id }"
          @click="selectGame(g.id)"
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
            <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">Copier depuis {{ g.name || '(sans nom)' }}</option>
          </select>
        </label>
        <div class="row">
          <button class="primary" :disabled="!newGame.name.trim() || nameTaken">Créer</button>
          <button type="button" class="ghost" @click="newGame = null">Annuler</button>
        </div>
      </form>
      <div v-else-if="importing" class="cfg-game">
        <h3 class="group-title">Importer un jeu</h3>
        <ShareImport v-slot="{ result }" :parse="parseShared" @import="importShared" @cancel="importing = false">
          <p v-if="takenName(result.name)" class="banner warn">{{ result.name }} existe déjà : il sera remplacé.</p>
        </ShareImport>
      </div>
      <div v-else-if="editedGame" class="cfg-game">
        <div class="field">
          <span>Nom</span>
          <input v-model="editedGame.name" aria-label="Nom du jeu" />
          <button class="ghost" @click="shareEdited">Partager</button>
          <button class="ghost danger" :disabled="draft.games.length < 2" @click="confirmRemove = editedGame">Supprimer ce jeu</button>
        </div>
        <div v-if="confirmRemove === editedGame" class="banner confirm">
          <p>Supprimer {{ editedGame.name || 'ce jeu' }} et tous ses événements ?</p>
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

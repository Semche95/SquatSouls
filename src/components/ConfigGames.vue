<script setup>
import { computed, nextTick, ref } from 'vue';
import { byName, clone, newId, sameName } from '../configDraft.js';
import { t } from '../i18n.js';
import { importGame, parseGame, shareGame } from '../share.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ConfigEvent from './ConfigEvent.vue';
import ShareImport from './ShareImport.vue';
import DisabledTip from './DisabledTip.vue';

const { config, draft, editedGameId, editedGame, newGame, notify, copy } = useConfig();
const { saveNow, saveChange } = useSession();
// Import of a shared game: form on the right, like creation.
const importing = ref(false);
// Game whose removal awaits confirmation; switching games closes it.
const confirmRemove = ref(null);
const newGameName = ref(null);
const nameTaken = computed(() => config.value.games.some((g) => sameName(g.name, newGame.value.name)));
// Why the new game cannot be created yet, '' when it can.
const createBlocked = computed(() => {
  if (!newGame.value.name.trim()) return t('config.games.nameNeeded');
  return nameTaken.value ? t('config.games.nameTaken') : '';
});

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
// only the other draft changes wait for "Save".
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
  notify(game.clamped ? t('config.games.importedClamped', { name: game.name }, count) : t('config.games.imported', { name: game.name }));
}

function addEvent(game) {
  const name = t('config.games.newEvent');
  game.events.push({ id: newId(name, game.events), name, draws: 1, jokers: 0 });
}
</script>

<template>
  <section class="group">
    <header class="group-head">
      <h2 class="group-title">{{ t('config.tabs.games') }}</h2>
      <DisabledTip :tip="newGame ? t('config.games.creating') : ''">
        <button class="primary" :disabled="!!newGame" @click="openNewGame">{{ t('config.games.add') }}</button>
      </DisabledTip>
      <DisabledTip :tip="importing ? t('common.importOpen') : ''">
        <button class="ghost" :disabled="importing" @click="openImport">{{ t('config.games.import') }}</button>
      </DisabledTip>
    </header>
    <div class="cfg-games">
      <nav class="subtabs cfg-game-list" :aria-label="t('config.games.list')">
        <button
          v-for="g in byName(config.games)"
          :key="g.id"
          :class="{ active: !newGame && !importing && g.id === editedGame?.id }"
          @click="selectGame(g.id)"
        >{{ g.name }}</button>
      </nav>
      <form v-if="newGame" class="cfg-game" @submit.prevent="createGame">
        <h3 class="group-title">{{ t('config.games.new') }}</h3>
        <label class="field">
          <span>{{ t('common.name') }}</span>
          <input ref="newGameName" v-model="newGame.name" :aria-label="t('config.games.newName')" />
        </label>
        <p v-if="nameTaken" class="muted">{{ t('config.games.nameTaken') }}</p>
        <label class="field">
          <span>{{ t('config.games.events') }}</span>
          <select v-model="newGame.source" :aria-label="t('config.games.newEvents')">
            <option value="">{{ t('config.games.noEvents') }}</option>
            <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">{{ t('config.games.copyFrom', { name: g.name || t('common.unnamed') }) }}</option>
          </select>
        </label>
        <div class="row">
          <DisabledTip :tip="createBlocked">
            <button class="primary" :disabled="!!createBlocked">{{ t('config.games.create') }}</button>
          </DisabledTip>
          <button type="button" class="ghost" @click="newGame = null">{{ t('common.cancel') }}</button>
        </div>
      </form>
      <div v-else-if="importing" class="cfg-game">
        <h3 class="group-title">{{ t('config.games.import') }}</h3>
        <ShareImport v-slot="{ result }" :parse="parseShared" @import="importShared" @cancel="importing = false">
          <p v-if="takenName(result.name)" class="banner warn">{{ t('config.games.willReplace', { name: result.name }) }}</p>
        </ShareImport>
      </div>
      <div v-else-if="editedGame" class="cfg-game">
        <div class="field">
          <span>{{ t('common.name') }}</span>
          <input v-model="editedGame.name" :aria-label="t('config.games.gameName')" />
          <button class="ghost" @click="shareEdited">{{ t('common.share') }}</button>
          <DisabledTip :tip="draft.games.length < 2 ? t('config.games.lastGame') : ''">
            <button class="ghost danger" :disabled="draft.games.length < 2" @click="confirmRemove = editedGame">{{ t('config.games.removeGame') }}</button>
          </DisabledTip>
        </div>
        <div v-if="confirmRemove === editedGame" class="banner confirm">
          <p>{{ t('config.games.confirmRemove', { name: editedGame.name || t('config.games.thisGame') }) }}</p>
          <div class="card-actions">
            <button class="danger" @click="removeGame(editedGame)">{{ t('common.remove') }}</button>
            <button class="ghost" @click="confirmRemove = null">{{ t('config.games.keepGame') }}</button>
          </div>
        </div>

        <div class="cfg-events">
          <ConfigEvent v-for="(ev, i) in editedGame.events" :key="ev.id" :ev="ev" @remove="editedGame.events.splice(i, 1)" />
        </div>
        <button class="ghost" @click="addEvent(editedGame)">{{ t('config.games.addEvent') }}</button>
      </div>
    </div>
  </section>
</template>

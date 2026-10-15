<script setup>
import { computed, nextTick, ref } from 'vue';
import { byName, clone, MAX_NAME, newId, sameName } from '../configDraft.js';
import { t } from '../i18n.js';
import { importGame, parseGame, shareGame } from '../share.js';
import { vReveal } from '../reveal.js';
import { vSelect } from '../select.js';
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
// Name being typed while renaming the edited game, null outside renaming.
const renaming = ref(null);
const renameInput = ref(null);
const nameTaken = computed(() => config.value.games.some((g) => sameName(g.name, newGame.value.name)));
// Why the new game cannot be created yet, '' when it can.
const createBlocked = computed(() => {
  if (!newGame.value.name.trim()) return t('config.games.nameNeeded');
  return nameTaken.value ? t('config.games.nameTaken') : '';
});
// Same for the renamed game, whose name only has to differ from the other games.
const renameBlocked = computed(() => {
  if (!renaming.value.trim()) return t('config.games.nameNeeded');
  const taken = draft.value.games.some((g) => g !== editedGame.value && sameName(g.name, renaming.value));
  return taken ? t('config.games.nameTaken') : '';
});
// Game shown in the select, none while a game is being created or imported.
const shownGame = computed(() => (newGame.value || importing.value ? null : editedGame.value));

function selectGame(id) {
  editedGameId.value = id;
  newGame.value = null;
  importing.value = false;
  renaming.value = null;
}

// The caret goes to the end of the name, to complete it rather than retype it.
async function openRename() {
  renaming.value = editedGame.value.name;
  await nextTick();
  const end = renaming.value.length;
  renameInput.value.focus();
  renameInput.value.setSelectionRange(end, end);
}

// The new name waits for "Save", like the other changes to the game.
function applyRename() {
  editedGame.value.name = renaming.value.trim();
  renaming.value = null;
}

async function openNewGame() {
  importing.value = false;
  renaming.value = null;
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
  renaming.value = null;
  importing.value = true;
}

const activeCount = () => config.value.exercises.filter((ex) => ex.enabled).length;
const parseShared = (text) => parseGame(text, activeCount());
const takenName = (name) => config.value.games.some((g) => sameName(g.name, name));

// The saved game is shared: without event, there is nothing to import.
const savedEdited = computed(() => config.value.games.find((g) => g.id === editedGame.value?.id));
function shareEdited() {
  copy(shareGame(savedEdited.value));
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
    <form v-if="renaming !== null" class="field" @submit.prevent="applyRename" @keydown.esc="renaming = null">
      <label for="cfg-game">{{ t('play.game') }}</label>
      <input id="cfg-game" ref="renameInput" v-model="renaming" :maxlength="MAX_NAME" :aria-label="t('config.games.gameName')" />
      <DisabledTip :tip="renameBlocked">
        <button class="primary" :disabled="!!renameBlocked">{{ t('config.games.renameDone') }}</button>
      </DisabledTip>
      <button type="button" class="ghost" @click="renaming = null">{{ t('common.cancel') }}</button>
    </form>
    <div v-else class="field">
      <label for="cfg-game">{{ t('play.game') }}</label>
      <select v-select id="cfg-game" :value="shownGame?.id ?? ''" @change="selectGame($event.target.value)">
        <option v-if="!shownGame" value="" disabled>{{ t('play.chooseGame') }}</option>
        <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">{{ g.name }}</option>
      </select>
      <template v-if="shownGame">
        <button class="ghost" @click="openRename">{{ t('config.games.rename') }}</button>
        <DisabledTip :tip="savedEdited?.events.length ? '' : t('config.games.shareEmpty')">
          <button class="ghost" :disabled="!savedEdited?.events.length" @click="shareEdited">{{ t('common.share') }}</button>
        </DisabledTip>
        <DisabledTip :tip="draft.games.length < 2 ? t('config.games.lastGame') : ''">
          <button class="ghost danger" :disabled="draft.games.length < 2" @click="confirmRemove = shownGame">{{ t('config.games.removeGame') }}</button>
        </DisabledTip>
      </template>
    </div>
    <form v-if="newGame" class="cfg-game" @submit.prevent="createGame">
      <h3 class="group-title">{{ t('config.games.new') }}</h3>
      <label class="field">
        <span>{{ t('common.name') }}</span>
        <input ref="newGameName" v-model="newGame.name" :maxlength="MAX_NAME" :aria-label="t('config.games.newName')" />
      </label>
      <p v-if="nameTaken" class="muted">{{ t('config.games.nameTaken') }}</p>
      <div class="field">
        <span>{{ t('config.games.events') }}</span>
        <select v-select v-model="newGame.source" :aria-label="t('config.games.newEvents')">
          <option value="">{{ t('config.games.noEvents') }}</option>
          <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">{{ t('config.games.copyFrom', { name: g.name || t('common.unnamed') }) }}</option>
        </select>
      </div>
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
      <div v-if="confirmRemove === editedGame" v-reveal class="banner confirm">
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
  </section>
</template>

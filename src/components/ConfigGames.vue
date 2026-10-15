<script setup>
import { computed, nextTick, ref } from 'vue';
import { byName, MAX_NAME, newId, sameName } from '../configDraft.js';
import { t } from '../i18n.js';
import { shareGame } from '../share.js';
import { vSelect } from '../select.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ConfigEvent from './ConfigEvent.vue';
import ConfirmDialog from './ConfirmDialog.vue';
import GameDialog from './GameDialog.vue';
import DisabledTip from './DisabledTip.vue';

const { config, draft, editedGameId, editedGame, newGame, copy } = useConfig();
const { saveNow } = useSession();
// Import of a shared game, in the same modal as creation.
const importing = ref(false);
// Removal of the edited game awaiting confirmation, and that game: its name stays shown while the modal fades out.
const removing = ref(false);
const toRemove = ref(null);
// Name being typed while renaming the edited game, null outside renaming.
const renaming = ref(null);
const renameInput = ref(null);
// Event just added, revealed as soon as it appears.
const freshId = ref(null);
// Why the renamed game cannot take this name, '' when it can: it only has to differ from the other games.
const renameBlocked = computed(() => {
  if (!renaming.value.trim()) return t('config.games.nameNeeded');
  const taken = draft.value.games.some((g) => g !== editedGame.value && sameName(g.name, renaming.value));
  return taken ? t('config.games.nameTaken') : '';
});

function selectGame(id) {
  editedGameId.value = id;
  renaming.value = null;
  freshId.value = null;
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

function openNewGame() {
  renaming.value = null;
  newGame.value = { name: '', source: '' };
}

function openImport() {
  renaming.value = null;
  importing.value = true;
}

function askRemove() {
  toRemove.value = editedGame.value;
  removing.value = true;
}

// A removal confirmed by the user is saved right away, only the other draft changes wait for "Save".
async function removeGame(game) {
  if (!(await saveNow({ ...config.value, games: config.value.games.filter((g) => g.id !== game.id) }))) return;
  draft.value.games = draft.value.games.filter((g) => g !== game);
  editedGameId.value = '';
}

// The saved game is shared: without event, there is nothing to import.
const savedEdited = computed(() => config.value.games.find((g) => g.id === editedGame.value?.id));
function shareEdited() {
  copy(shareGame(savedEdited.value));
}

// Event added to fill in: one exercise per set, no joker.
function addEvent(game) {
  const name = t('config.games.newEvent');
  const id = newId(name, game.events);
  game.events.push({ id, name, draws: 1, jokers: 0 });
  freshId.value = id;
}
</script>

<template>
  <section class="group">
    <header class="group-head">
      <h2 class="group-title">{{ t('config.tabs.games') }}</h2>
      <button class="primary" @click="openNewGame">{{ t('config.games.add') }}</button>
      <button class="ghost" @click="openImport">{{ t('config.games.import') }}</button>
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
      <select v-select id="cfg-game" :value="editedGame?.id ?? ''" @change="selectGame($event.target.value)">
        <option v-if="!editedGame" value="" disabled>{{ t('play.chooseGame') }}</option>
        <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">{{ g.name }}</option>
      </select>
      <template v-if="editedGame">
        <button class="ghost" @click="openRename">{{ t('config.games.rename') }}</button>
        <DisabledTip :tip="savedEdited?.events.length ? '' : t('config.games.shareEmpty')">
          <button class="ghost" :disabled="!savedEdited?.events.length" @click="shareEdited">{{ t('common.share') }}</button>
        </DisabledTip>
        <DisabledTip :tip="draft.games.length < 2 ? t('config.games.lastGame') : ''">
          <button class="ghost danger" :disabled="draft.games.length < 2" @click="askRemove">{{ t('config.games.removeGame') }}</button>
        </DisabledTip>
      </template>
    </div>
    <div v-if="editedGame" class="cfg-game">
      <div class="cfg-events">
        <ConfigEvent v-for="(ev, i) in editedGame.events" :key="ev.id" :ev="ev" :fresh="ev.id === freshId" @remove="editedGame.events.splice(i, 1)" />
      </div>
      <button class="ghost" @click="addEvent(editedGame)">{{ t('config.games.addEvent') }}</button>
    </div>
    <GameDialog v-model:importing="importing" />
    <ConfirmDialog
      v-model="removing"
      :text="t('config.games.confirmRemove', { name: toRemove?.name || t('config.games.thisGame') })"
      :confirm="t('common.remove')"
      :keep="t('config.games.keepGame')"
      @confirm="removeGame(toRemove)"
    />
  </section>
</template>

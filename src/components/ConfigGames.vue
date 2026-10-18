<script setup>
import { computed, nextTick, ref } from 'vue';
import { byName, MAX_NAME, newId, sameName } from '../configDraft.js';
import { t } from '../i18n.js';
import { shareGame } from '../share.js';
import { vSelect } from '../select.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useEditDialog } from '../composables/useEditDialog.js';
import ConfigEvent from './ConfigEvent.vue';
import ConfirmDialog from './ConfirmDialog.vue';
import EventDialog from './EventDialog.vue';
import GameDialog from './GameDialog.vue';
import DisabledTip from './DisabledTip.vue';

const { config, draft, editedGameId, editedGame, newGame, copy, shareBlocked } = useConfig();
const { saveNow } = useSession();
// Import of a shared game, in the same modal as creation.
const importing = ref(false);
// Removal of the edited game awaiting confirmation, and that game: its name stays shown while the modal fades out.
const removing = ref(false);
const toRemove = ref(null);
// Name being typed while renaming the edited game, null outside renaming.
const renaming = ref(null);
const renameInput = ref(null);
// Event in its edit dialog.
const { item: editing, open: editOpen, fresh, edit: editEvent, remove: removeEvent } = useEditDialog(() => editedGame.value.events);
// Why the renamed game cannot take this name, '' when it can: it only has to differ from the other games.
const renameBlocked = computed(() => {
  if (!renaming.value.trim()) return t('config.games.nameNeeded');
  const taken = draft.value.games.some((g) => g !== editedGame.value && sameName(g.name, renaming.value));
  return taken ? t('config.games.nameTaken') : '';
});

function selectGame(id) {
  editedGameId.value = id;
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
const shareGameBlocked = computed(() => shareBlocked.value || (savedEdited.value?.events.length ? '' : t('config.games.shareEmpty')));
function shareEdited() {
  copy(shareGame(savedEdited.value));
}

// Event added to fill in: one exercise per set, no joker.
function addEvent(game) {
  const name = t('config.games.newEvent');
  game.events.push({ id: newId(name, game.events), name, draws: 1, jokers: 0 });
  editEvent(game.events.at(-1), true);
}
</script>

<template>
  <section class="group">
    <header class="row cfg-game-bar">
      <form v-if="renaming !== null" class="field" @submit.prevent="applyRename" @keydown.esc="renaming = null">
        <input id="cfg-game" ref="renameInput" v-model="renaming" :maxlength="MAX_NAME" :aria-label="t('config.games.gameName')" />
        <DisabledTip :tip="renameBlocked">
          <button class="primary" :disabled="!!renameBlocked">{{ t('config.games.renameDone') }}</button>
        </DisabledTip>
        <button type="button" class="ghost" @click="renaming = null">{{ t('common.cancel') }}</button>
      </form>
      <div v-else-if="draft.games.length" class="field">
        <select v-select id="cfg-game" :value="editedGame?.id ?? ''" :aria-label="t('play.game')" @change="selectGame($event.target.value)">
          <option v-if="!editedGame" value="" disabled>{{ t('play.chooseGame') }}</option>
          <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">{{ g.name }}</option>
        </select>
        <template v-if="editedGame">
          <button class="ghost" @click="openRename">{{ t('config.games.rename') }}</button>
          <DisabledTip :tip="shareGameBlocked">
            <button class="ghost" :disabled="!!shareGameBlocked" @click="shareEdited">{{ t('common.share') }}</button>
          </DisabledTip>
          <button class="ghost danger" @click="askRemove">{{ t('common.remove') }}</button>
        </template>
      </div>
      <button class="primary" @click="openNewGame">{{ t('config.games.new') }}</button>
      <DisabledTip :tip="shareBlocked">
        <button :disabled="!!shareBlocked" @click="openImport">{{ t('common.import') }}</button>
      </DisabledTip>
    </header>
    <p v-if="!draft.games.length" class="empty">{{ t('config.games.none') }}</p>
    <template v-if="editedGame">
      <h2 class="setting-title">{{ t('config.games.events') }}</h2>
      <div class="tiles cfg-events">
        <ConfigEvent v-for="ev in editedGame.events" :key="ev.id" :ev="ev" @click="editEvent(ev)" />
        <button class="tile tile-new" @click="addEvent(editedGame)">{{ t('config.games.addEvent') }}</button>
      </div>
    </template>
    <EventDialog v-if="editing" v-model="editOpen" :ev="editing" :fresh="fresh" @remove="removeEvent" />
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

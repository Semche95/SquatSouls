<script setup>
// Name of the edited game over its events, with what applies to it: renaming, sharing and removal.
import { computed, nextTick, ref, watch } from 'vue';
import { MAX_NAME, sameName } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { shareGame } from '../sharing/share.js';
import { vTooltip } from '../ui/tooltip.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ConfirmDialog from './ConfirmDialog.vue';
import DisabledTip from './DisabledTip.vue';
import PenIcon from './PenIcon.vue';

const { config, draft, editedGameId, editedGame, copy, shareBlocked } = useConfig();
const { saveNow } = useSession();
// Removal of the edited game awaiting confirmation, and that game: its name stays shown while the modal fades out.
const removing = ref(false);
const toRemove = ref(null);
// Name being typed while renaming the edited game, null outside renaming.
const renaming = ref(null);
const renameInput = ref(null);
// Why the renamed game cannot take this name, '' when it can: it only has to differ from the other games.
const renameBlocked = computed(() => {
  if (!renaming.value.trim()) return t('config.games.nameNeeded');
  const taken = draft.value.games.some((g) => g !== editedGame.value && sameName(g.name, renaming.value));
  return taken ? t('config.games.nameTaken') : '';
});

// Renaming applies to one game: another game edited, whatever chose it, leaves it.
watch(editedGameId, () => (renaming.value = null));

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
</script>

<template>
  <header class="cfg-game-head">
    <form v-if="renaming !== null" class="field" @submit.prevent="applyRename" @keydown.esc="renaming = null">
      <input ref="renameInput" v-model="renaming" :maxlength="MAX_NAME" :aria-label="t('config.games.gameName')" />
      <DisabledTip :tip="renameBlocked">
        <button class="primary" :disabled="!!renameBlocked">{{ t('config.games.renameDone') }}</button>
      </DisabledTip>
      <button type="button" class="ghost" @click="renaming = null">{{ t('common.cancel') }}</button>
    </form>
    <template v-else>
      <h2 class="cfg-game-title">{{ editedGame.name }}</h2>
      <button v-tooltip="{ content: t('config.games.rename'), placement: 'top' }" class="ghost cfg-game-pen" :aria-label="t('config.games.rename')" @click="openRename">
        <PenIcon />
      </button>
      <div class="row cfg-game-actions">
        <DisabledTip :tip="shareGameBlocked">
          <button class="ghost" :disabled="!!shareGameBlocked" @click="shareEdited">{{ t('common.share') }}</button>
        </DisabledTip>
        <button class="ghost danger" @click="askRemove">{{ t('common.remove') }}</button>
      </div>
    </template>
    <ConfirmDialog
      v-model="removing"
      :text="t('config.games.confirmRemove', { name: toRemove?.name || t('config.games.thisGame') })"
      :confirm="t('common.remove')"
      :keep="t('config.games.keepGame')"
      @confirm="removeGame(toRemove)"
    />
  </header>
</template>

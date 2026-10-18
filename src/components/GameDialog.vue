<script setup>
// Creation or import of a game, in a modal over the Games tab: both are saved right away.
import { computed, ref, watch } from 'vue';
import { VueFinalModal } from 'vue-final-modal';
import { BOUNDS, byName, clone, MAX_NAME, newId, sameName } from '../configDraft.js';
import { t } from '../i18n.js';
import { importGame, parseGame } from '../share.js';
import { vReveal } from '../reveal.js';
import { vSelect } from '../select.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ShareImport from './ShareImport.vue';
import DisabledTip from './DisabledTip.vue';

const importing = defineModel('importing', { type: Boolean });
const { config, draft, editedGameId, newGame, notify } = useConfig();
const { saveNow, saveChange } = useSession();

const open = computed({
  get: () => !!newGame.value || importing.value,
  set: (value) => value || close(),
});
function close() {
  newGame.value = null;
  importing.value = false;
}

// Content kept until the next opening, so the modal does not empty itself while fading out.
const form = ref(null);
watch(newGame, (game) => game && (form.value = game), { immediate: true });
watch(importing, (on) => on && (form.value = null));

const nameTaken = computed(() => config.value.games.some((g) => sameName(g.name, form.value.name)));
// Why the new game cannot be created yet, '' when it can.
const createBlocked = computed(() => {
  if (!form.value.name.trim()) return t('config.games.nameNeeded');
  return nameTaken.value ? t('config.games.nameTaken') : '';
});

async function createGame() {
  const { name, source, steamAppId } = form.value;
  const model = draft.value.games.find((g) => g.id === source);
  const game = { id: newId(name, draft.value.games), name: name.trim(), events: model ? clone(model.events) : [] };
  if (steamAppId) game.steamAppId = steamAppId;
  const created = await saveChange((cfg) => {
    cfg.games.push(clone(game));
  });
  if (!created) return;
  editedGameId.value = game.id;
  close();
}

const takenName = (name) => config.value.games.some((g) => sameName(g.name, name));

async function importShared(game) {
  if (!(await saveNow({ ...config.value, games: importGame(config.value.games, game) }))) return;
  draft.value.games = importGame(draft.value.games, clone(game));
  editedGameId.value = config.value.games.find((g) => sameName(g.name, game.name)).id;
  close();
  notify(game.clamped ? t('config.games.importedClamped', { name: game.name }, BOUNDS.draws.max) : t('config.games.imported', { name: game.name }));
}
</script>

<template>
  <VueFinalModal
    v-model="open"
    class="confirm-modal"
    content-class="confirm-modal-content form-dialog"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
    :focus-trap="{ initialFocus: '.form-dialog :is(input, textarea)' }"
  >
    <button class="modal-close" :aria-label="t('common.close')" @click="close">×</button>
    <template v-if="form">
      <h3>{{ t('config.games.new') }}</h3>
      <form @submit.prevent="createGame">
        <div class="game-fields">
          <span>{{ t('common.name') }}</span>
          <input v-model="form.name" :maxlength="MAX_NAME" :aria-label="t('config.games.newName')" />
          <span>{{ t('config.games.copyEvents') }}</span>
          <select v-select v-model="form.source" :aria-label="t('config.games.newEvents')">
            <option value="">{{ t('config.games.noCopy') }}</option>
            <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">{{ g.name || t('common.unnamed') }}</option>
          </select>
        </div>
        <p v-if="nameTaken" v-reveal>{{ t('config.games.nameTaken') }}</p>
        <div class="card-actions">
          <button type="button" class="ghost" @click="close">{{ t('common.cancel') }} <kbd>{{ t('keys.escape') }}</kbd></button>
          <DisabledTip :tip="createBlocked">
            <button class="primary" :disabled="!!createBlocked">{{ t('config.games.create') }}</button>
          </DisabledTip>
        </div>
      </form>
    </template>
    <template v-else>
      <h3>{{ t('config.games.import') }}</h3>
      <ShareImport v-slot="{ result }" :parse="parseGame" @import="importShared" @cancel="close">
        <p v-if="takenName(result.name)" v-reveal class="banner warn">{{ t('config.games.willReplace', { name: result.name }) }}</p>
      </ShareImport>
    </template>
  </VueFinalModal>
</template>

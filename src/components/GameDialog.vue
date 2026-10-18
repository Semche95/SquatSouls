<script setup>
// Creation or import of a game, in a modal over the Games tab: both are saved right away.
import { computed, ref, shallowRef, watch } from 'vue';
import { VueFinalModal } from 'vue-final-modal';
import { BOUNDS, byName, clone, MAX_NAME, newId, sameName } from '../configDraft.js';
import { t } from '../i18n.js';
import { parseGame } from '../share.js';
import { eventChanges, importGame } from '../importChanges.js';
import { eventSummary } from '../itemSummary.js';
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

// How the shared game `game` is imported, once changed from its defaults: replacing the game whose id is
// `target`, or as a new game under `name`.
const choice = shallowRef(null);
// Content kept until the next opening, so the modal does not empty itself while fading out.
const form = ref(null);
watch(newGame, (game) => game && (form.value = game), { immediate: true });
watch(importing, (on) => on && ((form.value = null), (choice.value = null)));

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

const existingGame = (name) => config.value.games.find((g) => sameName(g.name, name));
const takenName = (name) => !!existingGame(name);
const replaceBlocked = () => (config.value.games.length ? '' : t('config.games.noReplace'));

// First name free among "Sekiro 2", "Sekiro 3"…, shortened to fit the length limit.
function freeName(name) {
  for (let n = 2; ; n++) {
    const suffix = ` ${n}`;
    const free = [...name].slice(0, MAX_NAME - suffix.length).join('') + suffix;
    if (!takenName(free)) return free;
  }
}
// By default the game with the same name is replaced, otherwise the game is imported as new under its name.
function choiceOf(game) {
  if (choice.value?.game === game) return choice.value;
  const existing = existingGame(game.name);
  const target = existing ?? byName(config.value.games)[0];
  return { game, replace: !!existing, target: target?.id ?? '', name: existing ? freeName(game.name) : game.name };
}
const choose = (game, change) => (choice.value = { ...choiceOf(game), ...change });
const targetOf = (game) => (choiceOf(game).replace ? choiceOf(game).target : undefined);
const importLine = (game) => {
  if (takenName(game.name)) return t('config.games.exists', { name: game.name });
  return choiceOf(game).replace ? game.name : t('config.games.importNew', { name: game.name });
};

// As a new game, all its events are new.
const changesOf = (game) => eventChanges(config.value.games.find((g) => g.id === targetOf(game)), game);
function importBlocked(game) {
  const { replace, name } = choiceOf(game);
  if (replace) return '';
  if (!name.trim()) return t('config.games.nameNeeded');
  return takenName(name) ? t('config.games.nameTaken') : '';
}

async function importShared(shared, chosen) {
  const target = targetOf(shared);
  const game = target ? shared : { ...shared, name: choiceOf(shared).name.trim() };
  const games = importGame(config.value.games, game, chosen, target);
  if (!(await saveNow({ ...config.value, games }))) return;
  draft.value.games = clone(games);
  const imported = target ? games.find((g) => g.id === target) : games.at(-1);
  editedGameId.value = imported.id;
  close();
  const { name } = imported;
  const clamped = game.clamped.some((key) => chosen.has(key));
  notify(clamped ? t('config.games.importedClamped', { name }, BOUNDS.draws.max) : t('config.games.imported', { name }));
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
      <ShareImport v-slot="{ result }" :parse="parseGame" :changes="changesOf" :summary="eventSummary" :blocked="importBlocked" @import="importShared" @cancel="close">
        <div class="import-mode">
          <span id="import-game">{{ importLine(result) }}</span>
          <div class="segmented" role="radiogroup" aria-labelledby="import-game">
            <DisabledTip :tip="replaceBlocked()">
              <button
                type="button"
                role="radio"
                :disabled="!!replaceBlocked()"
                :aria-checked="choiceOf(result).replace"
                :class="{ active: choiceOf(result).replace }"
                @click="choose(result, { replace: true })"
              >
                {{ t('config.games.replace') }}
              </button>
            </DisabledTip>
            <button type="button" role="radio" :aria-checked="!choiceOf(result).replace" :class="{ active: !choiceOf(result).replace }" @click="choose(result, { replace: false })">
              {{ t('config.games.importAsNew') }}
            </button>
          </div>
        </div>
        <div class="game-fields">
          <template v-if="choiceOf(result).replace">
            <span>{{ t('config.games.replaceTarget') }}</span>
            <select v-select :value="choiceOf(result).target" :aria-label="t('config.games.replaceTarget')" @change="choose(result, { target: $event.target.value })">
              <option v-for="g in byName(config.games)" :key="g.id" :value="g.id">{{ g.name }}</option>
            </select>
          </template>
          <template v-else>
            <span>{{ t('common.name') }}</span>
            <input
              :value="choiceOf(result).name"
              :maxlength="MAX_NAME"
              :aria-label="t('config.games.importName')"
              :aria-invalid="!!importBlocked(result)"
              @input="choose(result, { name: $event.target.value })"
            />
          </template>
        </div>
        <p v-if="importBlocked(result)" v-reveal>{{ importBlocked(result) }}</p>
      </ShareImport>
    </template>
  </VueFinalModal>
</template>

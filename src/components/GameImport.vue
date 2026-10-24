<script setup>
// Import of a shared game, saved right away: replacing a game or as a new one. Emits `close` once imported or cancelled.
import { shallowRef } from 'vue';
import { BOUNDS, byName, clone, MAX_NAME, sameName } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { parseGame } from '../sharing/share.js';
import { eventChanges, importGame } from '../sharing/importChanges.js';
import { eventSummary } from '../config/itemSummary.js';
import { vReveal } from '../ui/reveal.js';
import { vSelect } from '../ui/select.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ShareImport from './ShareImport.vue';
import DisabledTip from './DisabledTip.vue';

const emit = defineEmits(['close']);
const { config, draft, editedGameId, notify } = useConfig();
const { saveNow } = useSession();

// Saved games as the import opened: saving the imported game does not change what it shows while the modal fades out.
const games = config.value.games;

// How the shared game `game` is imported, once changed from its defaults: replacing the game whose id is
// `target`, or as a new game under `name`.
const choice = shallowRef(null);

const existingGame = (name) => games.find((g) => sameName(g.name, name));
const takenName = (name) => !!existingGame(name);
const replaceBlocked = () => (games.length ? '' : t('config.games.noReplace'));

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
  const target = existing ?? byName(games)[0];
  return { game, replace: !!existing, target: target?.id ?? '', name: existing ? freeName(game.name) : game.name };
}
const choose = (game, change) => (choice.value = { ...choiceOf(game), ...change });
const targetOf = (game) => (choiceOf(game).replace ? choiceOf(game).target : undefined);
const importLine = (game) => (takenName(game.name) ? t('config.games.exists', { name: game.name }) : game.name);

// As a new game, all its events are new.
const changesOf = (game) => eventChanges(games.find((g) => g.id === targetOf(game)), game);
function importBlocked(game) {
  const { replace, name } = choiceOf(game);
  if (replace) return '';
  if (!name.trim()) return t('config.games.nameNeeded');
  return takenName(name) ? t('config.games.nameTaken') : '';
}

async function importShared(shared, chosen) {
  const target = targetOf(shared);
  const game = target ? shared : { ...shared, name: choiceOf(shared).name.trim() };
  const saved = importGame(config.value.games, game, chosen, target);
  if (!(await saveNow({ ...config.value, games: saved }))) return;
  draft.value.games = clone(saved);
  const imported = target ? saved.find((g) => g.id === target) : saved.at(-1);
  editedGameId.value = imported.id;
  emit('close');
  const { name } = imported;
  const clamped = game.clamped.some((key) => chosen.has(key));
  notify(clamped ? t('config.games.importedClamped', { name }, BOUNDS.draws.max) : t('config.games.imported', { name }));
}
</script>

<template>
  <ShareImport :parse="parseGame" :changes="changesOf" :summary="eventSummary" :blocked="importBlocked" @import="importShared" @cancel="emit('close')">
    <template #default="{ result }">
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
            <option v-for="g in byName(games)" :key="g.id" :value="g.id">{{ g.name }}</option>
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
    </template>
  </ShareImport>
</template>

<script setup>
import { ref } from 'vue';
import { byName, newId } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { vSelect } from '../ui/select.js';
import { wide } from '../ui/wide.js';
import { useConfig } from '../composables/useConfig.js';
import { provideEditDialog } from '../composables/useEditDialog.js';
import ConfigEvent from './ConfigEvent.vue';
import EventDialog from './EventDialog.vue';
import GameDialog from './GameDialog.vue';
import GameHead from './GameHead.vue';
import GameList from './GameList.vue';
import DisabledTip from './DisabledTip.vue';

const { draft, editedGameId, editedGame, newGame, shareBlocked } = useConfig();
// Import of a shared game, in the same modal as creation.
const importing = ref(false);
// Event in its edit dialog.
const { item: editing, edit: editEvent } = provideEditDialog(() => editedGame.value.events);

function openNewGame() {
  newGame.value = { name: '', source: '' };
}

// Event added to fill in: one exercise per set, no joker.
function addEvent(game) {
  const name = t('config.games.newEvent');
  game.events.push({ id: newId(name, game.events), name, draws: 1, jokers: 0 });
  editEvent(game.events.at(-1), true);
}
</script>

<template>
  <!-- The wide window lists every game beside the edited one, the narrow one chooses it in a select above. -->
  <section class="group" :class="{ 'cfg-games-wide': wide }">
    <GameList v-if="wide" @new="openNewGame" @import="importing = true" />
    <header v-else class="row cfg-game-bar">
      <div v-if="draft.games.length" class="field">
        <select id="cfg-game" v-select="{ search: t('config.games.filter'), none: t('config.games.noMatch') }" :value="editedGame?.id ?? ''" :aria-label="t('play.game')" @change="editedGameId = $event.target.value">
          <option v-if="!editedGame" value="" disabled>{{ t('play.chooseGame') }}</option>
          <option v-for="g in byName(draft.games)" :key="g.id" :value="g.id">{{ g.name }}</option>
        </select>
      </div>
      <button class="primary" @click="openNewGame">{{ t('config.games.new') }}</button>
      <DisabledTip :tip="shareBlocked">
        <button :disabled="!!shareBlocked" @click="importing = true">{{ t('common.import') }}</button>
      </DisabledTip>
    </header>
    <div class="cfg-game">
      <p v-if="!draft.games.length" class="empty">{{ t('config.games.none') }}</p>
      <template v-if="editedGame">
        <GameHead />
        <h2 class="setting-title">{{ t('config.games.events') }}</h2>
        <div class="tiles cfg-events">
          <button class="tile tile-new" @click="addEvent(editedGame)">{{ t('config.games.addEvent') }}</button>
          <ConfigEvent v-for="ev in editedGame.events" :key="ev.id" :ev="ev" @click="editEvent(ev)" />
        </div>
      </template>
    </div>
    <EventDialog v-if="editing" />
    <GameDialog v-model:importing="importing" />
  </section>
</template>

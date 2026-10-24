<script setup>
// Games of the wide window, all listed with their number of events to choose the edited one, then the catalog first,
// their creation and import.
import { ref, watch } from 'vue';
import { nameMatches } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { useConfig } from '../composables/useConfig.js';
import DisabledTip from './DisabledTip.vue';
import GamePicker from './GamePicker.vue';

defineEmits(['new', 'import', 'catalog']);
const { draft, editedGameId, editedGame, shareBlocked } = useConfig();
const filter = ref('');

// The edited game always stays in the list, whatever chose it (creation, import): a filter hiding it is cleared.
watch(
  editedGame,
  (game) => {
    if (game && !nameMatches(game.name, filter.value)) filter.value = '';
  },
  { immediate: true },
);
</script>

<template>
  <aside class="cfg-games">
    <GamePicker v-if="draft.games.length" v-slot="{ game }" v-model:filter="filter" :games="draft.games" :label="t('play.game')" :active="editedGame" @pick="editedGameId = $event.id">
      <span class="cfg-game-count" :aria-label="t('config.games.eventCount', game.events.length)">{{ game.events.length }}</span>
    </GamePicker>
    <DisabledTip :tip="shareBlocked">
      <button class="primary" :disabled="!!shareBlocked" @click="$emit('catalog')">{{ t('config.catalog.open') }}</button>
    </DisabledTip>
    <button class="tile tile-new" @click="$emit('new')">{{ t('config.games.addGame') }}</button>
    <DisabledTip :tip="shareBlocked">
      <button class="ghost" :disabled="!!shareBlocked" @click="$emit('import')">{{ t('config.games.import') }}</button>
    </DisabledTip>
  </aside>
</template>

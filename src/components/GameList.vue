<script setup>
// Games of the wide window, all listed with their number of events to choose the edited one, then their creation and import.
import { computed, ref, watch } from 'vue';
import { byName, nameMatches } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { useConfig } from '../composables/useConfig.js';
import DisabledTip from './DisabledTip.vue';

defineEmits(['new', 'import']);
const { draft, editedGameId, editedGame, shareBlocked } = useConfig();
const filter = ref('');
const shown = computed(() => byName(draft.value.games).filter((g) => nameMatches(g.name, filter.value)));

// The edited game always stays in the list, whatever chose it (creation, import): a filter hiding it is cleared.
watch(
  editedGame,
  (game) => {
    if (game && !shown.value.includes(game)) filter.value = '';
  },
  { immediate: true },
);
</script>

<template>
  <aside class="cfg-games">
    <div v-if="draft.games.length" class="cfg-game-list">
      <input v-model="filter" class="cfg-game-filter" :placeholder="t('config.games.filter')" :aria-label="t('config.games.filter')" />
      <nav :aria-label="t('play.game')">
        <button v-for="g in shown" :key="g.id" :class="{ active: g === editedGame }" :aria-current="g === editedGame" @click="editedGameId = g.id">
          <span class="cfg-game-name">{{ g.name }}</span>
          <span class="cfg-game-count" :aria-label="t('config.games.eventCount', g.events.length)">{{ g.events.length }}</span>
        </button>
        <p v-if="!shown.length" class="cfg-game-none">{{ t('config.games.noMatch') }}</p>
      </nav>
    </div>
    <button class="tile tile-new" @click="$emit('new')">{{ t('config.games.addGame') }}</button>
    <DisabledTip :tip="shareBlocked">
      <button class="ghost" :disabled="!!shareBlocked" @click="$emit('import')">{{ t('config.games.import') }}</button>
    </DisabledTip>
  </aside>
</template>

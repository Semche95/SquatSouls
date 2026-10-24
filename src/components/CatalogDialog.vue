<script setup>
// Catalog of games, in a modal over the Games tab: its games filtered by name, those already in the program
// marked, then the import of the chosen one, the way back to the list in place of the paste field.
import { ref, watch } from 'vue';
import { VueFinalModal } from 'vue-final-modal';
import { sameName } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { useCatalog } from '../composables/useCatalog.js';
import { useConfig } from '../composables/useConfig.js';
import GameImport from './GameImport.vue';
import GamePicker from './GamePicker.vue';

const { config } = useConfig();
const { games, error, open, picked, load } = useCatalog();
const filter = ref('');
const added = (game) => config.value.games.some((g) => sameName(g.name, game.name));

// The filter is emptied each time the modal opens, and focused once the list is shown.
watch(open, (on) => on && (filter.value = ''));
const picker = ref(null);
watch(picker, (list) => list?.focus());
</script>

<template>
  <VueFinalModal
    v-model="open"
    class="confirm-modal"
    content-class="confirm-modal-content form-dialog"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
    :focus-trap="{ initialFocus: '.form-dialog .cfg-import :is(input, button)' }"
  >
    <button class="modal-close" :aria-label="t('common.close')" @click="open = false">×</button>
    <h3>{{ t('config.catalog.open') }}</h3>
    <GameImport v-if="picked" :key="picked.id" :shared="picked.text" @close="open = false">
      <template #source>
        <button type="button" class="ghost catalog-back" @click="picked = null">{{ t('config.catalog.back') }}</button>
      </template>
    </GameImport>
    <div v-else class="cfg-import">
      <p v-if="!games && !error" role="status">{{ t('config.catalog.loading') }}</p>
      <p v-else-if="error" class="banner error" role="alert">{{ error }}</p>
      <template v-else>
        <p>{{ t('config.catalog.intro') }}</p>
        <GamePicker ref="picker" v-slot="{ game }" v-model:filter="filter" :games="games" :label="t('config.catalog.open')" @pick="picked = $event">
          <span class="cfg-game-count">{{ t(added(game) ? 'config.catalog.added' : 'config.games.eventCount', game.events.length) }}</span>
        </GamePicker>
      </template>
      <div class="row">
        <button v-if="error" type="button" class="primary" @click="load">{{ t('startup.retry') }}</button>
        <button type="button" class="ghost" @click="open = false">{{ t('common.cancel') }} <kbd>{{ t('keys.escape') }}</kbd></button>
      </div>
    </div>
  </VueFinalModal>
</template>

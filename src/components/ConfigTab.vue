<script setup>
import { onBeforeUnmount } from 'vue';
import { clone } from '../configDraft.js';
import { t } from '../i18n.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ConfigExercises from './ConfigExercises.vue';
import ConfigGames from './ConfigGames.vue';
import ConfigSettings from './ConfigSettings.vue';
import DisabledTip from './DisabledTip.vue';

const { config, draft, configError, configMsg, notify, cfgTab, newGame, dirty, draftError } = useConfig();
const { saveNow } = useSession();
onBeforeUnmount(() => notify(''));

async function saveConfig() {
  notify('');
  if (!(await saveNow(draft.value))) return;
  draft.value = clone(config.value);
  notify(t('config.saved'));
}

function discardChanges() {
  draft.value = clone(config.value);
  newGame.value = null;
  notify('');
}
</script>

<template>
  <main class="panel config">
    <p v-if="configError" class="banner error">{{ configError }}</p>
    <p v-if="configMsg" class="banner ok">{{ configMsg }}</p>

    <template v-if="draft">
      <nav class="subtabs">
        <button :class="{ active: cfgTab === 'games' }" @click="cfgTab = 'games'">{{ t('config.tabs.games') }}</button>
        <button :class="{ active: cfgTab === 'exercises' }" @click="cfgTab = 'exercises'">{{ t('config.tabs.exercises') }}</button>
        <button :class="{ active: cfgTab === 'settings' }" @click="cfgTab = 'settings'">{{ t('config.tabs.settings') }}</button>
      </nav>
      <ConfigExercises v-if="cfgTab === 'exercises'" />
      <ConfigSettings v-else-if="cfgTab === 'settings'" />
      <ConfigGames v-else />
    </template>

    <div v-if="dirty" class="row cfg-actions">
      <DisabledTip :tip="draftError" error>
        <button class="primary" :disabled="!!draftError" @click="saveConfig">{{ t('config.save') }}</button>
      </DisabledTip>
      <button class="ghost" @click="discardChanges">{{ t('config.discard') }}</button>
    </div>
  </main>
</template>

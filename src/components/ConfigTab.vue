<script setup>
import { onBeforeUnmount } from 'vue';
import { clone } from '../configDraft.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ConfigExercises from './ConfigExercises.vue';
import ConfigGames from './ConfigGames.vue';

const { config, draft, configError, configMsg, notify, cfgTab, newGame, dirty } = useConfig();
const { saveNow } = useSession();
onBeforeUnmount(() => notify(''));

async function saveConfig() {
  notify('');
  if (!(await saveNow(draft.value))) return;
  draft.value = clone(config.value);
  notify('Configuration enregistrée.');
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
        <button :class="{ active: cfgTab === 'jeux' }" @click="cfgTab = 'jeux'">Jeux</button>
        <button :class="{ active: cfgTab === 'exercices' }" @click="cfgTab = 'exercices'">Exercices</button>
      </nav>
      <ConfigExercises v-if="cfgTab === 'exercices'" />
      <ConfigGames v-else />
    </template>

    <div v-if="dirty" class="row cfg-actions">
      <button class="primary" @click="saveConfig">Enregistrer</button>
      <button class="ghost" @click="discardChanges">Réinitialiser les modifications</button>
    </div>
  </main>
</template>

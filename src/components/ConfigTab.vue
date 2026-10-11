<script setup>
import { onBeforeUnmount, ref } from "vue";
import { clone } from "../configDraft.js";
import { useConfig } from "../composables/useConfig.js";
import { useSession } from "../composables/useSession.js";
import ConfigExercises from "./ConfigExercises.vue";
import ConfigGames from "./ConfigGames.vue";

const { config, draft, configError, cfgTab, newGame, dirty, store } = useConfig();
const { reload } = useSession();

const configMsg = ref("");
// Durée d'affichage de la confirmation d'enregistrement.
const CONFIG_MSG_MS = 3000;
let configMsgTimer;
onBeforeUnmount(() => clearTimeout(configMsgTimer));

async function saveConfig() {
  configMsg.value = "";
  if (!(await store(draft.value))) return;
  draft.value = clone(config.value);
  configMsg.value = "Configuration enregistrée.";
  clearTimeout(configMsgTimer);
  configMsgTimer = setTimeout(() => (configMsg.value = ""), CONFIG_MSG_MS);
  await reload();
}

function discardChanges() {
  draft.value = clone(config.value);
  newGame.value = null;
  configMsg.value = "";
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

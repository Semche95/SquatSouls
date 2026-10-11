<script setup>
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { pickSlogan } from "./slogans.js";
import { formatTime } from "./format.js";
import { provideApi } from "./composables/useApi.js";
import { provideConfig } from "./composables/useConfig.js";
import { provideSession } from "./composables/useSession.js";
import PlayTab from "./components/PlayTab.vue";
import BilanTab from "./components/BilanTab.vue";
import ConfigTab from "./components/ConfigTab.vue";

const tab = ref("jouer");
// Tiré une fois au lancement, affiché sous le titre.
const slogan = pickSlogan();

const api = provideApi();
const { error, call } = api;
const cfg = provideConfig();
const { configError, gameId, applyConfigView } = cfg;
const { session } = provideSession(api, cfg);

onMounted(async () => {
  try {
    // Lu avant la config : un jeu inconnu y est ensuite remis à vide.
    gameId.value = await invoke("get_last_game").catch(() => "");
    applyConfigView(await call("get_config"));
    session.value = await call("get_session");
  } catch {
    /* erreur déjà affichée */
  }
});
</script>

<template>
  <div class="app">
    <header class="top">
      <div class="brand">
        <h1>Squat Souls</h1>
        <p v-if="slogan" class="splash">{{ slogan }}</p>
      </div>
      <nav class="tabs">
        <button :class="{ active: tab === 'jouer' }" @click="tab = 'jouer'">Jouer</button>
        <button :class="{ active: tab === 'seance' }" @click="tab = 'seance'">
          Bilan <span v-if="session.sets" class="badge">{{ session.sets }}</span>
        </button>
        <button :class="{ active: tab === 'config' }" @click="tab = 'config'">Configuration</button>
      </nav>
      <dl class="hud">
        <div>
          <dt>Séries faites</dt>
          <dd>{{ session.sets }}</dd>
        </div>
        <div>
          <dt>Jokers</dt>
          <dd>{{ session.jokers }}</dd>
        </div>
        <div>
          <dt>Séance depuis</dt>
          <dd>{{ session.startedAt ? formatTime(session.startedAt) : "–" }}</dd>
        </div>
      </dl>
    </header>

    <p v-if="error" class="banner error">{{ error }}</p>
    <p v-if="configError && tab !== 'config'" class="banner error">
      {{ configError }}
      <button class="link" @click="tab = 'config'">Voir la configuration</button>
    </p>

    <PlayTab v-if="tab === 'jouer'" />
    <BilanTab v-else-if="tab === 'seance'" />
    <ConfigTab v-else />
  </div>
</template>

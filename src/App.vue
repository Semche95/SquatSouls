<script setup>
import { onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { pickSlogan } from './slogans.js';
import { formatTime } from './format.js';
import { provideApi } from './composables/useApi.js';
import { provideConfig } from './composables/useConfig.js';
import { provideSession } from './composables/useSession.js';
import { provideSteam } from './composables/useSteam.js';
import PlayTab from './components/PlayTab.vue';
import SummaryTab from './components/SummaryTab.vue';
import ConfigTab from './components/ConfigTab.vue';

// Picked once at launch, shown under the title.
const slogan = pickSlogan();

const api = provideApi();
const { error, call } = api;
const cfg = provideConfig();
const { configError, gameId, tab, applyConfigView } = cfg;
const { session, saveChange } = provideSession(api, cfg);
provideSteam(cfg, { saveChange });

onMounted(async () => {
  try {
    // Read before the config: an unknown game is then reset to empty.
    gameId.value = await invoke('get_last_game').catch(() => '');
    applyConfigView(await call('get_config'));
    session.value = await call('get_session');
  } catch {
    /* error already shown */
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
        <button :class="{ active: tab === 'play' }" @click="tab = 'play'">Jouer</button>
        <button :class="{ active: tab === 'summary' }" @click="tab = 'summary'">
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
          <dd>{{ session.startedAt ? formatTime(session.startedAt) : '–' }}</dd>
        </div>
      </dl>
    </header>

    <p v-if="error" class="banner error">{{ error }}</p>
    <p v-if="configError && tab !== 'config'" class="banner error">
      {{ configError }}
      <button class="link" @click="tab = 'config'">Voir la configuration</button>
    </p>

    <PlayTab v-if="tab === 'play'" />
    <SummaryTab v-else-if="tab === 'summary'" />
    <ConfigTab v-else />
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { pickSlogan } from './slogans.js';
import { t } from './i18n.js';
import { formatTime } from './format.js';
import { vReveal } from './reveal.js';
import { provideApi } from './composables/useApi.js';
import { provideConfig } from './composables/useConfig.js';
import { provideSession } from './composables/useSession.js';
import { provideSteam } from './composables/useSteam.js';
import PlayTab from './components/PlayTab.vue';
import HistoryTab from './components/HistoryTab.vue';
import ProgramTab from './components/ProgramTab.vue';
import ConfigFrame from './components/ConfigFrame.vue';
import ConfigSettings from './components/ConfigSettings.vue';
import QuitDialog from './components/QuitDialog.vue';
import AboutDialog from './components/AboutDialog.vue';
import OnboardingDialog from './components/OnboardingDialog.vue';
import UpdateBanner from './components/UpdateBanner.vue';

// Picked once at launch, shown under the title.
const slogan = pickSlogan();

const about = ref(false);
// The damaged database was replaced by the copy of the previous launch: told until closed.
const restored = ref(false);

const api = provideApi();
const { error, call } = api;
const cfg = provideConfig();
const { config, configError, gameId, intensity, tab, applyConfigView } = cfg;
const { session, saveChange } = provideSession(api, cfg);
provideSteam(cfg, { saveChange });

onMounted(async () => {
  restored.value = await invoke('was_restored').catch(() => false);
  try {
    // Read before the config: an unknown game is then reset to empty.
    gameId.value = await invoke('get_last_game').catch(() => '');
    applyConfigView(await call('get_config'));
    if (config.value.rememberIntensity) intensity.value = await invoke('get_intensity').catch(() => 'normal');
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
        <button :class="{ active: tab === 'play' }" @click="tab = 'play'">{{ t('app.tabs.play') }}</button>
        <button :class="{ active: tab === 'history' }" @click="tab = 'history'">
          {{ t('app.tabs.history') }} <span v-if="session.sets" class="badge">{{ session.sets }}</span>
        </button>
        <button :class="{ active: tab === 'program' }" @click="tab = 'program'">{{ t('app.tabs.program') }}</button>
        <button :class="{ active: tab === 'settings' }" @click="tab = 'settings'">{{ t('app.tabs.settings') }}</button>
        <button class="about" @click="about = true">{{ t('app.about') }}</button>
      </nav>
      <dl class="hud">
        <div>
          <dt>{{ t('common.setsDone') }}</dt>
          <dd>{{ session.sets }}</dd>
        </div>
        <div>
          <dt>{{ t('common.jokers') }}</dt>
          <dd>{{ session.jokers }}</dd>
        </div>
        <div>
          <dt>{{ t('app.sessionSince') }}</dt>
          <dd>{{ session.startedAt ? formatTime(session.startedAt) : '–' }}</dd>
        </div>
      </dl>
    </header>

    <div class="content">
      <UpdateBanner :locked="!!session.startedAt" />
      <p v-if="restored" class="banner warn closable">
        {{ t('app.restored') }}
        <button class="modal-close" :aria-label="t('common.close')" @click="restored = false">×</button>
      </p>
      <p v-if="error" v-reveal class="banner error">{{ error }}</p>
      <p v-if="configError && !['program', 'settings'].includes(tab)" class="banner error">
        {{ configError }}
        <button class="link" @click="tab = 'program'">{{ t('app.viewConfig') }}</button>
      </p>

      <PlayTab v-if="tab === 'play'" />
      <HistoryTab v-else-if="tab === 'history'" />
      <ProgramTab v-else-if="tab === 'program'" />
      <ConfigFrame v-else settings><ConfigSettings /></ConfigFrame>
    </div>
    <QuitDialog />
    <AboutDialog v-model="about" />
    <OnboardingDialog />
  </div>
</template>

<script setup>
import { t } from '../i18n.js';
import { useConfig } from '../composables/useConfig.js';
import ConfigExercises from './ConfigExercises.vue';
import ConfigGames from './ConfigGames.vue';
import ConfigFrame from './ConfigFrame.vue';

const { cfgTab } = useConfig();
</script>

<template>
  <ConfigFrame>
    <nav class="subtabs">
      <button :class="{ active: cfgTab === 'games' }" @click="cfgTab = 'games'">{{ t('config.tabs.games') }}</button>
      <button :class="{ active: cfgTab === 'exercises' }" @click="cfgTab = 'exercises'">{{ t('config.tabs.exercises') }}</button>
    </nav>
    <ConfigExercises v-if="cfgTab === 'exercises'" />
    <ConfigGames v-else />
  </ConfigFrame>
</template>

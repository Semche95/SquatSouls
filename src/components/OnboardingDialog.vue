<script setup>
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { VueFinalModal } from 'vue-final-modal';
import { rt, t, tm } from '../i18n/i18n.js';
import { useConfig } from '../composables/useConfig.js';

const { tab, cfgTab } = useConfig();

// The safety advice is shared with About.
const steps = computed(() => [
  tm('onboarding.intro'),
  tm('onboarding.yourProgram'),
  { title: tm('onboarding.safety'), lines: tm('safety.lines') },
  tm('onboarding.gitGud'),
]);
const step = ref(0);
const last = computed(() => step.value === steps.value.length - 1);

// Shown until closed once, and only left from its last step: the warnings must be read through.
const open = ref(false);
onMounted(async () => {
  open.value = !(await invoke('get_onboarding_seen').catch(() => true));
});

function closed() {
  invoke('set_onboarding_seen').catch(() => {});
}

// A new database has no game: they are the first thing to add.
function showProgram() {
  tab.value = 'program';
  cfgTab.value = 'games';
  open.value = false;
}
</script>

<template>
  <VueFinalModal
    v-model="open"
    class="onboarding"
    content-class="onboarding-screen"
    :esc-to-close="false"
    :click-to-close="false"
    hide-overlay
    content-transition="vfm-fade"
    @closed="closed"
  >
    <Transition name="step" mode="out-in">
      <div :key="step" class="onboarding-step">
        <h2>{{ rt(steps[step].title) }}</h2>
        <p v-for="(line, i) in steps[step].lines" :key="i">{{ rt(line) }}</p>
      </div>
    </Transition>
    <div class="onboarding-dots">
      <span v-for="(_, i) in steps" :key="i" :class="{ active: i === step }" />
    </div>
    <div class="onboarding-nav">
      <button class="ghost" :class="{ hidden: step === 0 }" @click="step--">{{ t('onboarding.back') }}</button>
      <div v-if="last" class="row">
        <button class="ghost" @click="showProgram">{{ t('onboarding.program') }}</button>
        <button class="primary" @click="open = false">{{ t('onboarding.start') }}</button>
      </div>
      <button v-else class="primary" @click="step++">{{ t('onboarding.next') }}</button>
    </div>
  </VueFinalModal>
</template>

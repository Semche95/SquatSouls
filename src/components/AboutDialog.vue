<script setup>
import { onMounted, ref } from 'vue';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { VueFinalModal } from 'vue-final-modal';
import { rt, t, tm } from '../i18n.js';
// src-tauri/capabilities/default.json only allows opening the repository URL written in package.json:
// when repository.url changes in package.json, change the allowed URL in src-tauri/capabilities/default.json too.
import { author, repository } from '../../package.json';
import icon from '../../src-tauri/icons/128x128.png';

const open = defineModel({ type: Boolean });

const version = ref('');
onMounted(async () => {
  version.value = await getVersion();
});
</script>

<template>
  <VueFinalModal
    v-model="open"
    class="confirm-modal"
    content-class="confirm-modal-content about-modal"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
  >
    <button class="modal-close" :aria-label="t('common.close')" @click="open = false">×</button>
    <header class="about-head">
      <img class="about-icon" :src="icon" alt="" />
      <h3>Squat Souls</h3>
      <p class="about-meta">{{ t('about.meta', { version, author }) }}</p>
    </header>
    <hr class="ornament" />
    <p class="about-lore">{{ t('about.description') }}</p>
    <section class="banner about-safety">
      <h4>{{ t('safety.title') }}</h4>
      <ul>
        <li v-for="(line, i) in tm('safety.lines')" :key="i">{{ rt(line) }}</li>
      </ul>
    </section>
    <hr class="ornament" />
    <footer class="about-foot">
      <button class="ghost about-source" @click="openUrl(repository.url)">
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path
            d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"
          />
        </svg>
        {{ t('about.sourceCode') }}
      </button>
      <p class="about-legal">{{ t('about.disclaimer') }}</p>
    </footer>
  </VueFinalModal>
</template>

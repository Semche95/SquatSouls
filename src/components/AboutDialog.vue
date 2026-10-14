<script setup>
import { onMounted, ref } from 'vue';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { VueFinalModal } from 'vue-final-modal';
import { t } from '../i18n.js';
// src-tauri/capabilities/default.json only allows opening the repository URL written in package.json:
// when repository.url changes in package.json, change the allowed URL in src-tauri/capabilities/default.json too.
import { author, repository } from '../../package.json';

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
    <button class="modal-close" :aria-label="t('about.close')" @click="open = false">×</button>
    <h3>Squat Souls</h3>
    <p>{{ t('about.description') }}</p>
    <p>{{ t('about.author', { author }) }}</p>
    <p>{{ t('about.version', { version }) }}</p>
    <a :href="repository.url" @click.prevent="openUrl(repository.url)">{{ t('about.sourceCode') }}</a>
  </VueFinalModal>
</template>

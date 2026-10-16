<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { errorText, t } from '../i18n.js';
import { issuesUrl, releaseUrl } from '../repository.js';
import { provideApi } from '../composables/useApi.js';
import UpdateBanner from './UpdateBanner.vue';

// Shown instead of the app when the database could not be opened: only the reason, the way out and the update.
const props = defineProps({ error: { type: Object, required: true } });

const { error: apiError } = provideApi();
// Read-only, used elsewhere or a full disk: fixed by the user, then tried again. Otherwise, a problem to report.
const fixable = ['database_denied', 'database_full'].includes(props.error.code);
// Data the version could not prepare: the fix comes with a newer one, fetched by hand while the banner shows none.
const notPrepared = props.error.code === 'database_not_prepared';
const update = ref(null);
</script>

<template>
  <main class="startup">
    <h1>Squat Souls</h1>
    <section class="card startup-failure">
      <h2>{{ t('startup.title') }}</h2>
      <p>{{ errorText(error) }}</p>
      <div class="card-actions">
        <button v-if="fixable" class="primary" @click="invoke('relaunch')">{{ t('startup.retry') }}</button>
        <button v-else class="primary" @click="openUrl(issuesUrl)">{{ t('startup.report') }}</button>
        <button v-if="notPrepared && !update" class="ghost" @click="openUrl(releaseUrl)">{{ t('startup.download') }}</button>
        <button class="ghost" @click="invoke('quit')">{{ t('native.quit') }}</button>
      </div>
    </section>
    <UpdateBanner v-model="update" />
    <p v-if="apiError" class="banner error">{{ apiError }}</p>
  </main>
</template>

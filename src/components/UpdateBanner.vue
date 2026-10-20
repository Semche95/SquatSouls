<script setup>
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import { t } from '../i18n/i18n.js';
import { releaseUrl } from '../project/repository.js';
import { useApi } from '../composables/useApi.js';
import DisabledTip from './DisabledTip.vue';

// During a session, the app is not restarted, and quitting does not install the update.
defineProps({ locked: Boolean });
// Stages whose hint tells the update installs on quitting, which waits for the end of the session.
const WAITS_FOR_SESSION = ['available', 'ready'];

// Update announced as soon as found, downloaded in the background, right away or on request, again on request
// after a failure, and installed on quitting outside a session, or, for the packages that cannot install it, downloaded by the user. Shared with the parent that binds it.
const update = defineModel({ type: Object, default: null });
const { call } = useApi();

const stage = computed(() => update.value?.stage);
// Unknown size: the bar sweeps instead of filling.
const width = computed(() => (update.value?.progress == null ? null : `${update.value.progress * 100}%`));

let unlisten = null;
onMounted(async () => {
  unlisten = await listen('update-changed', (e) => (update.value = e.payload));
  // Found before the interface listened.
  update.value ??= await invoke('get_update').catch(() => null);
});
onBeforeUnmount(() => unlisten?.());

function download() {
  call('download_update').catch(() => {});
}

function restart() {
  call('restart_to_update').catch(() => {});
}
</script>

<template>
  <div v-if="update" class="banner update">
    <svg class="update-icon" viewBox="0 0 24 24" aria-hidden="true">
      <path v-if="stage === 'ready'" d="M5 12.5l4.5 4.5L19 7.5" />
      <path v-else d="M12 4v12m-5-5l5 5 5-5M5 20h14" />
    </svg>
    <div class="update-text">
      <strong>{{ t(stage === 'ready' ? 'update.ready' : 'update.available', { version: update.version }) }}</strong>
      <span>{{ t(`update.${stage}Hint${locked && WAITS_FOR_SESSION.includes(stage) ? 'Session' : ''}`) }}</span>
    </div>
    <span v-if="stage === 'ready'" class="row">
      <DisabledTip :tip="locked ? t('update.restartLocked') : ''">
        <button class="primary" :disabled="locked" @click="restart">{{ t('update.restart') }}</button>
      </DisabledTip>
    </span>
    <span v-else-if="stage === 'available' || stage === 'failed'" class="row">
      <button class="primary" @click="download">{{ t(stage === 'failed' ? 'startup.retry' : 'update.update') }}</button>
    </span>
    <span v-else-if="stage === 'manual'" class="row">
      <button class="primary" @click="openUrl(releaseUrl)">{{ t('update.download') }}</button>
    </span>
    <span v-else class="edge-bar update-progress" :class="{ unknown: !width }"><span :style="{ width }" /></span>
  </div>
</template>

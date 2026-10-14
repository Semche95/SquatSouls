<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { t } from '../i18n.js';
import { useApi } from '../composables/useApi.js';
import { useSession } from '../composables/useSession.js';
import DisabledTip from './DisabledTip.vue';

// Version downloaded in the background, installed on quitting.
const version = ref(null);
const { call } = useApi();
const { session } = useSession();

let unlisten = null;
onMounted(async () => {
  unlisten = await listen('update-ready', (e) => (version.value = e.payload));
  // Downloaded before the interface listened.
  version.value ??= await invoke('get_update').catch(() => null);
});
onBeforeUnmount(() => unlisten?.());

function restart() {
  call('restart_to_update').catch(() => {});
}
</script>

<template>
  <p v-if="version" class="banner info update">
    <span>{{ t('update.ready', { version }) }}</span>
    <span class="row">
      <DisabledTip :tip="session.startedAt ? t('update.restartLocked') : ''">
        <button class="primary" :disabled="!!session.startedAt" @click="restart">{{ t('update.restart') }}</button>
      </DisabledTip>
    </span>
  </p>
</template>

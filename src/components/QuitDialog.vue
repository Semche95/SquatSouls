<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { VueFinalModal } from 'vue-final-modal';
import { t } from '../i18n.js';
import { useKeydown } from '../composables/useKeydown.js';

// Quitting asked during a session (shortcut, tray menu, window closed): awaits confirmation.
const asked = ref(false);

// e.key follows the layout: Q stays Q on AZERTY.
const isMac = navigator.userAgent?.includes('Mac');

// Ctrl+Q, or Cmd+Q on macOS, quits even when closing the window only hides it; a second one confirms.
useKeydown(
  (e) => {
    if ((isMac ? e.metaKey : e.ctrlKey) && e.key.toLowerCase() === 'q') {
      e.preventDefault();
      invoke(asked.value ? 'quit' : 'request_quit');
    }
  },
  { inModal: true },
);

let unlisten = null;
onMounted(async () => {
  unlisten = await listen('quit-requested', () => (asked.value = true));
});
onBeforeUnmount(() => unlisten?.());
</script>

<template>
  <VueFinalModal
    v-model="asked"
    class="confirm-modal"
    content-class="confirm-modal-content"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
  >
    <h3>{{ t('quit.title') }}</h3>
    <p>{{ t('quit.confirm') }}</p>
    <div class="card-actions">
      <button class="ghost" @click="asked = false">{{ t('session.keep') }} <kbd>{{ t('keys.escape') }}</kbd></button>
      <button class="danger" @click="invoke('quit')">{{ t('native.quit') }}</button>
    </div>
  </VueFinalModal>
</template>

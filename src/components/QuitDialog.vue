<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { t } from '../i18n.js';
import { useKeydown } from '../composables/useKeydown.js';
import ConfirmDialog from './ConfirmDialog.vue';

// Quitting asked during a session (shortcut, tray menu, window closed): awaits confirmation. Nothing is lost, the
// session resumes at the next launch.
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
  <ConfirmDialog
    v-model="asked"
    :title="t('quit.title')"
    :text="t('quit.confirm')"
    :confirm="t('native.quit')"
    :keep="t('quit.stay')"
    validate
    @confirm="invoke('quit')"
  />
</template>

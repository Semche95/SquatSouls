import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { errorText } from '../i18n/i18n.js';
import { defineShared } from './shared.js';

export const [provideApi, useApi] = defineShared(() => {
  const error = ref('');

  async function call(command, args) {
    error.value = '';
    try {
      return await invoke(command, args);
    } catch (e) {
      error.value = errorText(e);
      throw e;
    }
  }

  return { error, call };
});

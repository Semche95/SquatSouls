import { onUnmounted, ref } from 'vue';
import { t } from '../i18n/i18n.js';
import { useApi } from './useApi.js';

// How long a copy button reads "Copied".
const COPIED_MS = 2000;

// Copy of a summary to the clipboard, `copied` holding the key of the button that last copied, '' after a while.
export function useCopy() {
  const { error } = useApi();
  const copied = ref('');
  let timer;

  async function copy(key, text) {
    try {
      await navigator.clipboard.writeText(text);
      copied.value = key;
      clearTimeout(timer);
      timer = setTimeout(() => (copied.value = ''), COPIED_MS);
    } catch (e) {
      error.value = t('common.copyFailed', { error: String(e) });
    }
  }

  onUnmounted(() => clearTimeout(timer));
  return { copied, copy };
}

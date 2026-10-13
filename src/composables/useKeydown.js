import { onBeforeUnmount, onMounted } from 'vue';
import { useVfm } from 'vue-final-modal';

// Inactive while a modal is open, unless `inModal`: the modal's own shortcuts.
export function useKeydown(handler, { inModal = false } = {}) {
  const vfm = useVfm();
  const listener = (e) => {
    if (inModal || !vfm.openedModals.length) handler(e);
  };
  onMounted(() => window.addEventListener('keydown', listener));
  onBeforeUnmount(() => window.removeEventListener('keydown', listener));
}

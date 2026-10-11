import { onBeforeUnmount, onMounted } from "vue";

export function useKeydown(handler) {
  onMounted(() => window.addEventListener("keydown", handler));
  onBeforeUnmount(() => window.removeEventListener("keydown", handler));
}

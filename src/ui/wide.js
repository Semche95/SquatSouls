import { ref } from 'vue';

// Wide window, the same width as the layout of style/wide.css: for what changes structure, not only style, with it.
const query = window.matchMedia('(min-width: 900px)');
export const wide = ref(query.matches);
query.addEventListener('change', (e) => (wide.value = e.matches));

import { createApp } from 'vue';
import { createVfm } from 'vue-final-modal';
import App from './App.vue';
import { i18n, loadLanguage } from './i18n.js';
import { logError, logUncaught } from './log.js';
import { unlockSound } from './sound.js';
import 'floating-vue/dist/style.css';
import 'tom-select/dist/css/tom-select.css';
import 'vue-final-modal/style.css';
import './style.css';

logUncaught();
for (const type of ['pointerdown', 'keydown']) window.addEventListener(type, unlockSound, { capture: true });
// The webview's right-click menu (reload, inspect...) is only useful in development; text fields keep theirs for copy and paste.
if (!import.meta.env.DEV) {
  window.addEventListener('contextmenu', (e) => {
    if (!e.target.closest('input, textarea')) e.preventDefault();
  });
}
// Language known before mounting: no flash in the wrong language.
loadLanguage().then(() => {
  const app = createApp(App);
  app.config.errorHandler = (e, _, info) => logError(`Vue error (${info})`, e);
  app.use(i18n).use(createVfm()).mount('#app');
});

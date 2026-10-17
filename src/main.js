import { createApp, h } from 'vue';
import { createVfm } from 'vue-final-modal';
import { invoke } from '@tauri-apps/api/core';
import App from './App.vue';
import StartupFailure from './components/StartupFailure.vue';
import { i18n, loadLanguage, showLanguage } from './i18n.js';
import { logError, logUncaught } from './log.js';
import { unlockSound } from './sound.js';
import 'floating-vue/dist/style.css';
import 'tom-select/dist/css/tom-select.css';
import 'vue-final-modal/style.css';
import './theme.css';
import './style/index.css';

logUncaught();
for (const type of ['pointerdown', 'keydown']) window.addEventListener(type, unlockSound, { capture: true });
// The webview's right-click menu (reload, inspect...) is only useful in development; text fields keep theirs for copy and paste.
if (!import.meta.env.DEV) {
  window.addEventListener('contextmenu', (e) => {
    if (!e.target.closest('input, textarea')) e.preventDefault();
  });
}
// Database unusable at launch: the minimal mode, in the system language, the saved one being in the database.
// Otherwise, the language known before mounting: no flash in the wrong language.
async function root() {
  const failure = await invoke('get_startup_failure').catch(() => null);
  if (!failure) return loadLanguage().then(() => App);
  showLanguage(failure.language);
  return { render: () => h(StartupFailure, { error: failure.error }) };
}

root().then((component) => {
  const app = createApp(component);
  app.config.errorHandler = (e, _, info) => logError(`Vue error (${info})`, e);
  app.use(i18n).use(createVfm()).mount('#app');
});

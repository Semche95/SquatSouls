import { createApp } from 'vue';
import App from './App.vue';
import { i18n, loadLanguage } from './i18n.js';
import { logError, logUncaught } from './log.js';
import 'floating-vue/dist/style.css';
import './style.css';

logUncaught();
// Language known before mounting: no flash in the wrong language.
loadLanguage().then(() => {
  const app = createApp(App);
  app.config.errorHandler = (e, _, info) => logError(`Vue error (${info})`, e);
  app.use(i18n).mount('#app');
});

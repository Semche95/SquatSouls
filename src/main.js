import { createApp } from 'vue';
import App from './App.vue';
import { LOCALE } from './locale.js';
import 'floating-vue/dist/style.css';
import './style.css';

document.documentElement.lang = LOCALE;
createApp(App).mount('#app');

import { ref } from 'vue';
import { createI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import en from '../locales/en.json';
import fr from '../locales/fr.json';
import { logError } from '../logging/log.js';

// Language of a session without answer from the backend, and of every missing text.
const FALLBACK = 'en';

// French: 0 and 1 are singular. With three forms, the first one is for 0.
const frenchPlural = (n, forms) => (forms === 3 ? Math.min(n, 2) : n > 1 ? 1 : 0);

export const i18n = createI18n({
  legacy: false,
  locale: FALLBACK,
  fallbackLocale: FALLBACK,
  messages: { en, fr },
  pluralRules: { fr: frenchPlural },
});

export const { t, te, tm, rt } = i18n.global;
// Effective language, 'en' or 'fr'.
export const language = i18n.global.locale;
// Saved setting: 'system', 'en' or 'fr'.
export const languageSetting = ref('system');

// Texts of the tray menu and notifications, translated here: the backend has none. Notification
// texts keep `{name}`, replaced by the backend with the game's name.
const NATIVE_KEYS = ['show', 'quit', 'launchedTitle', 'launchedConfigured', 'launchedUnknown'];
export const nativeTexts = () => Object.fromEntries(NATIVE_KEYS.map((key) => [key, t(`native.${key}`, { name: '{name}' })]));

// Switches the interface only to `lang`.
export function showLanguage(lang) {
  language.value = lang;
  document.documentElement.lang = lang;
}

// Switches the interface, then the tray menu and notifications, to `lang`.
export async function setLanguage(lang) {
  showLanguage(lang);
  try {
    await invoke('set_native_texts', { texts: nativeTexts() });
  } catch (e) {
    logError('Native texts not sent', e);
  }
}

// Asks the backend for the language before mounting the app, English if it does not answer.
export async function loadLanguage() {
  try {
    const { setting, language: lang } = await invoke('get_language');
    languageSetting.value = setting;
    await setLanguage(lang);
  } catch {
    await setLanguage(FALLBACK);
  }
}

// Message of a backend error ({ code, params }, a nested error in `cause`), generic for an unknown code.
export function errorText(error) {
  const key = `errors.${error?.code}`;
  if (!te(key)) return t('errors.unknown');
  const params = { ...error.params };
  if (params.cause) params.cause = errorText(params.cause);
  return t(key, params);
}

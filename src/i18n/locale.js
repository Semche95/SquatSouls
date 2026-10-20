import { language } from './i18n.js';

// Steam Store API language names, by interface language.
const STEAM_LANGUAGES = { en: 'english', fr: 'french' };

// Language of the game names asked from the Steam Store: the interface's.
export const steamLanguage = () => STEAM_LANGUAGES[language.value];

import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { errorText, language, languageSetting, loadLanguage, setLanguage, t } from '../i18n.js';
import en from '../locales/en.json';
import fr from '../locales/fr.json';
import { config, rejection, setup, mountApp } from './app.js';

// Every text of a locale file, by dotted key.
const flat = (messages, prefix = '') =>
  Object.entries(messages).flatMap(([key, value]) =>
    typeof value === 'object' ? flat(value, `${prefix}${key}.`) : [[`${prefix}${key}`, value]],
  );
const placeholders = (text) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
const nativeCalls = () => invoke.mock.calls.filter(([cmd]) => cmd === 'set_native_texts').map(([, args]) => args.texts);

describe('locale files', () => {
  it('have the same keys, plural forms and placeholders, without empty texts', () => {
    const french = Object.fromEntries(flat(fr));
    expect(flat(en).map(([key]) => key)).toEqual(Object.keys(french));
    for (const [key, text] of flat(en)) {
      expect(text.trim(), key).not.toBe('');
      expect(french[key].trim(), key).not.toBe('');
      expect(text.split('|').length, key).toBe(french[key].split('|').length);
      expect(placeholders(text), key).toEqual(placeholders(french[key]));
    }
  });

  it('pluralize in both languages', async () => {
    expect([0, 1, 2].map((n) => t('common.joker', n))).toEqual(['0 jokers', '1 joker', '2 jokers']);
    expect([0, 1, 2].map((n) => t('play.jokers', n))).toEqual(['no jokers', '1 joker', '2 jokers']);
    expect(t('history.sets', 1)).toBe('1 set');
    await setLanguage('fr');
    expect([0, 1, 2].map((n) => t('common.joker', n))).toEqual(['0 joker', '1 joker', '2 jokers']);
    expect([0, 1, 2].map((n) => t('play.jokers', n))).toEqual(['aucun joker', '1 joker', '2 jokers']);
    expect(t('config.games.importedClamped', { name: 'Sekiro' }, 3)).toBe('Sekiro importé, limité à 3 exercices par événement.');
  });
});

describe('language loading', () => {
  it('applies the effective language given by the backend', async () => {
    setup({ get_language: { setting: 'system', language: 'fr' } });
    await loadLanguage();
    expect([language.value, languageSetting.value, document.documentElement.lang]).toEqual(['fr', 'system', 'fr']);
    setup({ get_language: { setting: 'en', language: 'en' } });
    await loadLanguage();
    expect([language.value, languageSetting.value, document.documentElement.lang]).toEqual(['en', 'en', 'en']);
  });

  it('falls back to English when the backend does not answer', async () => {
    setup({ get_language: rejection('internal') });
    await loadLanguage();
    expect(language.value).toBe('en');
  });

  it('sends the native texts at launch, and again on each language change', async () => {
    setup();
    await loadLanguage();
    expect(nativeCalls()).toEqual([
      {
        show: 'Show',
        quit: 'Quit',
        launchedTitle: '{name} launched',
        launchedConfigured: 'Select it in Play to start the session.',
        launchedUnknown: 'Game not configured yet: add it from Play.',
      },
    ]);
    await setLanguage('fr');
    expect(nativeCalls().at(-1)).toMatchObject({ show: 'Afficher', quit: 'Quitter', launchedTitle: '{name} lancé' });
  });

  it('asks the Steam Store for names in the interface language', async () => {
    await setLanguage('fr');
    setup({ get_config: { config: { ...config, steamDetection: true } }, running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    await mountApp();
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('steam_game_name', { appId: 367520, language: 'french' });
  });
});

describe('backend errors', () => {
  it('are shown from their code, with their parameters', () => {
    expect(errorText({ code: 'exercise_range', params: { name: 'Squats', min: 20, max: 15 } })).toBe('Invalid range for "Squats" (20 to 15).');
    expect(errorText({ code: 'not_enough_jokers', params: {} })).toBe('Not enough jokers.');
  });

  it('show their cause', () => {
    const error = { code: 'config_ignored', params: { cause: { code: 'game_name_duplicate', params: { name: 'Sekiro' } } } };
    expect(errorText(error)).toBe('Saved configuration ignored, default configuration used. Two games are named "Sekiro".');
  });

  it('get a generic message for an unknown code or a non-coded error', () => {
    expect(errorText({ code: 'from_the_future', params: {} })).toBe('Unexpected error.');
    expect(errorText('command not found')).toBe('Unexpected error.');
  });

  it('follow the language', async () => {
    await setLanguage('fr');
    expect(errorText({ code: 'unknown_game', params: { id: 'ds4' } })).toBe('Jeu inconnu\u00a0: ds4.');
  });
});

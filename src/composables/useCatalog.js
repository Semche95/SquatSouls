import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { newId } from '../config/configDraft.js';
import { errorText, language, t } from '../i18n/i18n.js';
import { parseGame } from '../sharing/share.js';
import { defineShared } from './shared.js';

// Catalog of games published with each release, fetched when first shown and kept for the effective language,
// and its modal over the Games tab: the list, then the import of the chosen game.
export const [provideCatalog, useCatalog] = defineShared(({ shareBlocked }) => {
  // Games of the catalog, read as pasted texts, each with its text: null until fetched.
  const games = ref(null);
  // Why the catalog could not be fetched, '' otherwise.
  const error = ref('');
  const open = ref(false);
  // Game whose import is shown, null for the list.
  const picked = ref(null);
  // Language of the fetched games, and the fetch running.
  let fetchedIn = null;
  let fetching = null;

  // Games unreadable by this version are left out: a catalog none of them is readable in comes from a newer one.
  async function fetchCatalog() {
    error.value = '';
    fetchedIn = language.value;
    try {
      const texts = await invoke('game_catalog');
      const read = texts.flatMap((text) => {
        try {
          const game = parseGame(text);
          return [{ ...game, id: newId(), text }];
        } catch {
          return [];
        }
      });
      if (texts.length && !read.length) throw t('config.catalog.newer');
      games.value = read;
    } catch (e) {
      error.value = typeof e === 'string' ? e : errorText(e);
    }
  }

  // Fetched again once the language changed, or after an error.
  function load() {
    if (fetchedIn !== language.value) games.value = null;
    if (games.value) return Promise.resolve();
    fetching ??= fetchCatalog().finally(() => (fetching = null));
    return fetching;
  }

  // Not over pending changes of Program: the import, saved right away, would have to merge with them.
  function browse() {
    if (shareBlocked.value) return;
    picked.value = null;
    open.value = true;
    load();
  }

  // Game of the catalog carrying the Steam AppId `appId`, undefined without one or without catalog.
  async function find(appId) {
    await load();
    return games.value?.find((g) => g.steamAppId === appId);
  }

  function show(game) {
    picked.value = game;
    open.value = true;
  }

  return { games, error, open, picked, load, browse, find, show };
});

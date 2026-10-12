import { onScopeDispose, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { byName, extendsName } from '../configDraft.js';
import { steamLanguage } from '../locale.js';
import { defineShared } from './shared.js';

// Interval between process scans when detection is enabled.
const SCAN_MS = 10000;
// Backend refusal when the Steam Store rate-limited requests: no more calls for a while.
const BLOCKED = 'blocked';

export const [provideSteam, useSteam] = defineShared(({ config, gameId, tab, cfgTab, newGame }, { saveChange }) => {
  // Running Steam game: `games` lists the configured games matching it (empty if it is unknown),
  // `name` is absent until the Steam Store has given it. Null without a running Steam game.
  const detected = ref(null);
  // The Steam Store rate-limited requests: unknown games are no longer offered.
  const storeBlocked = ref(false);
  let timer;
  // AppId of the last notified game, forgotten when no game runs anymore: a relaunched game is notified again.
  let notified = null;

  // Several games can carry the same AppId (the same game, for different challenges), and a game without AppId
  // whose name extends theirs is a variant of it. Without any, an ignored game keeps the name recorded when ignoring it;
  // otherwise the name comes from the Steam Store, and a configured game without AppId is recognized by that name.
  async function identify(appId) {
    const { games, ignoredGames } = config.value;
    const variants = (names) => games.filter((g) => g.steamAppId === undefined && names.some((n) => extendsName(g.name, n)));
    const tagged = byName(games.filter((g) => g.steamAppId === appId));
    if (tagged.length) return { appId, name: tagged[0].name, games: byName([...tagged, ...variants(tagged.map((g) => g.name))]) };
    const ignored = ignoredGames.find((g) => g.appId === appId);
    if (ignored) return { appId, name: ignored.name, games: [], ignored: true };
    let name;
    try {
      name = await invoke('steam_game_name', { appId, language: steamLanguage() });
      storeBlocked.value = false;
    } catch (e) {
      storeBlocked.value = e === BLOCKED;
    }
    const named = name === undefined ? [] : byName(variants([name]));
    return { appId, name: named[0]?.name ?? name, games: named, ignored: false };
  }

  // One notification per launch, once the name is known, except for an ignored game. Its texts are
  // those sent to the backend for the current language.
  async function notify(found) {
    if (found?.appId !== notified) notified = null;
    if (!found?.name || found.ignored || notified !== null || !config.value.steamNotify) return;
    notified = found.appId;
    await invoke('notify', { name: found.name, configured: found.games.length > 0 });
  }

  // Unreadable processes: nothing is detected, the next scan tries again.
  async function scan() {
    try {
      const appId = await invoke('running_steam_app');
      const found = appId ? await identify(appId) : null;
      if (!config.value.steamDetection) return;
      detected.value = found;
      await notify(found);
    } catch {
      detected.value = null;
    }
  }

  // Restarted on each saved configuration: an added or ignored game is taken into account right away.
  watch(
    config,
    (cfg) => {
      clearInterval(timer);
      detected.value = null;
      if (!cfg?.steamDetection) return;
      scan();
      timer = setInterval(scan, SCAN_MS);
    },
    { immediate: true },
  );
  onScopeDispose(() => clearInterval(timer));

  // The game recognized by its name receives the detected AppId, so it no longer depends on its name.
  async function selectDetected(game) {
    gameId.value = game.id;
    if (game.steamAppId !== undefined) return;
    const { appId } = detected.value;
    await saveChange((cfg) => {
      cfg.games.find((g) => g.id === game.id).steamAppId = appId;
    });
  }

  // Opens game creation, prefilled with the Steam game's name and AppId.
  function addGame({ appId, name }) {
    tab.value = 'config';
    cfgTab.value = 'games';
    newGame.value = { name, source: '', steamAppId: appId };
  }

  function ignoreDetected() {
    const { appId, name } = detected.value;
    return saveChange((cfg) => {
      cfg.ignoredGames.push({ appId, name });
    });
  }

  // Removed from the ignored games list, it will be offered again.
  function unignore(appId) {
    return saveChange((cfg) => {
      cfg.ignoredGames = cfg.ignoredGames.filter((g) => g.appId !== appId);
    });
  }

  return { detected, storeBlocked, selectDetected, addGame, ignoreDetected, unignore };
});

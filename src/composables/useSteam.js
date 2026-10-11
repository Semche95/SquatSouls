import { onScopeDispose, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { byName, extendsName } from '../configDraft.js';
import { defineShared } from './shared.js';

// Intervalle de lecture des processus quand la détection est activée.
const SCAN_MS = 10000;
// Refus du backend quand le Steam Store a limité les requêtes : plus aucun appel pendant un temps.
const BLOCKED = 'blocked';

export const [provideSteam, useSteam] = defineShared(({ config, gameId, tab, cfgTab, newGame }, { saveChange }) => {
  // Jeu Steam en cours : `games` liste les jeux configurés qui lui correspondent (vide s'il est inconnu),
  // `name` est absent tant que le Steam Store ne l'a pas donné. Null sans jeu Steam en cours.
  const detected = ref(null);
  // Le Steam Store a limité les requêtes : les jeux inconnus ne sont plus proposés.
  const storeBlocked = ref(false);
  let timer;

  // Plusieurs jeux peuvent porter le même AppId (un même jeu, pour des défis différents), et un jeu sans AppId
  // dont le nom reprend le leur en est une variante. Sans aucun, un jeu ignoré garde le nom noté en l'ignorant ;
  // sinon le nom vient du Steam Store, et un jeu configuré sans AppId se reconnaît à ce nom.
  async function identify(appId) {
    const { games, ignoredGames } = config.value;
    const variants = (names) => games.filter((g) => g.steamAppId === undefined && names.some((n) => extendsName(g.name, n)));
    const tagged = byName(games.filter((g) => g.steamAppId === appId));
    if (tagged.length) return { appId, name: tagged[0].name, games: byName([...tagged, ...variants(tagged.map((g) => g.name))]) };
    const ignored = ignoredGames.find((g) => g.appId === appId);
    if (ignored) return { appId, name: ignored.name, games: [], ignored: true };
    let name;
    try {
      name = await invoke('steam_game_name', { appId });
      storeBlocked.value = false;
    } catch (e) {
      storeBlocked.value = e === BLOCKED;
    }
    const named = name === undefined ? [] : byName(variants([name]));
    return { appId, name: named[0]?.name ?? name, games: named, ignored: false };
  }

  // Processus illisibles : rien n'est détecté, le prochain passage réessaie.
  async function scan() {
    try {
      const appId = await invoke('running_steam_app');
      const found = appId ? await identify(appId) : null;
      if (config.value.steamDetection) detected.value = found;
    } catch {
      detected.value = null;
    }
  }

  // Relancée à chaque configuration enregistrée : un jeu ajouté ou ignoré est aussitôt pris en compte.
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

  // Le jeu reconnu à son nom reçoit l'AppId détecté, pour ne plus dépendre de son nom.
  async function selectDetected(game) {
    gameId.value = game.id;
    if (game.steamAppId !== undefined) return;
    const { appId } = detected.value;
    await saveChange((cfg) => {
      cfg.games.find((g) => g.id === game.id).steamAppId = appId;
    });
  }

  // Ouvre la création d'un jeu, préremplie avec le nom et l'AppId du jeu Steam.
  function addGame({ appId, name }) {
    tab.value = 'config';
    cfgTab.value = 'jeux';
    newGame.value = { name, source: '', steamAppId: appId };
  }

  function ignoreDetected() {
    const { appId, name } = detected.value;
    return saveChange((cfg) => {
      cfg.ignoredGames.push({ appId, name });
    });
  }

  // Retiré de la liste des jeux ignorés, il sera de nouveau proposé.
  function unignore(appId) {
    return saveChange((cfg) => {
      cfg.ignoredGames = cfg.ignoredGames.filter((g) => g.appId !== appId);
    });
  }

  return { detected, storeBlocked, selectDetected, addGame, ignoreDetected, unignore };
});

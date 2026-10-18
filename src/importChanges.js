import { newId, sameName } from './configDraft.js';
import { sharedSettings, withOptional } from './share.js';

// The imported game replaces the one with the same name (keeping its id and, unless it brings one,
// its Steam AppId), otherwise it is added.
export function importGame(games, { name, steamAppId, events }) {
  const existing = games.find((g) => sameName(g.name, name));
  const game = withOptional({ id: existing?.id ?? newId(name, games), name, events }, 'steamAppId', steamAppId ?? existing?.steamAppId);
  return existing ? games.map((g) => (g === existing ? game : g)) : [...games, game];
}

// Imported exercises update those with the same name, which stay enabled or not, and
// add the others, enabled.
export function mergeExercises(exercises, imported) {
  const merged = exercises.map((ex) => ({ ...ex }));
  let added = 0;
  let updated = 0;
  for (const { name, ...settings } of imported) {
    const i = merged.findIndex((e) => sameName(e.name, name));
    const ex = merged[i];
    const shared = (item) => ({ ...item, ...sharedSettings(settings) });
    if (!ex) {
      merged.push(shared({ id: newId(name, merged), name: name.trim(), enabled: true }));
      added++;
    } else if (JSON.stringify(sharedSettings(ex)) !== JSON.stringify(sharedSettings(settings))) {
      merged[i] = shared({ id: ex.id, name: ex.name, enabled: ex.enabled });
      updated++;
    }
  }
  return { exercises: merged, added, updated };
}

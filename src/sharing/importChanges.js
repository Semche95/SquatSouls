import { nameKey, newId, sameName } from '../config/configDraft.js';
import { sharedSettings, withOptional } from './share.js';

// Changes an import brings to the configuration, listed for the user to choose from, then applied as chosen.
// A change is `{ key, status, name, before, after }`: `key` is the name key, `status` 'new' (only `after`),
// 'changed' (both) or 'absent' (an event of the replaced game missing from the import, only `before`).
// Unchanged items are not listed. The chosen changes are a Set of keys.

const eventSettings = ({ draws, maxPerSession, jokers = 0 }) => withOptional({ draws, jokers }, 'maxPerSession', maxPerSession);
export const differ = (a, b) => JSON.stringify(a) !== JSON.stringify(b);

function changes(current, imported, settings) {
  const byKey = new Map(current.map((item) => [nameKey(item.name), item]));
  const list = [];
  for (const after of imported) {
    const key = nameKey(after.name);
    const before = byKey.get(key);
    if (!before) list.push({ key, status: 'new', name: after.name.trim(), after });
    else if (differ(settings(before), settings(after))) list.push({ key, status: 'changed', name: before.name, before, after });
    byKey.delete(key);
  }
  return { list, absent: [...byKey].map(([key, before]) => ({ key, status: 'absent', name: before.name, before })) };
}

export const exerciseChanges = (exercises, imported) => changes(exercises, imported, sharedSettings).list;

// Changes to the events of `game`, the game the import replaces (none when it adds a game).
export function eventChanges(game, imported) {
  const { list, absent } = changes(game?.events ?? [], imported.events, eventSettings);
  return [...list, ...absent];
}

// Chosen by default: what the import adds or changes, not keeping the events it lacks.
export const defaultChoice = (list) => new Set(list.filter((c) => c.status !== 'absent').map((c) => c.key));

// Changes applied: the chosen additions and changes, and the absent events not kept.
export const appliedCount = (list, chosen) => list.filter((c) => chosen.has(c.key) !== (c.status === 'absent')).length;

// The imported game replaces the game whose id is `target` (keeping its id, its name and, unless it
// brings one, its Steam AppId), or is added without target. Its events follow the import's order: a chosen one is
// added or replaces the event with the same name (keeping its id and name), otherwise that event stays;
// the replaced game's events absent from the import stay at the end if chosen.
export function importGame(games, { name, steamAppId, events }, chosen, target) {
  const existing = games.find((g) => g.id === target);
  const old = existing?.events ?? [];
  const byKey = new Map(old.map((ev) => [nameKey(ev.name), ev]));
  const kept = [];
  for (const ev of events) {
    const key = nameKey(ev.name);
    const before = byKey.get(key);
    byKey.delete(key);
    if (!chosen.has(key)) {
      if (before) kept.push(before);
    } else kept.push({ ...ev, id: before?.id ?? newId(ev.name, [...old, ...kept]), name: before?.name ?? ev.name });
  }
  for (const [key, ev] of byKey) if (chosen.has(key)) kept.push(ev);
  const game = withOptional({ id: existing?.id ?? newId(name, games), name: existing?.name ?? name, events: kept }, 'steamAppId', steamAppId ?? existing?.steamAppId);
  return existing ? games.map((g) => (g === existing ? game : g)) : [...games, game];
}

// The chosen imported exercises update those with the same name, which stay enabled or not, or
// are added, enabled.
export function mergeExercises(exercises, imported, chosen) {
  const merged = exercises.map((ex) => ({ ...ex }));
  let added = 0;
  let updated = 0;
  for (const { name, ...settings } of imported) {
    if (!chosen.has(nameKey(name))) continue;
    const i = merged.findIndex((e) => sameName(e.name, name));
    const ex = merged[i];
    const shared = (item) => ({ ...item, ...sharedSettings(settings) });
    if (!ex) {
      merged.push(shared({ id: newId(name, merged), name: name.trim(), enabled: true }));
      added++;
    } else if (differ(sharedSettings(ex), sharedSettings(settings))) {
      merged[i] = shared({ id: ex.id, name: ex.name, enabled: ex.enabled });
      updated++;
    }
  }
  return { exercises: merged, added, updated };
}

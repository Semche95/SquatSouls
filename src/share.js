import { deflateSync, inflateSync, strFromU8, strToU8 } from 'fflate';
import { isCount, newId, sameName } from './configDraft.js';
import { t } from './i18n.js';

// Text shared between users: a game (name and events) or the exercise list, without ids,
// as compressed JSON (deflate) then base64-encoded. The format version and content type are in the
// compressed JSON; the short identifier placed in front (`squatsouls/game/sekiro:`, `squatsouls/exercises:`)
// is only informative and the import ignores it.
const MARK = 'squatsouls';
const VERSION = 2;
const GAME = 'game';
const EXERCISES = 'exercises';

// Reads the content according to the format version that wrote it. Changing the format means moving to a
// new version and adding a reader for it, keeping those of the previous ones to import their texts.
// Version 2: a game can carry the Steam AppId used to detect it.
const READERS = {
  1: (data) => data,
  2: (data) => data,
};

const isText = (v) => typeof v === 'string' && v.trim() !== '';

// Base64 byte by byte: spreading a large array as arguments of String.fromCharCode would exceed the
// argument limit of a function call.
export const toBase64 = (bytes) => btoa(Array.from(bytes, (b) => String.fromCharCode(b)).join(''));
const fromBase64 = (text) => Uint8Array.from(atob(text), (c) => c.charCodeAt(0));

function share(kind, data, id) {
  const payload = JSON.stringify({ version: VERSION, kind, ...data });
  return `${MARK}/${id ? `${kind}/${id}` : kind}:` + toBase64(deflateSync(strToU8(payload)));
}

// Version, type and content of a shared text (content absent if the version is unknown); throws an
// error if the content is unreadable. The identifier, ignored, precedes the last `:`, absent from base64.
export function unpack(text) {
  const payload = text.slice(text.lastIndexOf(':') + 1);
  const { version, kind, ...data } = JSON.parse(strFromU8(inflateSync(fromBase64(payload))));
  return { version, kind, data: READERS[version]?.(data) };
}

// Keys of the import messages by type: invalid text, or text of this type pasted into the other type's import.
const ERRORS = {
  [GAME]: { invalid: 'share.gameInvalid', misplaced: 'share.gameMisplaced' },
  [EXERCISES]: { invalid: 'share.exercisesInvalid', misplaced: 'share.exercisesMisplaced' },
};

// Returns the content of the pasted text if it has the expected type and `valid` accepts it, otherwise throws an error.
function parse(text, kind, valid) {
  let read = {};
  try {
    read = unpack(text);
  } catch {
    // Unreadable content: refused below.
  }
  if (read.kind !== kind && Object.hasOwn(ERRORS, read.kind)) throw new Error(t(ERRORS[read.kind].misplaced));
  if (read.kind === kind && !Object.hasOwn(READERS, read.version)) throw new Error(t('share.newer'));
  if (read.kind !== kind || !read.data || !valid(read.data)) throw new Error(t(ERRORS[kind].invalid));
  return read.data;
}

export function shareGame(game) {
  const events = game.events.map(({ name, draws, maxPerSession, jokers }) => ({ name, draws, maxPerSession, jokers }));
  return share(GAME, { name: game.name, steamAppId: game.steamAppId, events }, newId(game.name, []));
}

// Missing optional field (exercise-specific pause, game Steam AppId): the key is omitted.
const withOptional = (item, key, value) => (value === undefined ? item : { ...item, [key]: value });

// The global pause and the checked exercises are not shared: everyone keeps their own.
export function shareExercises(exercises) {
  return share(EXERCISES, { exercises: exercises.map(({ name, unit, min, max, pause }) => withOptional({ name, unit, min, max }, 'pause', pause)) });
}

const validEvent = (ev) =>
  isText(ev?.name) && isCount(ev.draws, 1) && (ev.maxPerSession === undefined || isCount(ev.maxPerSession, 1)) && (ev.jokers === undefined || isCount(ev.jokers, 0));

// Shared game ready to import; `draws` is capped at the number of checked exercises (`clamped` reports it).
export function parseGame(text, exerciseCount) {
  const data = parse(
    text,
    GAME,
    (d) =>
      isText(d.name) &&
      (d.steamAppId === undefined || isCount(d.steamAppId, 1)) &&
      Array.isArray(d.events) &&
      d.events.length > 0 &&
      d.events.every(validEvent),
  );
  const events = [];
  for (const { name, draws, maxPerSession, jokers = 0 } of data.events) {
    const ev = { id: newId(name, events), name: name.trim(), draws: Math.min(draws, exerciseCount), jokers };
    if (maxPerSession !== undefined) ev.maxPerSession = maxPerSession;
    events.push(ev);
  }
  const game = { name: data.name.trim(), events, clamped: data.events.some((ev) => ev.draws > exerciseCount) };
  return withOptional(game, 'steamAppId', data.steamAppId);
}

const validExercise = (ex) =>
  isText(ex?.name) && typeof ex.unit === 'string' && isCount(ex.min, 0) && isCount(ex.max, 1) && ex.min <= ex.max && (ex.pause === undefined || isCount(ex.pause, 0));

export function parseExercises(text) {
  return parse(
    text,
    EXERCISES,
    (d) => Array.isArray(d.exercises) && d.exercises.length > 0 && d.exercises.every(validExercise),
  ).exercises;
}

// The imported game replaces the one with the same name (keeping its id and, unless it brings one,
// its Steam AppId), otherwise it is added.
export function importGame(games, { name, steamAppId, events }) {
  const existing = games.find((g) => sameName(g.name, name));
  const game = withOptional({ id: existing?.id ?? newId(name, games), name, events }, 'steamAppId', steamAppId ?? existing?.steamAppId);
  return existing ? games.map((g) => (g === existing ? game : g)) : [...games, game];
}

// Imported exercises update those with the same name, which stay checked or not, and
// add the others, checked.
export function mergeExercises(exercises, imported) {
  const merged = exercises.map((ex) => ({ ...ex }));
  let added = 0;
  let updated = 0;
  for (const { name, unit, min, max, pause } of imported) {
    const i = merged.findIndex((e) => sameName(e.name, name));
    const ex = merged[i];
    if (!ex) {
      merged.push(withOptional({ id: newId(name, merged), name: name.trim(), enabled: true, unit, min, max }, 'pause', pause));
      added++;
    } else if (ex.unit !== unit || ex.min !== min || ex.max !== max || ex.pause !== pause) {
      merged[i] = withOptional({ id: ex.id, name: ex.name, enabled: ex.enabled, unit, min, max }, 'pause', pause);
      updated++;
    }
  }
  return { exercises: merged, added, updated };
}

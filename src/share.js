import { deflateSync, inflateSync, strFromU8, strToU8 } from 'fflate';
import { isCleanText, isCount, MAX_ITEMS, MAX_NAME, MAX_UNIT, newId, sameName } from './configDraft.js';
import { t } from './i18n.js';

// Text shared between users: a game (name and events) or the exercise list, without ids,
// as compressed JSON (deflate) then base64-encoded. The format version and content type are in the
// compressed JSON; the short identifier placed in front (`squatsouls/game/sekiro:`, `squatsouls/exercises:`)
// is only informative and the import ignores it.
const MARK = 'squatsouls';
const VERSION = 5;
const GAME = 'game';
const EXERCISES = 'exercises';

// Reads the content according to the format version that wrote it. Changing the format means moving to a
// new version and adding a reader for it, keeping those of the previous ones to import their texts.
// Version 2: a game can carry the Steam AppId used to detect it.
// Version 3: an exercise can be timed; before, it is timed if its unit reads as a duration.
// Version 4: an exercise's own recovery is named `recovery`; before, `pause`.
// Version 5: a timed exercise can be done on each side (`eachSide`); before, never.
const READERS = {
  1: (data) => withRecoveries(withUnitTimes(data)),
  2: (data) => withRecoveries(withUnitTimes(data)),
  3: (data) => withRecoveries(data),
  4: (data) => data,
  5: (data) => data,
};

// Time units of a timed exercise, and the units read as one in texts shared before exercises could be timed
// (the same as the database migration that made the existing exercises timed).
const TIME_UNITS = ['seconds', 'minutes'];
const DURATION_UNITS = {
  seconds: ['s', 'sec', 'sec.', 'secs', 'second', 'seconds', 'seconde', 'secondes'],
  minutes: ['min', 'min.', 'mins', 'minute', 'minutes'],
};

const unitTime = (unit) => (typeof unit === 'string' ? TIME_UNITS.find((u) => DURATION_UNITS[u].includes(unit.trim().toLowerCase())) : undefined);

function withUnitTimes(data) {
  if (!Array.isArray(data.exercises)) return data;
  return { ...data, exercises: data.exercises.map((ex) => withOptional(ex, 'timed', unitTime(ex?.unit))) };
}

function withRecoveries(data) {
  if (!Array.isArray(data.exercises)) return data;
  return { ...data, exercises: data.exercises.map(({ pause, ...ex }) => withOptional(ex, 'recovery', pause)) };
}

const isName = (v) => typeof v === 'string' && v.trim() !== '' && isCleanText(v, MAX_NAME);
const isList = (v, valid) => Array.isArray(v) && v.length > 0 && v.length <= MAX_ITEMS && v.every(valid);

// Two items with the same name in a pasted text: only the first is kept.
const firstByName = (items) => items.filter((it, i) => items.findIndex((other) => sameName(other.name, it.name)) === i);

// Largest pasted text and decompressed content, in characters and bytes, far above a legitimate share:
// a pasted text is never trusted, and a few bytes can decompress to gigabytes.
const MAX_SIZE = 256 * 1024;

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
  if (text.length > MAX_SIZE) throw new Error('shared text too large');
  const payload = text.slice(text.lastIndexOf(':') + 1);
  // Past its output buffer, inflate stops without an error: one byte beyond the limit reveals it.
  const bytes = inflateSync(fromBase64(payload), { out: new Uint8Array(MAX_SIZE + 1) });
  if (bytes.length > MAX_SIZE) throw new Error('shared content too large');
  const { version, kind, ...data } = JSON.parse(strFromU8(bytes));
  return { version, kind, data: Object.hasOwn(READERS, version) ? READERS[version](data) : undefined };
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

// Missing optional field (exercise-specific recovery or timer, game Steam AppId): the key is omitted.
const withOptional = (item, key, value) => (value === undefined ? item : { ...item, [key]: value });

// Settings of an exercise carried by a shared text, its optional ones omitted when missing; on each side only when true.
const sharedSettings = ({ unit, min, max, recovery, timed, eachSide }) =>
  withOptional(withOptional(withOptional({ unit, min, max }, 'recovery', recovery), 'timed', timed), 'eachSide', eachSide || undefined);

// The global recovery and whether exercises are enabled are not shared: everyone keeps their own.
export function shareExercises(exercises) {
  return share(EXERCISES, { exercises: exercises.map((ex) => ({ name: ex.name, ...sharedSettings(ex) })) });
}

const validEvent = (ev) =>
  isName(ev?.name) && isCount(ev.draws, 1) && (ev.maxPerSession === undefined || isCount(ev.maxPerSession, 1)) && (ev.jokers === undefined || isCount(ev.jokers, 0));

// Shared game ready to import; `draws` is capped at the number of enabled exercises (`clamped` reports it).
export function parseGame(text, exerciseCount) {
  const data = parse(text, GAME, (d) => isName(d.name) && (d.steamAppId === undefined || isCount(d.steamAppId, 1)) && isList(d.events, validEvent));
  const shared = firstByName(data.events);
  const events = [];
  for (const { name, draws, maxPerSession, jokers = 0 } of shared) {
    const ev = { id: newId(name, events), name: name.trim(), draws: Math.min(draws, exerciseCount), jokers };
    if (maxPerSession !== undefined) ev.maxPerSession = maxPerSession;
    events.push(ev);
  }
  const game = { name: data.name.trim(), events, clamped: shared.some((ev) => ev.draws > exerciseCount) };
  return withOptional(game, 'steamAppId', data.steamAppId);
}

const validExercise = (ex) =>
  isName(ex?.name) && isCleanText(ex.unit, MAX_UNIT) && isCount(ex.min, 0) && isCount(ex.max, 1) && ex.min <= ex.max && (ex.recovery === undefined || isCount(ex.recovery, 0)) && (ex.timed === undefined || TIME_UNITS.includes(ex.timed)) && (ex.eachSide === undefined || (ex.eachSide === true && ex.timed !== undefined));

export function parseExercises(text) {
  return firstByName(parse(text, EXERCISES, (d) => isList(d.exercises, validExercise)).exercises);
}

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

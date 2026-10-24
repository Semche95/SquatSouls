import { deflateSync, inflateSync, strFromU8, strToU8 } from 'fflate';
import { BOUNDS, folded, isCleanText, isCount, MAX_ITEMS, MAX_NAME, MAX_UNIT, nameKey, newId, rangeBounds, sameName } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';

// Text shared between users: a game (name and events) or the exercise list, without ids,
// as compressed JSON (deflate) then base64-encoded. The format version and content type are in the
// compressed JSON; the short identifier placed in front (`squatsouls/game/sekiro:`, `squatsouls/exercises:`)
// is only informative and the import ignores it.
const MARK = 'squatsouls';
const VERSION = 6;
const GAME = 'game';
const EXERCISES = 'exercises';

// Reads the content according to the format version that wrote it. Changing the format means moving to a
// new version and adding a reader for it, keeping those of the previous ones to import their texts.
// Version 2: a game can carry the Steam AppId used to detect it.
// Version 3: an exercise can be timed; before, it is timed if its unit reads as a duration.
// Version 4: an exercise's own recovery is named `recovery`; before, `pause`.
// Version 5: a timed exercise can be done on each side (`eachSide`); before, never.
// Version 6: the numbers follow the configuration bounds; before, they are brought within them.
const READERS = {
  1: (data) => withinBounds(withRecoveries(withUnitTimes(data))),
  2: (data) => withinBounds(withRecoveries(withUnitTimes(data))),
  3: (data) => withinBounds(withRecoveries(data)),
  4: (data) => withinBounds(data),
  5: (data) => withinBounds(data),
  6: (data) => data,
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

// A number the versions before the bounds accepted, from `floor`, brought within `min`..`max`; any other value
// is left for the validation to refuse.
const within = (v, floor, { min, max }) => (Number.isInteger(v) && v >= floor ? Math.min(Math.max(v, min), max) : v);

// Numbers of a text shared before the bounds, brought within them as the configurations were on the update
// that brought them: the requested exercises stay capped at import, which reports it.
function withinBounds(data) {
  const exercise = (ex) => {
    const min = within(ex?.min, 0, BOUNDS.amount);
    const bounded = { ...ex, min, max: within(ex?.max, Math.max(ex?.min, 1), { ...BOUNDS.amount, min }) };
    return withOptional(bounded, 'recovery', within(ex?.recovery, 0, BOUNDS.seconds));
  };
  const event = (ev) => ({ ...ev, maxPerSession: within(ev?.maxPerSession, 1, BOUNDS.amount), jokers: within(ev?.jokers, 0, BOUNDS.jokers) });
  if (Array.isArray(data.exercises)) return { ...data, exercises: data.exercises.map(exercise) };
  if (Array.isArray(data.events)) return { ...data, events: data.events.map(event) };
  return data;
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

// Name of what is shared as it appears in the label: lowercase, without accents, words joined by hyphens.
const labelOf = (name) => folded(name).replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

// `label`, only informative, follows the mark: the type, or what is shared and its name.
function share(kind, data, label = kind) {
  const payload = JSON.stringify({ version: VERSION, kind, ...data });
  return `${MARK}/${label}:${toBase64(deflateSync(strToU8(payload)))}`;
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
  return share(GAME, { name: game.name, steamAppId: game.steamAppId, events }, `game/${labelOf(game.name)}`);
}

// Missing optional field (exercise-specific recovery or timer, game Steam AppId): the key is omitted.
export const withOptional = (item, key, value) => (value === undefined ? item : { ...item, [key]: value });

// Settings of an exercise carried by a shared text, its optional ones omitted when missing; on each side only when true.
export const sharedSettings = ({ unit, min, max, recovery, timed, eachSide }) =>
  withOptional(withOptional(withOptional({ unit, min, max }, 'recovery', recovery), 'timed', timed), 'eachSide', eachSide || undefined);

// The global recovery and whether exercises are enabled are not shared: everyone keeps their own.
export function shareExercises(exercises, label) {
  return share(EXERCISES, { exercises: exercises.map((ex) => ({ name: ex.name, ...sharedSettings(ex) })) }, label);
}

// A single exercise is a list of one, named in the label.
export const shareExercise = (ex) => shareExercises([ex], `exercise/${labelOf(ex.name)}`);

const validEvent = (ev) =>
  isName(ev?.name) && isCount(ev.draws, BOUNDS.positive) && (ev.maxPerSession === undefined || isCount(ev.maxPerSession, BOUNDS.amount)) && (ev.jokers === undefined || isCount(ev.jokers, BOUNDS.jokers));

// Shared game ready to import; `draws` is capped at the largest number of requested exercises (`clamped` lists
// the name keys of the capped events).
export function parseGame(text) {
  const data = parse(text, GAME, (d) => isName(d.name) && (d.steamAppId === undefined || isCount(d.steamAppId, BOUNDS.positive)) && isList(d.events, validEvent));
  const shared = firstByName(data.events);
  const events = [];
  for (const { name, draws, maxPerSession, jokers = 0 } of shared) {
    const ev = { id: newId(), name: name.trim(), draws: Math.min(draws, BOUNDS.draws.max), jokers };
    if (maxPerSession !== undefined) ev.maxPerSession = maxPerSession;
    events.push(ev);
  }
  const game = { name: data.name.trim(), events, clamped: shared.filter((ev) => ev.draws > BOUNDS.draws.max).map((ev) => nameKey(ev.name)) };
  return withOptional(game, 'steamAppId', data.steamAppId);
}

const validExercise = (ex) =>
  isName(ex?.name) && isCleanText(ex.unit, MAX_UNIT) && isCount(ex.min, BOUNDS.amount) && isCount(ex.max, rangeBounds(ex).max) && (ex.recovery === undefined || isCount(ex.recovery, BOUNDS.seconds)) && (ex.timed === undefined || TIME_UNITS.includes(ex.timed)) && (ex.eachSide === undefined || (ex.eachSide === true && ex.timed !== undefined));

export function parseExercises(text) {
  return firstByName(parse(text, EXERCISES, (d) => isList(d.exercises, validExercise)).exercises);
}

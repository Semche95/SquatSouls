import { language, t } from '../i18n/i18n.js';

export function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

// Configuration fields edited in the Settings tab; the Program tab edits all the others.
export const SETTINGS_FIELDS = ['rememberIntensity', 'rememberLimit', 'sound', 'timerAutoDone', 'timerPrep', 'tray', 'closeToTray', 'startHidden', 'steamDetection', 'steamNotify', 'updateCheck', 'updateAuto'];

// Saved configuration with the fields of one tab taken from the draft: each tab saves and resets only its own.
export function withTabDraft(config, draft, settings) {
  const cfg = clone(settings ? config : draft);
  for (const f of SETTINGS_FIELDS) cfg[f] = (settings ? draft : config)[f];
  return cfg;
}

// Ids stay internal: derived from the name on creation, then fixed even when renamed.
export function newId(name, items) {
  const base = name.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'id';
  const taken = new Set(items.map((it) => it.id));
  let id = base;
  for (let n = 2; taken.has(id); n++) id = `${base}-${n}`;
  return id;
}

// Name compared to others, without case nor surrounding spaces.
export const nameKey = (name) => name.trim().toLowerCase();

// Two names identical up to surrounding spaces and case refer to the same game.
export function sameName(a, b) {
  return nameKey(a) === nameKey(b);
}

// A name that starts with a game's name, without extending it with a letter ("Megabonk 2", "Elden Ring no hit"),
// refers to a variant of that game, up to case and surrounding spaces.
export function extendsName(name, base) {
  const [n, b] = [name, base].map((s) => s.trim().toLowerCase());
  return n.startsWith(b) && !/\p{L}/u.test(n[b.length] ?? '');
}

// Games are displayed by name, whatever the saved order.
export function byName(games) {
  return [...games].sort((a, b) => a.name.localeCompare(b.name, language.value, { sensitivity: 'base' }));
}

// Limits checked by the backend: longest name and unit, in characters, and most exercises or events in a game.
export const MAX_NAME = 300;
export const MAX_UNIT = 50;
export const MAX_ITEMS = 100;

// Bounds of each number field, checked by the backend: far above any training, they keep the numbers
// shown within their frames. The requested exercises never exceed the exercises; `positive` is any u32
// from 1, for a Steam AppId or the requested exercises of a shared game, capped at import.
const MAX_AMOUNT = 9999;
export const BOUNDS = {
  seconds: { min: 0, max: 3600 },
  repDuration: { min: 1, max: 3600 },
  amount: { min: 1, max: MAX_AMOUNT },
  jokers: { min: 0, max: MAX_AMOUNT },
  draws: { min: 1, max: MAX_ITEMS },
  positive: { min: 1, max: 4294967295 },
};

export const isCount = (v, { min, max }) => Number.isInteger(v) && v >= min && v <= max;

// Bounds of an exercise's minimum and maximum: each one also stops at the other, when it is valid.
export function rangeBounds(ex) {
  const { amount } = BOUNDS;
  return {
    min: { ...amount, max: isCount(ex.max, amount) ? ex.max : amount.max },
    max: { ...amount, min: isCount(ex.min, amount) ? ex.min : amount.min },
  };
}

// Control or invisible character (zero width, text direction override), refused by the backend.
// The zero width joiner stays allowed, emoji sequences use it.
const HIDDEN_CHAR = /[\p{Cc}\u00AD\u200B\u200C\u200E\u200F\u202A-\u202E\u2060-\u2064\u2066-\u2069\uFEFF]/u;

// Text within `max` characters (counted like the backend), without a hidden character.
export const isCleanText = (v, max) => typeof v === 'string' && [...v].length <= max && !HIDDEN_CHAR.test(v);

// Text typed in a number field: a number if it only has digits, otherwise the text as is
// (empty, it is ''; anything else, countError reports it).
export function parseCount(text) {
  const trimmed = text.trim();
  return /^\d+$/.test(trimmed) ? Number(trimmed) : trimmed;
}

// A text, a decimal number or a number out of `bounds`, reported while typing.
export function countError(value, bounds) {
  if (value === undefined || value === '' || isCount(value, bounds)) return '';
  if (!Number.isInteger(value)) return t('fields.notCount', bounds);
  return value < bounds.min ? t('fields.tooSmall', bounds) : t('fields.tooLarge', bounds);
}

// First badly filled field of the configuration, '' if there is none: an empty event or exercise name,
// then an invalid number, out of its bounds included.
export function fieldError(cfg) {
  // Field path: names from the data, then the translated field name.
  const at = (...path) => path.slice(0, -1).concat(t(`fields.${path.at(-1)}`)).join(' › ');
  const names = cfg.exercises.map((ex) => [ex.name, at('exerciseName')]);
  for (const g of cfg.games) names.push(...g.events.map((ev) => [ev.name, at(g.name, 'eventName')]));
  const unnamed = names.find(([name]) => !name.trim());
  if (unnamed) return t('fields.error', { field: unnamed[1], error: t('fields.empty') });
  const { seconds, repDuration, amount, jokers, draws } = BOUNDS;
  const fields = [[cfg.recovery, seconds, at('globalRecovery')], [cfg.repDuration, repDuration, at('repDuration')], [cfg.timerPrep, seconds, at('timerPrep')]];
  for (const ex of cfg.exercises) {
    const range = rangeBounds(ex);
    fields.push([ex.min, range.min, at(ex.name, 'minimum')], [ex.max, range.max, at(ex.name, 'maximum')], [ex.recovery, seconds, at(ex.name, 'recovery')]);
  }
  for (const g of cfg.games) {
    for (const ev of g.events) {
      fields.push([ev.draws, draws, at(g.name, ev.name, 'draws')], [ev.maxPerSession, amount, at(g.name, ev.name, 'limit')], [ev.jokers, jokers, at(g.name, ev.name, 'jokers')]);
    }
  }
  for (const [value, bounds, field] of fields) {
    const error = countError(value, bounds);
    if (error) return t('fields.error', { field, error });
  }
  return '';
}

// A cleared required field is "": bring it back to 0 so the backend refuses it cleanly.
// An invalid text is kept so fieldError reports it.
const orZero = (v) => (v === '' || v === undefined ? 0 : v);

export function normalized(cfg) {
  const c = clone(cfg);
  c.recovery = orZero(c.recovery);
  c.repDuration = orZero(c.repDuration);
  c.timerPrep = orZero(c.timerPrep);
  for (const ex of c.exercises) {
    ex.min = orZero(ex.min);
    ex.max = orZero(ex.max);
  }
  for (const g of c.games) {
    for (const ev of g.events) {
      ev.draws = orZero(ev.draws);
      ev.jokers = orZero(ev.jokers);
    }
  }
  return c;
}

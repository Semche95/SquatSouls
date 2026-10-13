import { language, t } from './i18n.js';

export function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

// Configuration fields edited in the Settings tab; the Program tab edits all the others.
export const SETTINGS_FIELDS = ['rememberIntensity', 'tray', 'closeToTray', 'startHidden', 'steamDetection', 'steamNotify'];

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

// Two names identical up to surrounding spaces and case refer to the same game.
export function sameName(a, b) {
  return a.trim().toLowerCase() === b.trim().toLowerCase();
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

// Largest number accepted by the backend (u32).
const MAX_COUNT = 4294967295;

export const isCount = (v, min) => Number.isInteger(v) && v >= min && v <= MAX_COUNT;

// Limits checked by the backend: longest name and unit, in characters, and most exercises or events in a game.
export const MAX_NAME = 300;
export const MAX_UNIT = 50;
export const MAX_ITEMS = 100;

// Control or invisible character (zero width, text direction override), refused by the backend.
// The zero width joiner stays allowed, emoji sequences use it.
const HIDDEN_CHAR = /[\p{Cc}­​‌‎‏‪-‮⁠-⁤⁦-⁩﻿]/u;

// Text within `max` characters (counted like the backend), without a hidden character.
export const isCleanText = (v, max) => typeof v === 'string' && [...v].length <= max && !HIDDEN_CHAR.test(v);

// Text typed in a number field: a number if it only has digits, otherwise the text as is
// (empty, it is ''; anything else, countError reports it).
export function parseCount(text) {
  const t = text.trim();
  return /^\d+$/.test(t) ? Number(t) : t;
}

// The backend refuses a negative, decimal or too large number without explanation: report it while typing.
export function countError(value) {
  if (value === undefined || value === '' || isCount(value, 0)) return '';
  return t(Number.isInteger(value) && value > MAX_COUNT ? 'fields.tooLarge' : 'fields.notCount');
}

// First invalid number field of the configuration, '' if there is none.
export function numberError(cfg) {
  // Field path: names from the data, then the translated field name.
  const at = (...path) => path.slice(0, -1).concat(t(`fields.${path.at(-1)}`)).join(' › ');
  const fields = [[cfg.pause, at('globalPause')]];
  for (const ex of cfg.exercises) {
    fields.push([ex.min, at(ex.name, 'minimum')], [ex.max, at(ex.name, 'maximum')], [ex.pause, at(ex.name, 'pause')]);
  }
  for (const g of cfg.games) {
    for (const ev of g.events) {
      fields.push([ev.draws, at(g.name, ev.name, 'draws')], [ev.maxPerSession, at(g.name, ev.name, 'limit')], [ev.jokers, at(g.name, ev.name, 'jokers')]);
    }
  }
  for (const [value, field] of fields) {
    const error = countError(value);
    if (error) return t('fields.error', { field, error });
  }
  return '';
}

// A cleared required field is "": bring it back to 0 so the backend refuses it cleanly.
// An invalid text is kept so numberError reports it.
const orZero = (v) => (v === '' || v === undefined ? 0 : v);

export function normalized(cfg) {
  const c = clone(cfg);
  c.pause = orZero(c.pause);
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

import { describe, expect, it } from 'vitest';
import { byName, countError, extendsName, newId, normalized, numberError, parseCount } from '../configDraft.js';

describe('newId', () => {
  it('derives the id from the name without accents and avoids duplicates', () => {
    expect(newId('Déjà Vu Event', [])).toBe('deja-vu-event');
    expect(newId('Death', [{ id: 'death' }, { id: 'death-2' }])).toBe('death-3');
    expect(newId('!!', [])).toBe('id');
  });
});

describe('byName', () => {
  it('sorts by name ignoring case and accents, without touching the original', () => {
    const games = [{ name: 'sekiro' }, { name: 'Élden Ring' }, { name: 'Bloodborne' }];
    expect(byName(games).map((g) => g.name)).toEqual(['Bloodborne', 'Élden Ring', 'sekiro']);
    expect(games[0].name).toBe('sekiro');
  });
});

describe('extendsName', () => {
  it('recognizes the same name or a name extending it without an adjoining letter', () => {
    expect(extendsName(' megabonk ', 'Megabonk')).toBe(true);
    expect(extendsName('megabonk 2', 'Megabonk')).toBe(true);
    expect(extendsName('Megabonk: no-hit', 'Megabonk')).toBe(true);
    expect(extendsName('Megabonkers', 'Megabonk')).toBe(false);
    expect(extendsName('Mega', 'Megabonk')).toBe(false);
  });
});

describe('normalized', () => {
  it('brings cleared number fields back to 0 without touching the original', () => {
    const cfg = { pause: '', exercises: [{ min: '', max: 5 }], games: [{ events: [{ draws: '', jokers: undefined }] }] };
    expect(normalized(cfg)).toEqual({ pause: 0, exercises: [{ min: 0, max: 5 }], games: [{ events: [{ draws: 0, jokers: 0 }] }] });
    expect(cfg.exercises[0].min).toBe('');
  });

  it('keeps an invalid text so it gets reported', () => {
    expect(normalized({ pause: 'abc', exercises: [], games: [] }).pause).toBe('abc');
  });
});

describe('parseCount and countError', () => {
  it('reads a typed integer and reports any other text', () => {
    expect(parseCount(' 12 ')).toBe(12);
    expect(parseCount('')).toBe('');
    expect(countError(parseCount(''))).toBe('');
    expect(countError(parseCount('12'))).toBe('');
    for (const text of ['abc', '-2', '1.5', '3e2']) expect(countError(parseCount(text))).toBe('il faut un nombre entier, 0 ou plus');
    expect(countError(parseCount('99999999999'))).toBe('nombre trop grand');
  });
});

describe('numberError', () => {
  const cfg = (patch = {}) => ({
    pause: 60,
    exercises: [{ name: 'Squats', min: 1, max: 5, ...patch.ex }],
    games: [{ name: 'Elden Ring', events: [{ name: 'Death', draws: 1, jokers: 0, ...patch.ev }] }],
  });

  it('accepts non-negative integers and missing optional fields', () => {
    expect(numberError(cfg())).toBe('');
  });

  it('names the first negative, decimal or too large field', () => {
    expect(numberError({ ...cfg(), pause: -1 })).toBe('Pause après chaque exercice : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ex: { min: 1.5 } }))).toBe('Squats › minimum : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ex: { pause: -3 } }))).toBe('Squats › pause : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ev: { maxPerSession: 2.5 } }))).toBe('Elden Ring › Death › limite par séance : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ev: { jokers: 1e10 } }))).toBe('Elden Ring › Death › jokers gagnés : nombre trop grand.');
    expect(numberError(cfg({ ev: { draws: 'deux' } }))).toBe('Elden Ring › Death › exercices demandés : il faut un nombre entier, 0 ou plus.');
  });
});

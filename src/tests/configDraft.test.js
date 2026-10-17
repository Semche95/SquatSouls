import { describe, expect, it } from 'vitest';
import { byName, countError, extendsName, newId, normalized, fieldError, parseCount } from '../configDraft.js';

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
    const cfg = { recovery: '', timerPrep: '', exercises: [{ min: '', max: 5 }], games: [{ events: [{ draws: '', jokers: undefined }] }] };
    expect(normalized(cfg)).toEqual({ recovery: 0, timerPrep: 0, exercises: [{ min: 0, max: 5 }], games: [{ events: [{ draws: 0, jokers: 0 }] }] });
    expect(cfg.exercises[0].min).toBe('');
  });

  it('keeps an invalid text so it gets reported', () => {
    expect(normalized({ recovery: 'abc', exercises: [], games: [] }).recovery).toBe('abc');
  });
});

describe('parseCount and countError', () => {
  it('reads a typed integer and reports any other text', () => {
    expect(parseCount(' 12 ')).toBe(12);
    expect(parseCount('')).toBe('');
    expect(countError(parseCount(''))).toBe('');
    expect(countError(parseCount('12'))).toBe('');
    for (const text of ['abc', '-2', '1.5', '3e2']) expect(countError(parseCount(text))).toBe('a whole number is needed, 0 or more');
    expect(countError(parseCount('99999999999'))).toBe('number too large');
  });
});

describe('fieldError', () => {
  const cfg = (patch = {}) => ({
    recovery: 60,
    exercises: [{ name: 'Squats', min: 1, max: 5, ...patch.ex }],
    games: [{ name: 'Elden Ring', events: [{ name: 'Death', draws: 1, jokers: 0, ...patch.ev }] }],
  });

  it('accepts non-negative integers and missing optional fields', () => {
    expect(fieldError(cfg())).toBe('');
  });

  it('names the first event or exercise left without a name', () => {
    expect(fieldError(cfg({ ex: { name: ' ' }, ev: { name: '' } }))).toBe('exercise name: it cannot be empty.');
    expect(fieldError(cfg({ ev: { name: '', jokers: -1 } }))).toBe('Elden Ring › event name: it cannot be empty.');
  });

  it('names the first negative, decimal or too large field', () => {
    expect(fieldError({ ...cfg(), recovery: -1 })).toBe('Recovery after each exercise: a whole number is needed, 0 or more.');
    expect(fieldError({ ...cfg(), timerPrep: 'abc' })).toBe('Countdown before the timer: a whole number is needed, 0 or more.');
    expect(fieldError(cfg({ ex: { min: 1.5 } }))).toBe('Squats › minimum: a whole number is needed, 0 or more.');
    expect(fieldError(cfg({ ex: { recovery: -3 } }))).toBe('Squats › recovery: a whole number is needed, 0 or more.');
    expect(fieldError(cfg({ ev: { maxPerSession: 2.5 } }))).toBe('Elden Ring › Death › limit per session: a whole number is needed, 0 or more.');
    expect(fieldError(cfg({ ev: { jokers: 1e10 } }))).toBe('Elden Ring › Death › jokers earned: number too large.');
    expect(fieldError(cfg({ ev: { draws: 'two' } }))).toBe('Elden Ring › Death › exercises required: a whole number is needed, 0 or more.');
  });
});

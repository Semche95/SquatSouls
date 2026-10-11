import { describe, expect, it } from 'vitest';
import { byName, countError, newId, normalized, numberError, parseCount } from '../configDraft.js';

describe('newId', () => {
  it('dérive l\'id du nom sans accents et évite les doublons', () => {
    expect(newId('Événement Spécial', [])).toBe('evenement-special');
    expect(newId('Mort', [{ id: 'mort' }, { id: 'mort-2' }])).toBe('mort-3');
    expect(newId('!!', [])).toBe('id');
  });
});

describe('byName', () => {
  it('trie par nom sans tenir compte de la casse ni des accents, sans toucher l\'original', () => {
    const games = [{ name: 'sekiro' }, { name: 'Élden Ring' }, { name: 'Bloodborne' }];
    expect(byName(games).map((g) => g.name)).toEqual(['Bloodborne', 'Élden Ring', 'sekiro']);
    expect(games[0].name).toBe('sekiro');
  });
});

describe('normalized', () => {
  it('ramène les champs numériques vidés à 0 sans toucher l\'original', () => {
    const cfg = { pause: '', exercises: [{ min: '', max: 5 }], games: [{ events: [{ draws: '', jokers: undefined }] }] };
    expect(normalized(cfg)).toEqual({ pause: 0, exercises: [{ min: 0, max: 5 }], games: [{ events: [{ draws: 0, jokers: 0 }] }] });
    expect(cfg.exercises[0].min).toBe('');
  });

  it('garde un texte invalide pour qu\'il soit signalé', () => {
    expect(normalized({ pause: 'abc', exercises: [], games: [] }).pause).toBe('abc');
  });
});

describe('parseCount et countError', () => {
  it('lit un nombre entier saisi et signale tout autre texte', () => {
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
    games: [{ name: 'Elden Ring', events: [{ name: 'Mort', draws: 1, jokers: 0, ...patch.ev }] }],
  });

  it('accepte des entiers positifs ou nuls et les champs facultatifs absents', () => {
    expect(numberError(cfg())).toBe('');
  });

  it('nomme le premier champ négatif, décimal ou trop grand', () => {
    expect(numberError({ ...cfg(), pause: -1 })).toBe('Pause après chaque série : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ex: { min: 1.5 } }))).toBe('Squats › minimum : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ex: { pause: -3 } }))).toBe('Squats › pause : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ev: { maxPerSession: 2.5 } }))).toBe('Elden Ring › Mort › limite par séance : il faut un nombre entier, 0 ou plus.');
    expect(numberError(cfg({ ev: { jokers: 1e10 } }))).toBe('Elden Ring › Mort › jokers gagnés : nombre trop grand.');
    expect(numberError(cfg({ ev: { draws: 'deux' } }))).toBe('Elden Ring › Mort › exercices tirés : il faut un nombre entier, 0 ou plus.');
  });
});

import { describe, expect, it } from 'vitest';
import { importGame, mergeExercises, parseExercises, parseGame, shareExercises, shareGame } from '../share.js';
import { config } from './app.js';

const exercises = [
  { id: 'squat', name: 'Squats', unit: 'rép.', min: 10, max: 15 },
  { id: 'pompes', name: 'Pompes', unit: 'rép.', min: 8, max: 12 },
];

describe('partage d\'un jeu', () => {
  it('partage le nom et les événements sans id ni exercices, puis les relit', () => {
    const text = shareGame(config.games[0]);
    expect(text).not.toContain('"id"');
    expect(parseGame(text, 3)).toEqual({
      name: 'Elden Ring',
      events: [
        { id: 'mort', name: 'Mort', draws: 1, jokers: 0, maxPerSession: 15 },
        { id: 'boss-vaincu', name: 'Boss vaincu', draws: 1, jokers: 2 },
      ],
      clamped: false,
    });
  });

  it('ramène le nombre tiré au nombre d\'exercices définis', () => {
    const text = shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 5, jokers: 0 }] });
    expect(parseGame(text, 2)).toMatchObject({ events: [{ draws: 2 }], clamped: true });
  });

  it('refuse un texte qui n\'est pas un jeu partagé valide', () => {
    const invalid = [
      'du texte',
      shareExercises(exercises),
      shareGame({ name: 'Nioh', events: [] }),
      shareGame({ name: ' ', events: [{ name: 'Mort', draws: 1 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 1, maxPerSession: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 1, jokers: -1 }] }),
    ];
    for (const text of invalid) expect(() => parseGame(text, 3)).toThrow('Ce texte n\'est pas un jeu partagé valide.');
  });

  it('remplace le jeu du même nom en gardant son id, sinon l\'ajoute', () => {
    const events = [{ id: 'feu', name: 'Feu', draws: 1, jokers: 0 }];
    expect(importGame(config.games, { name: ' sekiro ', events }).map((g) => [g.id, g.events])).toEqual([
      ['er', config.games[0].events],
      ['sekiro', events],
    ]);
    expect(importGame(config.games, { name: 'Nioh', events }).at(-1)).toEqual({ id: 'nioh', name: 'Nioh', events });
  });
});

describe('partage des exercices', () => {
  it('partage les exercices sans id, puis les relit', () => {
    const text = shareExercises(exercises);
    expect(text).not.toContain('"id"');
    expect(parseExercises(text)).toEqual(exercises.map(({ id, ...ex }) => ex));
  });

  it('refuse un texte qui n\'est pas une liste d\'exercices valide', () => {
    const invalid = [
      '',
      shareGame(config.games[0]),
      shareExercises([]),
      shareExercises([{ name: 'Squats', unit: 'rép.', min: 5, max: 0 }]),
      shareExercises([{ name: 'Squats', unit: 'rép.', min: 6, max: 5 }]),
      shareExercises([{ name: '', unit: 'rép.', min: 1, max: 5 }]),
    ];
    for (const text of invalid) expect(() => parseExercises(text)).toThrow('Ce texte n\'est pas une liste d\'exercices partagée valide.');
  });

  it('met à jour les exercices du même nom et ajoute les autres, sans toucher l\'original', () => {
    const imported = [
      { name: 'squats', unit: 's', min: 20, max: 30 },
      { name: 'Pompes', unit: 'rép.', min: 8, max: 12 },
      { name: 'Fentes', unit: 'rép.', min: 5, max: 8 },
    ];
    expect(mergeExercises(exercises, imported)).toEqual({
      exercises: [
        { id: 'squat', name: 'Squats', unit: 's', min: 20, max: 30 },
        exercises[1],
        { id: 'fentes', name: 'Fentes', unit: 'rép.', min: 5, max: 8 },
      ],
      added: 1,
      updated: 1,
    });
    expect(exercises[0].unit).toBe('rép.');
  });
});

import { deflateSync, strToU8 } from 'fflate';
import { describe, expect, it } from 'vitest';
import { MAX_ITEMS, MAX_NAME, MAX_UNIT } from '../configDraft.js';
import { importGame, mergeExercises, parseExercises, parseGame, shareExercises, shareGame, toBase64, unpack } from '../share.js';
import { config } from './app.js';

// Shared text written in a given format version, with the content of `shared`.
function packed(version, shared, data = unpack(shared).data) {
  const payload = JSON.stringify({ version, kind: unpack(shared).kind, ...data });
  return shared.slice(0, shared.indexOf(':') + 1) + toBase64(deflateSync(strToU8(payload)));
}

const exercises = [
  { id: 'squat', name: 'Squats', enabled: false, unit: 'reps', min: 10, max: 15 },
  { id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12, pause: 90 },
  { id: 'plank', name: 'Plank', enabled: true, unit: 'hold', min: 20, max: 45, timed: 'seconds' },
];

describe('game sharing', () => {
  it('shares the name and events without ids or exercises, then reads them back', () => {
    const text = shareGame(config.games[0]);
    expect(text).toMatch(/^squatsouls\/game\/elden-ring:/);
    expect(JSON.stringify(unpack(text).data)).not.toContain('"id"');
    expect(parseGame(text, 3)).toEqual({
      name: 'Elden Ring',
      events: [
        { id: 'death', name: 'Death', draws: 1, jokers: 0, maxPerSession: 15 },
        { id: 'boss-defeated', name: 'Boss defeated', draws: 1, jokers: 2 },
      ],
      clamped: false,
    });
  });

  it('ignores the identifier placed in front of the compressed text', () => {
    const text = shareGame(config.games[0]);
    const payload = text.slice(text.indexOf(':') + 1);
    expect(parseGame(`squatsouls/exercises:${payload}`, 3)).toEqual(parseGame(text, 3));
    expect(parseGame(payload, 3)).toEqual(parseGame(text, 3));
  });

  it('shares the game\'s Steam AppId and reads a version 1 text, which has none', () => {
    const game = { ...config.games[0], steamAppId: 1245620 };
    expect(parseGame(shareGame(game), 3).steamAppId).toBe(1245620);
    const { steamAppId, ...v1 } = unpack(shareGame(game)).data;
    const read = parseGame(packed(1, shareGame(game), v1), 3);
    expect(read).toEqual(parseGame(shareGame(config.games[0]), 3));
    expect(read).not.toHaveProperty('steamAppId');
  });

  it('caps the drawn number at the number of defined exercises', () => {
    const text = shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 5, jokers: 0 }] });
    expect(parseGame(text, 2)).toMatchObject({ events: [{ draws: 2 }], clamped: true });
  });

  it('refuses a text that is not a valid shared game', () => {
    const invalid = [
      'some text',
      shareGame({ name: 'Nioh', events: [] }),
      shareGame({ name: ' ', events: [{ name: 'Death', draws: 1 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1, maxPerSession: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1, jokers: -1 }] }),
      shareGame({ name: 'Nioh', steamAppId: 0, events: [{ name: 'Death', draws: 1 }] }),
      shareGame({ name: 'Nioh', steamAppId: '485510', events: [{ name: 'Death', draws: 1 }] }),
      shareGame({ name: 'N'.repeat(MAX_NAME + 1), events: [{ name: 'Death', draws: 1 }] }),
      ...['\n', '\t', '\u200B', '\u202E', '\uFEFF', '\u0085'].map((c) => shareGame({ name: 'Nioh', events: [{ name: `Death${c}`, draws: 1 }] })),
      shareGame({ name: 'Nioh', events: Array.from({ length: MAX_ITEMS + 1 }, () => ({ name: 'Death', draws: 1 })) }),
    ];
    for (const text of invalid) expect(() => parseGame(text, 3)).toThrow('Ce texte n\'est pas un jeu partagé valide.');
  });

  it('keeps only the first of two events with the same name', () => {
    const text = shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1 }, { name: ' death ', draws: 9 }, { name: 'Boss', draws: 1 }] });
    expect(parseGame(text, 3)).toMatchObject({ events: [{ name: 'Death', draws: 1 }, { name: 'Boss' }], clamped: false });
  });

  it('accepts the emoji sequences, joined by a zero width joiner', () => {
    expect(parseGame(shareGame({ name: 'Nioh 👨‍👩‍👧', events: [{ name: 'Death', draws: 1 }] }), 3).name).toBe('Nioh 👨‍👩‍👧');
  });

  it('refuses a pasted text or a content too large without decompressing it all', () => {
    const bomb = 'squatsouls/game:' + toBase64(deflateSync(new Uint8Array(100 * 1024 * 1024)));
    expect(() => parseGame(bomb, 3)).toThrow('Ce texte n\'est pas un jeu partagé valide.');
    expect(() => parseGame(shareGame(config.games[0]) + ' '.repeat(300 * 1024), 3)).toThrow('Ce texte n\'est pas un jeu partagé valide.');
  });

  it('tells where to import exercises pasted instead of a game', () => {
    expect(() => parseGame(shareExercises(exercises), 3)).toThrow('Ce texte partage des exercices : l\'importer depuis l\'onglet Exercices, avec Importer.');
  });

  it('asks for an update for a text from an unknown format version', () => {
    const text = packed(999, shareGame(config.games[0]));
    expect(() => parseGame(text, 3)).toThrow('Ce texte vient d\'une version plus récente de SquatSouls : la mettre à jour pour l\'importer.');
  });

  it('replaces the game with the same name keeping its id, otherwise adds it', () => {
    const events = [{ id: 'bonfire', name: 'Bonfire', draws: 1, jokers: 0 }];
    expect(importGame(config.games, { name: ' sekiro ', events }).map((g) => [g.id, g.events])).toEqual([
      ['er', config.games[0].events],
      ['sekiro', events],
    ]);
    expect(importGame(config.games, { name: 'Nioh', events }).at(-1)).toEqual({ id: 'nioh', name: 'Nioh', events });
  });

  it('takes the imported Steam AppId, or else the replaced game\'s, even if carried by another game', () => {
    const events = [{ id: 'bonfire', name: 'Bonfire', draws: 1, jokers: 0 }];
    const games = [{ ...config.games[0], steamAppId: 1245620 }, { ...config.games[1], steamAppId: 814380 }];
    expect(importGame(games, { name: 'Sekiro', events })[1].steamAppId).toBe(814380);
    expect(importGame(games, { name: 'Sekiro', steamAppId: 814381, events })[1].steamAppId).toBe(814381);
    expect(importGame(games, { name: 'Nioh', steamAppId: 485510, events }).at(-1).steamAppId).toBe(485510);
    expect(importGame(games, { name: 'Sekiro no prosthetic', steamAppId: 814380, events }).at(-1).steamAppId).toBe(814380);
  });
});

describe('exercise sharing', () => {
  it('shares the exercises without ids, with their own pause and timer, then reads them back', () => {
    const text = shareExercises(exercises);
    expect(text).toMatch(/^squatsouls\/exercises:/);
    expect(JSON.stringify(unpack(text).data)).not.toContain('"id"');
    expect(JSON.stringify(unpack(text).data)).toContain('"pause":90');
    expect(JSON.stringify(unpack(text).data)).toContain('"timed":"seconds"');
    expect(parseExercises(text)).toEqual(exercises.map(({ id, enabled, ...ex }) => ex));
    // Spaces and line breaks added by a messaging app are ignored.
    const cut = text.indexOf(':') + 10;
    expect(parseExercises(` ${text.slice(0, cut)}\n${text.slice(cut)} `)).toEqual(parseExercises(text));
  });

  it('encodes content too large to fit in the arguments of a function call', () => {
    const bytes = Uint8Array.from({ length: 500000 }, (_, i) => i % 256);
    expect(Uint8Array.from(atob(toBase64(bytes)), (c) => c.charCodeAt(0))).toEqual(bytes);
  });

  it('reads back the largest list allowed, with the longest names and units', () => {
    const longest = Array.from({ length: MAX_ITEMS }, (_, i) => ({ name: `${i} ${'😀'.repeat(MAX_NAME - 4)}`, unit: '😀'.repeat(MAX_UNIT), min: 1, max: 5 }));
    expect(parseExercises(shareExercises(longest))).toEqual(longest);
  });

  it('refuses a text that is not a valid exercise list', () => {
    const invalid = [
      '',
      shareExercises([]),
      shareExercises([{ name: 'Squats', unit: 'reps', min: 5, max: 0 }]),
      shareExercises([{ name: 'Squats', unit: 'reps', min: 6, max: 5 }]),
      shareExercises([{ name: '', unit: 'reps', min: 1, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'reps', min: 1, max: 5, pause: -1 }]),
      shareExercises([{ name: 'Plank', unit: 's', min: 1, max: 5, timed: 'hours' }]),
      shareExercises([{ name: 'S'.repeat(MAX_NAME + 1), unit: 'reps', min: 1, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'r'.repeat(MAX_UNIT + 1), min: 1, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'reps\u200B', min: 1, max: 5 }]),
      shareExercises(Array.from({ length: MAX_ITEMS + 1 }, (_, i) => ({ name: `Squats ${i}`, unit: 'reps', min: 1, max: 5 }))),
    ];
    for (const text of invalid) expect(() => parseExercises(text)).toThrow('Ce texte n\'est pas une liste d\'exercices partagée valide.');
  });

  it('keeps only the first of two exercises with the same name', () => {
    const text = shareExercises([...exercises, { name: 'squats ', unit: 's', min: 1, max: 2 }]);
    expect(parseExercises(text)).toEqual(exercises.map(({ id, enabled, ...ex }) => ex));
  });

  it('times the exercises of older versions whose unit reads as a duration', () => {
    const units = ['s', ' Secondes ', 'min', 'reps', 'seconds held'];
    const old = { exercises: units.map((unit, i) => ({ name: `Exercise ${i}`, unit, min: 1, max: 5 })) };
    for (const version of [1, 2]) {
      const read = parseExercises(packed(version, shareExercises(exercises), old));
      expect(read.map((ex) => ex.timed)).toEqual(['seconds', 'seconds', 'minutes', undefined, undefined]);
    }
  });

  it('tells where to import a game pasted instead of exercises', () => {
    expect(() => parseExercises(shareGame(config.games[0]))).toThrow('Ce texte partage un jeu : l\'importer depuis l\'onglet Jeux, avec Importer un jeu.');
  });

  it('updates exercises with the same name, enabled or not, and adds the others, enabled, without touching the original', () => {
    const imported = [
      { name: 'squats', unit: 's', min: 20, max: 30, pause: 45 },
      { name: 'Push-ups', unit: 'reps', min: 8, max: 12, pause: 90 },
      { name: 'Lunges', unit: 'reps', min: 5, max: 8 },
    ];
    expect(mergeExercises(exercises, imported)).toEqual({
      exercises: [
        { id: 'squat', name: 'Squats', enabled: false, unit: 's', min: 20, max: 30, pause: 45 },
        exercises[1],
        exercises[2],
        { id: 'lunges', name: 'Lunges', enabled: true, unit: 'reps', min: 5, max: 8 },
      ],
      added: 1,
      updated: 1,
    });
    // Without an own pause in the import, the exercise goes back to the global pause.
    expect(mergeExercises(exercises, [{ name: 'Push-ups', unit: 'reps', min: 8, max: 12 }]).exercises[1]).toEqual({ id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12 });
    // Likewise without a timer, the exercise is no longer timed; with one, it becomes timed.
    expect(mergeExercises(exercises, [{ name: 'Plank', unit: 'hold', min: 20, max: 45 }]).exercises[2]).not.toHaveProperty('timed');
    expect(mergeExercises(exercises, [{ name: 'Squats', unit: 'reps', min: 10, max: 15, timed: 'minutes' }]).exercises[0].timed).toBe('minutes');
    expect(exercises[0].unit).toBe('reps');
  });
});

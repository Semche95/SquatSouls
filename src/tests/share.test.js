import { deflateSync, strToU8 } from 'fflate';
import { describe, expect, it } from 'vitest';
import { MAX_ITEMS, MAX_NAME, MAX_UNIT } from '../configDraft.js';
import { parseExercises, parseGame, shareExercise, shareExercises, shareGame, toBase64, unpack } from '../share.js';
import { config } from './app.js';

// Shared text written in a given format version, with the content of `shared`.
function packed(version, shared, data = unpack(shared).data) {
  const payload = JSON.stringify({ version, kind: unpack(shared).kind, ...data });
  return shared.slice(0, shared.indexOf(':') + 1) + toBase64(deflateSync(strToU8(payload)));
}

const exercises = [
  { id: 'squat', name: 'Squats', enabled: false, unit: 'reps', min: 10, max: 15 },
  { id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12, recovery: 90 },
  { id: 'plank', name: 'Plank', enabled: true, unit: 'hold', min: 20, max: 45, timed: 'seconds', eachSide: true },
];

describe('game sharing', () => {
  it('shares the name and events without ids or exercises, then reads them back', () => {
    const text = shareGame(config.games[0]);
    expect(text).toMatch(/^squatsouls\/game\/elden-ring:/);
    expect(JSON.stringify(unpack(text).data)).not.toContain('"id"');
    expect(parseGame(text)).toEqual({
      name: 'Elden Ring',
      events: [
        { id: 'death', name: 'Death', draws: 1, jokers: 0, maxPerSession: 15 },
        { id: 'boss-defeated', name: 'Boss defeated', draws: 1, jokers: 2 },
      ],
      clamped: [],
    });
  });

  it('ignores the identifier placed in front of the compressed text', () => {
    const text = shareGame(config.games[0]);
    const payload = text.slice(text.indexOf(':') + 1);
    expect(parseGame(`squatsouls/exercises:${payload}`)).toEqual(parseGame(text));
    expect(parseGame(payload)).toEqual(parseGame(text));
  });

  it('shares the game\'s Steam AppId and reads a version 1 text, which has none', () => {
    const game = { ...config.games[0], steamAppId: 1245620 };
    expect(parseGame(shareGame(game)).steamAppId).toBe(1245620);
    const { steamAppId, ...v1 } = unpack(shareGame(game)).data;
    const read = parseGame(packed(1, shareGame(game), v1));
    expect(read).toEqual(parseGame(shareGame(config.games[0])));
    expect(read).not.toHaveProperty('steamAppId');
  });

  it('caps the requested exercises at 100, whatever the included exercises', () => {
    const text = shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 101, jokers: 0 }, { name: 'Boss', draws: 100, jokers: 0 }] });
    expect(parseGame(text)).toMatchObject({ events: [{ draws: 100 }, { draws: 100 }], clamped: ['death'] });
  });

  it('refuses a text that is not a valid shared game', () => {
    const invalid = [
      'some text',
      shareGame({ name: 'Nioh', events: [] }),
      shareGame({ name: ' ', events: [{ name: 'Death', draws: 1 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1, maxPerSession: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1, jokers: -1 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1, maxPerSession: 10000 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1, jokers: 10000 }] }),
      shareGame({ name: 'Nioh', steamAppId: 0, events: [{ name: 'Death', draws: 1 }] }),
      shareGame({ name: 'Nioh', steamAppId: '485510', events: [{ name: 'Death', draws: 1 }] }),
      shareGame({ name: 'N'.repeat(MAX_NAME + 1), events: [{ name: 'Death', draws: 1 }] }),
      ...['\n', '\t', '\u200B', '\u202E', '\uFEFF', '\u0085'].map((c) => shareGame({ name: 'Nioh', events: [{ name: `Death${c}`, draws: 1 }] })),
      shareGame({ name: 'Nioh', events: Array.from({ length: MAX_ITEMS + 1 }, () => ({ name: 'Death', draws: 1 })) }),
    ];
    for (const text of invalid) expect(() => parseGame(text)).toThrow('This text is not a valid shared game.');
  });

  it('keeps only the first of two events with the same name', () => {
    const text = shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1 }, { name: ' death ', draws: 9 }, { name: 'Boss', draws: 1 }] });
    expect(parseGame(text)).toMatchObject({ events: [{ name: 'Death', draws: 1 }, { name: 'Boss' }], clamped: [] });
  });

  it('accepts the emoji sequences, joined by a zero width joiner', () => {
    expect(parseGame(shareGame({ name: 'Nioh 👨‍👩‍👧', events: [{ name: 'Death', draws: 1 }] })).name).toBe('Nioh 👨‍👩‍👧');
  });

  it('refuses a pasted text or a content too large without decompressing it all', () => {
    const bomb = 'squatsouls/game:' + toBase64(deflateSync(new Uint8Array(100 * 1024 * 1024)));
    expect(() => parseGame(bomb)).toThrow('This text is not a valid shared game.');
    expect(() => parseGame(shareGame(config.games[0]) + ' '.repeat(300 * 1024))).toThrow('This text is not a valid shared game.');
  });

  it('tells where to import exercises pasted instead of a game', () => {
    expect(() => parseGame(shareExercises(exercises))).toThrow('This text shares exercises: import it from the Exercises tab, with Import.');
  });

  it('brings the limits and jokers of older versions within the bounds', () => {
    const old = { name: 'Nioh', events: [{ name: 'Death', draws: 1, maxPerSession: 10000, jokers: 10000 }] };
    for (const version of [1, 2, 3, 4, 5]) {
      expect(parseGame(packed(version, shareGame(config.games[0]), old)).events[0]).toMatchObject({ maxPerSession: 9999, jokers: 9999 });
    }
    expect(() => parseGame(packed(5, shareGame(config.games[0]), { name: 'Nioh', events: [{ name: 'Death', draws: 1, jokers: -1 }] }))).toThrow('This text is not a valid shared game.');
  });

  it('asks for an update for a text from an unknown format version', () => {
    const text = packed(999, shareGame(config.games[0]));
    expect(() => parseGame(text)).toThrow('This text comes from a newer version of SquatSouls: update it to import the text.');
  });

});

describe('exercise sharing', () => {
  it('shares a single exercise as a list of one, named in its identifier', () => {
    const text = shareExercise(exercises[0]);
    expect(text).toMatch(/^squatsouls\/exercise\/squats:/);
    expect(parseExercises(text)).toEqual(parseExercises(shareExercises([exercises[0]])));
  });

  it('shares the exercises without ids, with their own recovery, timer and sides, then reads them back', () => {
    const text = shareExercises(exercises);
    expect(text).toMatch(/^squatsouls\/exercises:/);
    expect(JSON.stringify(unpack(text).data)).not.toContain('"id"');
    expect(JSON.stringify(unpack(text).data)).toContain('"recovery":90');
    expect(JSON.stringify(unpack(text).data)).toContain('"timed":"seconds"');
    expect(JSON.stringify(unpack(shareExercises([{ ...exercises[0], eachSide: false }])).data)).not.toContain('eachSide');
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
      shareExercises([{ name: 'Squats', unit: 'reps', min: 0, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'reps', min: 1, max: 10000 }]),
      shareExercises([{ name: 'Squats', unit: 'reps', min: 1, max: 5, recovery: 3601 }]),
      shareExercises([{ name: '', unit: 'reps', min: 1, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'reps', min: 1, max: 5, recovery: -1 }]),
      shareExercises([{ name: 'Plank', unit: 's', min: 1, max: 5, timed: 'hours' }]),
      shareExercises([{ name: 'Plank', unit: 's', min: 1, max: 5, eachSide: true }]),
      packed(5, shareExercises(exercises), { exercises: [{ name: 'Plank', unit: 's', min: 1, max: 5, timed: 'seconds', eachSide: 'yes' }] }),
      shareExercises([{ name: 'S'.repeat(MAX_NAME + 1), unit: 'reps', min: 1, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'r'.repeat(MAX_UNIT + 1), min: 1, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'reps\u200B', min: 1, max: 5 }]),
      shareExercises(Array.from({ length: MAX_ITEMS + 1 }, (_, i) => ({ name: `Squats ${i}`, unit: 'reps', min: 1, max: 5 }))),
    ];
    for (const text of invalid) expect(() => parseExercises(text)).toThrow('This text is not a valid shared exercise list.');
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

  it('reads the exercises of older versions as done once', () => {
    const old = { exercises: [{ name: 'Plank', unit: 'hold', min: 20, max: 45, timed: 'seconds' }] };
    for (const version of [1, 2, 3, 4]) {
      expect(parseExercises(packed(version, shareExercises(exercises), old))[0]).not.toHaveProperty('eachSide');
    }
  });

  it('reads the own pause of older versions as the own recovery', () => {
    const old = { exercises: [{ name: 'Squats', unit: 'reps', min: 1, max: 5, pause: 45 }, { name: 'Push-ups', unit: 'reps', min: 1, max: 5 }] };
    for (const version of [1, 2, 3]) {
      const read = parseExercises(packed(version, shareExercises(exercises), old));
      expect(read).toEqual([{ name: 'Squats', unit: 'reps', min: 1, max: 5, recovery: 45 }, { name: 'Push-ups', unit: 'reps', min: 1, max: 5 }]);
    }
  });

  it('brings the numbers of older versions within the bounds', () => {
    const old = { exercises: [{ name: 'Squats', unit: 'reps', min: 0, max: 10000, recovery: 3601 }, { name: 'Lunges', unit: 'reps', min: 10000, max: 20000 }] };
    for (const version of [1, 2, 3, 4, 5]) {
      const read = parseExercises(packed(version, shareExercises(exercises), old));
      expect(read).toEqual([{ name: 'Squats', unit: 'reps', min: 1, max: 9999, recovery: 3600 }, { name: 'Lunges', unit: 'reps', min: 9999, max: 9999 }]);
    }
    for (const ex of [{ min: -1, max: 5 }, { min: 6, max: 5 }, { min: 0, max: 0 }, { min: 1, max: 5, recovery: -1 }]) {
      const text = packed(5, shareExercises(exercises), { exercises: [{ name: 'Squats', unit: 'reps', ...ex }] });
      expect(() => parseExercises(text)).toThrow('This text is not a valid shared exercise list.');
    }
  });

  it('tells where to import a game pasted instead of exercises', () => {
    expect(() => parseExercises(shareGame(config.games[0]))).toThrow('This text shares a game: import it from the Games tab, with Import a game.');
  });
});

import { describe, expect, it } from 'vitest';
import { importGame, mergeExercises } from '../importChanges.js';
import { config } from './app.js';

const exercises = [
  { id: 'squat', name: 'Squats', enabled: false, unit: 'reps', min: 10, max: 15 },
  { id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12, recovery: 90 },
  { id: 'plank', name: 'Plank', enabled: true, unit: 'hold', min: 20, max: 45, timed: 'seconds', eachSide: true },
];

describe('game import', () => {
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

describe('exercise import', () => {
  it('updates exercises with the same name, enabled or not, and adds the others, enabled, without touching the original', () => {
    const imported = [
      { name: 'squats', unit: 's', min: 20, max: 30, recovery: 45 },
      { name: 'Push-ups', unit: 'reps', min: 8, max: 12, recovery: 90 },
      { name: 'Lunges', unit: 'reps', min: 5, max: 8 },
    ];
    expect(mergeExercises(exercises, imported)).toEqual({
      exercises: [
        { id: 'squat', name: 'Squats', enabled: false, unit: 's', min: 20, max: 30, recovery: 45 },
        exercises[1],
        exercises[2],
        { id: 'lunges', name: 'Lunges', enabled: true, unit: 'reps', min: 5, max: 8 },
      ],
      added: 1,
      updated: 1,
    });
    // Without an own recovery in the import, the exercise goes back to the global recovery.
    expect(mergeExercises(exercises, [{ name: 'Push-ups', unit: 'reps', min: 8, max: 12 }]).exercises[1]).toEqual({ id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12 });
    // Likewise without a timer, the exercise is no longer timed; with one, it becomes timed. The same goes for the sides.
    expect(mergeExercises(exercises, [{ name: 'Plank', unit: 'hold', min: 20, max: 45 }]).exercises[2]).not.toHaveProperty('timed');
    expect(mergeExercises(exercises, [{ name: 'Plank', unit: 'hold', min: 20, max: 45, timed: 'seconds' }]).exercises[2]).not.toHaveProperty('eachSide');
    expect(mergeExercises(exercises, [{ name: 'Squats', unit: 'reps', min: 10, max: 15, timed: 'minutes' }]).exercises[0].timed).toBe('minutes');
    expect(mergeExercises(exercises, [{ name: 'Squats', unit: 'reps', min: 10, max: 15, timed: 'minutes', eachSide: true }]).exercises[0].eachSide).toBe(true);
    expect(exercises[0].unit).toBe('reps');
  });
});

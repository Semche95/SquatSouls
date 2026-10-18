import { describe, expect, it } from 'vitest';
import { nameKey } from '../configDraft.js';
import { appliedCount, defaultChoice, eventChanges, exerciseChanges, importGame, mergeExercises } from '../importChanges.js';
import { config } from './app.js';

const exercises = [
  { id: 'squat', name: 'Squats', enabled: false, unit: 'reps', min: 10, max: 15 },
  { id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12, recovery: 90 },
  { id: 'plank', name: 'Plank', enabled: true, unit: 'hold', min: 20, max: 45, timed: 'seconds', eachSide: true },
];

// Every item of an import chosen.
const all = (items) => new Set(items.map((it) => nameKey(it.name)));
const summed = (changes) => changes.map(({ key, status, name }) => [key, status, name]);

const [eldenRing, sekiro] = config.games;
const imported = {
  name: 'elden ring',
  events: [
    { id: 'death', name: 'death', draws: 1, jokers: 0, maxPerSession: 15 },
    { id: 'boss-defeated', name: 'Boss defeated', draws: 2, jokers: 2 },
    { id: 'grace', name: 'Grace', draws: 1, jokers: 0 },
  ],
  clamped: [],
};

describe('game import changes', () => {
  it('lists the new and changed events, then those of the replaced game missing from the import', () => {
    const game = { ...eldenRing, events: [...eldenRing.events, { id: 'rune', name: 'Rune lost', draws: 1, jokers: 0 }] };
    const changes = eventChanges(game, { ...imported, events: imported.events.slice(1) });
    expect(summed(changes)).toEqual([
      ['boss defeated', 'changed', 'Boss defeated'],
      ['grace', 'new', 'Grace'],
      ['death', 'absent', 'Death'],
      ['rune lost', 'absent', 'Rune lost'],
    ]);
    expect(changes[0].before).toBe(eldenRing.events[1]);
    expect(changes[0].after).toBe(imported.events[1]);
  });

  it('lists every event as new for a game the import adds', () => {
    expect(eventChanges(undefined, imported).map((c) => c.status)).toEqual(['new', 'new', 'new']);
  });

  it('chooses the additions and changes by default, not keeping the absent events', () => {
    const changes = eventChanges(sekiro, { name: 'Sekiro', events: [{ id: 'death', name: 'Death', draws: 1, jokers: 0 }] });
    expect(defaultChoice(changes)).toEqual(new Set(['death']));
    expect(appliedCount(changes, defaultChoice(changes))).toBe(2);
    expect(appliedCount(changes, new Set(['bonfire']))).toBe(0);
  });

  it('applies the chosen events in the import\'s order, keeping the others and the chosen absent ones', () => {
    const [game] = importGame(config.games, imported, new Set(['grace', 'death']), 'er');
    expect(game.id).toBe('er');
    expect(game.name).toBe('Elden Ring');
    expect(game.events).toEqual([eldenRing.events[0], eldenRing.events[1], imported.events[2]]);
    expect(importGame(config.games, imported, new Set(['boss defeated']), 'er')[0].events).toEqual([
      eldenRing.events[0],
      { id: 'boss', name: 'Boss defeated', draws: 2, jokers: 2 },
    ]);
  });

  it('gives a new event an id that no event of the replaced game holds', () => {
    const games = [{ id: 'er', name: 'Elden Ring', events: [{ id: 'grace', name: 'Site of grace', draws: 1, jokers: 0 }] }];
    const [game] = importGame(games, imported, new Set(['grace', 'site of grace']), 'er');
    expect(game.events.map((ev) => ev.id)).toEqual(['grace-2', 'grace']);
  });

  it('replaces the chosen game whatever the imported name, keeping its id, name and place', () => {
    const games = importGame(config.games, imported, all(imported.events), 'sekiro');
    expect(games.map((g) => [g.id, g.name])).toEqual([
      ['er', 'Elden Ring'],
      ['sekiro', 'Sekiro'],
    ]);
    expect(games[0]).toBe(eldenRing);
    expect(games[1].events.map((ev) => ev.name)).toEqual(['death', 'Boss defeated', 'Grace']);
  });

  it('adds the game without target, with its chosen events only', () => {
    const games = importGame(config.games, { ...imported, name: 'Nioh' }, new Set(['grace']));
    expect(games.at(-1)).toEqual({ id: 'nioh', name: 'Nioh', events: [imported.events[2]] });
  });

  it('takes the imported Steam AppId, or else the replaced game\'s, even if carried by another game', () => {
    const events = [{ id: 'bonfire', name: 'Bonfire', draws: 1, jokers: 0 }];
    const games = [{ ...eldenRing, steamAppId: 1245620 }, { ...sekiro, steamAppId: 814380 }];
    const chosen = all(events);
    expect(importGame(games, { name: 'Sekiro', events }, chosen, 'sekiro')[1].steamAppId).toBe(814380);
    expect(importGame(games, { name: 'Sekiro', steamAppId: 814381, events }, chosen, 'sekiro')[1].steamAppId).toBe(814381);
    expect(importGame(games, { name: 'Nioh', steamAppId: 485510, events }, chosen).at(-1).steamAppId).toBe(485510);
    expect(importGame(games, { name: 'Sekiro no prosthetic', steamAppId: 814380, events }, chosen).at(-1).steamAppId).toBe(814380);
  });
});

describe('exercise import changes', () => {
  it('lists the new and changed exercises, never those missing from the import', () => {
    const changes = exerciseChanges(exercises, [
      { name: 'squats', unit: 'reps', min: 10, max: 15 },
      { name: 'Push-ups', unit: 'reps', min: 8, max: 12 },
      { name: 'Lunges', unit: 'reps', min: 5, max: 8 },
    ]);
    expect(summed(changes)).toEqual([
      ['push-ups', 'changed', 'Push-ups'],
      ['lunges', 'new', 'Lunges'],
    ]);
  });

  it('updates exercises with the same name, enabled or not, and adds the others, enabled, without touching the original', () => {
    const shared = [
      { name: 'squats', unit: 's', min: 20, max: 30, recovery: 45 },
      { name: 'Push-ups', unit: 'reps', min: 8, max: 12, recovery: 90 },
      { name: 'Lunges', unit: 'reps', min: 5, max: 8 },
    ];
    expect(mergeExercises(exercises, shared, all(shared))).toEqual({
      exercises: [
        { id: 'squat', name: 'Squats', enabled: false, unit: 's', min: 20, max: 30, recovery: 45 },
        exercises[1],
        exercises[2],
        { id: 'lunges', name: 'Lunges', enabled: true, unit: 'reps', min: 5, max: 8 },
      ],
      added: 1,
      updated: 1,
    });
    const merge = (ex) => mergeExercises(exercises, [ex], all([ex])).exercises;
    // Without an own recovery in the import, the exercise goes back to the global recovery.
    expect(merge({ name: 'Push-ups', unit: 'reps', min: 8, max: 12 })[1]).toEqual({ id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12 });
    // Likewise without a timer, the exercise is no longer timed; with one, it becomes timed. The same goes for the sides.
    expect(merge({ name: 'Plank', unit: 'hold', min: 20, max: 45 })[2]).not.toHaveProperty('timed');
    expect(merge({ name: 'Plank', unit: 'hold', min: 20, max: 45, timed: 'seconds' })[2]).not.toHaveProperty('eachSide');
    expect(merge({ name: 'Squats', unit: 'reps', min: 10, max: 15, timed: 'minutes' })[0].timed).toBe('minutes');
    expect(merge({ name: 'Squats', unit: 'reps', min: 10, max: 15, timed: 'minutes', eachSide: true })[0].eachSide).toBe(true);
    expect(exercises[0].unit).toBe('reps');
  });

  it('applies only the chosen exercises', () => {
    const shared = [
      { name: 'Squats', unit: 'reps', min: 20, max: 30 },
      { name: 'Lunges', unit: 'reps', min: 5, max: 8 },
    ];
    expect(mergeExercises(exercises, shared, new Set(['lunges']))).toMatchObject({ exercises: [exercises[0], exercises[1], exercises[2], { id: 'lunges' }], added: 1, updated: 0 });
  });
});

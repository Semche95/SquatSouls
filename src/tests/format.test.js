import { describe, expect, it } from 'vitest';
import { formatCountdown, formatDuration, sessionSummary } from '../i18n/format.js';
import { setLanguage } from '../i18n/i18n.js';

describe('formatCountdown', () => {
  it('shows minutes and seconds', () => {
    expect(formatCountdown(5)).toBe('0:05');
    expect(formatCountdown(90)).toBe('1:30');
  });
});

describe('formatDuration', () => {
  it('rounds to the minute and switches to hours', () => {
    expect(formatDuration(20)).toBe('less than a minute');
    expect(formatDuration(25 * 60)).toBe('25 min');
    expect(formatDuration(65 * 60)).toBe('1 h 05 min');
  });

  it('follows the language', async () => {
    await setLanguage('fr');
    expect(formatDuration(20)).toBe('moins d\'une minute');
    expect(formatDuration(65 * 60)).toBe('1 h 05');
  });
});

describe('sessionSummary', () => {
  const summary = {
    game: 'Elden Ring',
    duration: 130 * 60,
    events: [{ event: 'Death', count: 14 }, { event: 'Boss defeated', count: 3 }],
    totals: [{ exercise: 'Squats', amount: 186, unit: 'reps' }, { exercise: 'Push-ups', amount: 40, unit: 'reps' }],
    jokers: { earned: 5, spent: 2 },
  };

  it('lists the game, events, exercises and jokers on separate lines', () => {
    expect(sessionSummary(summary)).toBe(
      'Elden Ring · 2 h 10 min\nDeath ×14 · Boss defeated ×3\nSquats 186 reps · Push-ups 40 reps\nJokers: 5 earned, 2 used',
    );
  });

  it('leaves out what was not recorded', () => {
    expect(sessionSummary({ ...summary, events: [], jokers: undefined })).toBe('Elden Ring · 2 h 10 min\nSquats 186 reps · Push-ups 40 reps');
  });
});

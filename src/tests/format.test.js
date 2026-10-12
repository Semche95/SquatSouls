import { describe, expect, it } from 'vitest';
import { formatCountdown, formatDuration, sessionSummary } from '../format.js';
import { setLanguage } from '../i18n.js';

describe('formatCountdown', () => {
  it('shows minutes and seconds', () => {
    expect(formatCountdown(5)).toBe('0:05');
    expect(formatCountdown(90)).toBe('1:30');
  });
});

describe('formatDuration', () => {
  it('rounds to the minute and switches to hours', () => {
    expect(formatDuration(20)).toBe('moins d\'une minute');
    expect(formatDuration(25 * 60)).toBe('25 min');
    expect(formatDuration(65 * 60)).toBe('1 h 05');
  });

  it('follows the language', async () => {
    await setLanguage('en');
    expect(formatDuration(20)).toBe('less than a minute');
    expect(formatDuration(65 * 60)).toBe('1 h 05 min');
  });
});

describe('sessionSummary', () => {
  const summary = {
    game: 'Elden Ring',
    duration: 130 * 60,
    events: [{ event: 'Mort', count: 14 }, { event: 'Boss vaincu', count: 3 }],
    totals: [{ exercise: 'Squats', amount: 186, unit: 'rép.' }, { exercise: 'Pompes', amount: 40, unit: 'rép.' }],
    jokers: { earned: 5, spent: 2 },
  };

  it('lists the game, events, exercises and jokers on separate lines', () => {
    expect(sessionSummary(summary)).toBe(
      'Elden Ring · 2 h 10\nMort ×14 · Boss vaincu ×3\nSquats 186 rép. · Pompes 40 rép.\nJokers : 5 gagnés, 2 utilisés',
    );
  });

  it('leaves out what was not recorded', () => {
    expect(sessionSummary({ ...summary, events: [], jokers: undefined })).toBe('Elden Ring · 2 h 10\nSquats 186 rép. · Pompes 40 rép.');
  });
});

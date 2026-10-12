import { describe, expect, it } from 'vitest';
import { formatCountdown, formatDuration } from '../format.js';
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

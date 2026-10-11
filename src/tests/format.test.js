import { describe, expect, it } from 'vitest';
import { formatCountdown, formatDuration, plural } from '../format.js';

describe('plural', () => {
  it('accorde au-delà de un', () => {
    expect(plural(1, 'joker')).toBe('1 joker');
    expect(plural(2, 'joker')).toBe('2 jokers');
  });
});

describe('formatCountdown', () => {
  it('affiche minutes et secondes', () => {
    expect(formatCountdown(5)).toBe('0:05');
    expect(formatCountdown(90)).toBe('1:30');
  });
});

describe('formatDuration', () => {
  it('arrondit à la minute et passe aux heures', () => {
    expect(formatDuration(20)).toBe('moins d\'une minute');
    expect(formatDuration(25 * 60)).toBe('25 min');
    expect(formatDuration(65 * 60)).toBe('1 h 05');
  });
});

import { describe, expect, it } from 'vitest';
import { pickSlogan, slogans } from '../slogans.js';
import { setup, mountApp } from './app.js';

describe('slogan', () => {
  it('affiche un slogan de la liste', async () => {
    setup();
    const w = await mountApp();
    expect(slogans).toContain(w.find('.splash').text());
  });

  it('n\'affiche un slogan de nouveau qu\'une fois tous les autres affichés', () => {
    localStorage.clear();
    const firstRound = slogans.map(() => pickSlogan());
    expect(new Set(firstRound).size).toBe(slogans.length);
    expect(slogans).toContain(pickSlogan());
  });
});

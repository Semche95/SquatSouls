import { describe, expect, it } from 'vitest';
import { pickSlogan, slogans } from '../slogans.js';
import { setup, mountApp } from './app.js';

describe('slogan', () => {
  it('shows a slogan from the list', async () => {
    setup();
    const w = await mountApp();
    expect(slogans).toContain(w.find('.splash').text());
  });

  it('only shows a slogan again once all the others have been shown', () => {
    localStorage.clear();
    const firstRound = slogans.map(() => pickSlogan());
    expect(new Set(firstRound).size).toBe(slogans.length);
    expect(slogans).toContain(pickSlogan());
  });
});

import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { emptySession, setup, mountApp } from './app.js';

describe('summary', () => {
  it('loads the history of the last 7 days', async () => {
    setup({
      get_session: emptySession({
        gameId: 'er',
        sets: 1,
        totals: [{ exercise: 'Squats', unit: 'reps', sets: 1, amount: 10 }],
        log: [{ at: 0, amount: 10, unit: 'reps', exercise: 'Squats', event: 'Death' }],
      }),
      get_history: [
        { date: '2026-10-10', sets: 0, sessions: [] },
        {
          date: '2026-10-11',
          sets: 1,
          sessions: [
            { start: 0, duration: 3900, game: 'Elden Ring', sets: 1, totals: [{ exercise: 'Squats', unit: 'reps', amount: 10 }] },
          ],
        },
      ],
    });
    const w = await mountApp();
    expect(w.find('.tabs .badge').text()).toBe('1');
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('get_history', { days: 7 });
    expect(w.find('.current .group-head').text()).toContain('Elden Ring · en cours depuis');
    expect(w.find('.totals').text()).toContain('10 reps');
    const days = w.findAll('.days li');
    expect(days[0].text()).toContain('repos');
    expect(days[1].text()).toContain('Squats 10 reps');
    expect(days[1].text()).toContain('Elden Ring');
    expect(days[1].text()).toContain('1 h 05');
  });

  it('counts spent jokers and tells halves from skips', async () => {
    const squats = { exerciseId: 'squat', name: 'Squats', amount: 5, unit: 'reps' };
    setup({
      get_session: emptySession({
        jokersEarned: 3,
        skipped: [
          { at: 0, event: 'Death', items: [squats], jokers: 1, halved: true },
          { at: 0, event: 'Boss', items: [squats], jokers: 2, halved: false },
        ],
      }),
    });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    expect(w.text()).toContain('3 gagnés · 3 utilisés');
    const rows = w.findAll('.log li').map((li) => li.text());
    expect(rows.some((r) => r.includes('Death (moitié)'))).toBe(true);
    expect(rows.some((r) => r.includes('Boss (sauté)'))).toBe(true);
  });

  it('copies the summary of the current session and of a past one', async () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(3601 * 1000);
    const writeText = vi.fn().mockResolvedValue();
    vi.stubGlobal('navigator', { clipboard: { writeText } });
    setup({
      get_session: emptySession({
        gameId: 'er',
        sets: 1,
        totals: [{ exercise: 'Squats', unit: 'reps', sets: 1, amount: 10 }],
        log: [{ at: 0, amount: 10, unit: 'reps', exercise: 'Squats', event: 'Death' }],
        triggers: [{ gameId: 'er', eventId: 'boss', count: 1 }, { gameId: 'er', eventId: 'death', count: 2 }],
        jokersEarned: 2,
      }),
      get_history: [
        {
          date: '2026-10-11',
          sets: 1,
          sessions: [
            { start: 0, duration: 600, game: 'Sekiro', sets: 1, totals: [{ exercise: 'Squats', unit: 'reps', amount: 8 }], events: [] },
          ],
        },
      ],
    });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    const current = w.find('.current .group-head button');
    await current.trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith('Elden Ring · 1 h 00\nDeath ×2 · Boss defeated ×1\nSquats 10 reps\nJokers : 2 gagnés, 0 utilisé');
    expect(current.text()).toBe('Copié');
    await w.find('.day-games button').trigger('click');
    expect(writeText).toHaveBeenLastCalledWith('Sekiro · 10 min\nSquats 8 reps');
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });
});

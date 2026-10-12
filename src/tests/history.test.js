import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { emptySession, setup, mountApp } from './app.js';

describe('history', () => {
  it.each([
    [null, 'Rien pour l\'instant.'],
    [1, 'Séance lancée. Le premier combat t\'attend.'],
  ])('encourages a session without sets, started at %s', async (startedAt, text) => {
    setup({ get_session: emptySession({ startedAt }) });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    expect(w.find('.current .empty').text()).toBe(text);
  });

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
      count_active_days: 12,
    });
    const w = await mountApp();
    expect(w.find('.tabs .badge').text()).toBe('1');
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('get_history', { days: 7 });
    expect(w.find('.current .group-head').text()).toContain('Elden Ring · en cours depuis');
    expect(w.find('.totals').text()).toContain('10 reps');
    expect(w.find('.group:not(.current) .group-title').text()).toBe('7 derniers jours');
    expect(w.find('.group:not(.current) .group-head').text()).toContain('par séance · 12 jours d\'activité');
    const days = w.findAll('.days li');
    expect(days[0].text()).toContain('repos');
    expect(days[1].text()).toContain('Squats 10 reps');
    expect(days[1].text()).toContain('Elden Ring');
    expect(days[1].text()).toContain('1 h 05');
  });

  it('counts jokers without listing what they removed', async () => {
    setup({
      get_session: emptySession({ jokersEarned: 3, jokersSpent: 3 }),
    });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    expect(w.text()).toContain('3 gagnés · 3 utilisés');
    expect(w.find('.log').exists()).toBe(false);
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

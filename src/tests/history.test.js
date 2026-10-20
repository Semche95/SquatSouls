import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { emptySession, setup, mountApp } from './app.js';
import { formatDay, formatTime } from '../i18n/format.js';
import { intersect } from './setup.js';

describe('history', () => {
  it.each([
    [null, 'Nothing yet.'],
    [1, 'Session underway. The first fight awaits.'],
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
    expect(w.find('.current .group-head').text()).toContain('Elden Ring · running since');
    expect(w.find('.totals').text()).toContain('10 reps');
    expect(w.find('.group:not(.current) .group-title').text()).toBe('Last 7 days');
    expect(w.find('.group:not(.current) .group-head').text()).toContain('by session · 12 days of activity');
    const days = w.findAll('.days li');
    expect(days[0].text()).toContain('rest');
    expect(days[1].text()).toContain('Squats 10 reps');
    expect(days[1].text()).toContain('Elden Ring');
    expect(days[1].text()).toContain('1 h 05');
  });

  it('loads the full history by pages of days, older ones as its end comes into view', async () => {
    // Active days, most recent first, from 2026-10-10.
    const days = (from, count) =>
      Array.from({ length: count }, (_, i) => ({
        date: new Date(Date.UTC(2026, 9, 10 - from - i)).toISOString().slice(0, 10),
        sets: 1,
        sessions: [],
      }));
    const pages = { null: days(0, 60), '2026-08-12': days(60, 1) };
    setup({ get_full_history: ({ before }) => pages[before] });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    await w.find('.group:not(.current) .group-head button').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('get_full_history', { before: null, days: 60 });
    expect(w.find('.full .group-title').text()).toBe('Full history');
    expect(w.findAll('.full .days li')).toHaveLength(60);

    intersect();
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('get_full_history', { before: '2026-08-12', days: 60 });
    expect(w.findAll('.full .days li')).toHaveLength(61);
    expect(w.findAll('.full .days li')[60].text()).toContain(formatDay('2026-08-11'));

    invoke.mockClear();
    intersect();
    await flushPromises();
    expect(invoke).not.toHaveBeenCalledWith('get_full_history', expect.anything());

    await w.find('.full .group-head button').trigger('click');
    expect(w.find('.full').exists()).toBe(false);
  });

  it('counts jokers without listing what they removed', async () => {
    setup({
      get_session: emptySession({ jokersEarned: 3, jokersSpent: 3 }),
    });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    expect(w.text()).toContain('3 earned · 3 used');
    expect(w.find('.log').exists()).toBe(false);
  });

  it('copies the summary of the current session, without its sets not started, and of a past one', async () => {
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
        // A second boss set is still to do, not started.
        triggers: [{ gameId: 'er', eventId: 'boss', count: 2 }, { gameId: 'er', eventId: 'death', count: 2 }],
        countedTriggers: [{ gameId: 'er', eventId: 'boss', count: 1 }, { gameId: 'er', eventId: 'death', count: 2 }],
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
    expect(writeText).toHaveBeenCalledWith('Elden Ring · 1 h 00 min\nDeath ×2 · Boss defeated ×1\nSquats 10 reps\nJokers: 2 earned, 0 used');
    expect(current.text()).toBe('Copied');
    await w.find('.day-games button').trigger('click');
    expect(writeText).toHaveBeenLastCalledWith('Sekiro · 10 min\nSquats 8 reps');
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('leaves out of the current session the time the app was closed', async () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(3601 * 1000);
    const writeText = vi.fn().mockResolvedValue();
    vi.stubGlobal('navigator', { clipboard: { writeText } });
    // Resumed after the app stayed closed for half an hour.
    setup({
      get_session: emptySession({
        countedFrom: 1801,
        sets: 1,
        totals: [{ exercise: 'Squats', unit: 'reps', sets: 1, amount: 10 }],
        log: [{ at: 0, amount: 10, unit: 'reps', exercise: 'Squats', event: 'Death' }],
      }),
    });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    await w.find('.current .group-head button').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith(expect.stringMatching(/^Elden Ring · 30 min\n/));
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('copies a whole day once it has several sessions', async () => {
    const writeText = vi.fn().mockResolvedValue();
    vi.stubGlobal('navigator', { clipboard: { writeText } });
    const session = (start, game, amount) => ({
      start,
      duration: 600,
      game,
      sets: 1,
      totals: [{ exercise: 'Squats', unit: 'reps', amount }, { exercise: 'Push-ups', unit: 'reps', amount: 5 }],
      events: [],
    });
    setup({
      get_history: [
        { date: '2026-10-12', sets: 1, sessions: [session(0, 'Sekiro', 8)] },
        { date: '2026-10-11', sets: 2, sessions: [session(0, 'Sekiro', 8), session(3600, 'Elden Ring', 10)] },
      ],
    });
    const w = await mountApp();
    await w.findAll('.tabs button')[1].trigger('click');
    await flushPromises();
    const days = w.findAll('.days li');
    expect(days[0].find('.day button').exists()).toBe(false);
    const copyDay = days[1].find('.day button');
    await copyDay.trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith(
      `${formatDay('2026-10-11')}\n${formatTime(0)} · Sekiro · 10 min\nSquats 8 reps · Push-ups 5 reps\n\n`
        + `${formatTime(3600)} · Elden Ring · 10 min\nSquats 10 reps · Push-ups 5 reps\n\nDay total: Squats 18 reps · Push-ups 10 reps`,
    );
    expect(copyDay.text()).toBe('Copied');
    vi.unstubAllGlobals();
  });
});

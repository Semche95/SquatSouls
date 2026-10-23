import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { emptySession, draw, responses, setup, mountApp, press } from './app.js';

// The current set, then `count` sets waiting, ids 8 onward.
const queued = (count) => Array.from({ length: count }, (_, i) => ({ ...draw, id: 8 + i }));
const withQueue = (count, extra = {}) => emptySession({ pending: [draw, ...queued(count)], ...extra });
const rows = (w) => w.findAll('.queue .queued');
const more = (w) => w.find('.queue-more');

// The backend gives the set `id`, added at the end of a queue of `count` sets.
function drawsNext(id, count) {
  responses.draw = { ...draw, id, capped: false };
  responses.get_session = emptySession({ pending: [draw, ...queued(count), { ...draw, id }] });
}

describe('queue', () => {
  it('shows a queued set in a compact row and only offers to dismiss it', async () => {
    const boss = { ...draw, id: 8, eventName: 'Boss defeated', jokers: 2, items: [...draw.items, { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps', halveCost: 1, skipCost: 2 }] };
    setup({ get_session: emptySession({ pending: [draw, boss], jokers: 5 }), dismiss: emptySession({ pending: [draw] }) });
    const w = await mountApp();
    const [row] = rows(w);
    expect(row.find('.queued-event').text()).toBe('Boss defeated +2 jokers');
    expect(row.findAll('.queued-items li').map((li) => li.text())).toEqual(['10 Squats reps', '8 Push-ups reps']);
    expect(row.findAll('button').map((b) => b.text())).toEqual(['Cancel']);
    await row.find('button').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('dismiss', { drawId: 8 });
    expect(rows(w)).toHaveLength(0);
  });

  it('shows up to 3 queued sets without folding', async () => {
    setup({ get_session: withQueue(3) });
    const w = await mountApp();
    expect(rows(w)).toHaveLength(3);
    expect(more(w).exists()).toBe(false);
  });

  it('folds the queued sets past 3, then shows them in order', async () => {
    setup({ get_session: withQueue(5) });
    const w = await mountApp();
    expect(rows(w)).toHaveLength(3);
    expect(more(w).text()).toBe('+2 sets waiting');
    expect(more(w).attributes('aria-expanded')).toBe('false');
    await more(w).trigger('click');
    expect(rows(w)).toHaveLength(5);
    expect(more(w).text()).toBe('Collapse');
    expect(more(w).attributes('aria-expanded')).toBe('true');
    expect(w.findAll('.queue > *').at(-1).classes()).toContain('queue-more');
    await more(w).trigger('click');
    expect(rows(w)).toHaveLength(3);
  });

  it('says a single folded set', async () => {
    setup({ get_session: withQueue(4) });
    const w = await mountApp();
    expect(more(w).text()).toBe('+1 set waiting');
  });

  it('folds again once 3 sets or fewer are left waiting', async () => {
    setup({ get_session: withQueue(4), dismiss: withQueue(3) });
    const w = await mountApp();
    await more(w).trigger('click');
    await rows(w)[0].find('button').trigger('click');
    await flushPromises();
    expect(more(w).exists()).toBe(false);
    drawsNext(20, 3);
    await press('Digit1');
    expect(rows(w)).toHaveLength(3);
    expect(more(w).text()).toBe('+1 set waiting');
  });

  it('folds again on coming back to Play', async () => {
    setup({ get_session: withQueue(5) });
    const w = await mountApp();
    await more(w).trigger('click');
    await w.findAll('.tabs button')[2].trigger('click');
    await w.findAll('.tabs button')[0].trigger('click');
    await flushPromises();
    expect(rows(w)).toHaveLength(3);
  });
});

describe('queue cues', () => {
  it('lights up the fold button for a set added to the folded part', async () => {
    setup({ get_session: withQueue(3) });
    const w = await mountApp();
    drawsNext(20, 3);
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(more(w).classes()).toContain('added');
    expect(w.find('.card.added').exists()).toBe(false);
    await more(w).trigger('animationend');
    expect(more(w).classes()).not.toContain('added');
  });

  it('lights it up on the key of an event too', async () => {
    setup({ get_session: withQueue(3) });
    const w = await mountApp();
    drawsNext(20, 3);
    await press('Digit1');
    expect(more(w).classes()).toContain('added');
  });

  it('lights up the set itself when the queue is shown', async () => {
    setup({ get_session: withQueue(4) });
    const w = await mountApp();
    await more(w).trigger('click');
    drawsNext(20, 4);
    await press('Digit1');
    expect(rows(w).at(-1).classes()).toContain('added');
    expect(more(w).classes()).not.toContain('added');
  });

  it('lights up a set added in view, not the fold button', async () => {
    setup({ get_session: withQueue(1) });
    const w = await mountApp();
    drawsNext(20, 1);
    await press('Digit1');
    expect(rows(w).at(-1).classes()).toContain('added');
    expect(more(w).exists()).toBe(false);
  });

  it('moves nothing on a resumed session', async () => {
    setup({ get_session: withQueue(5) });
    const w = await mountApp();
    expect(w.find('.added').exists()).toBe(false);
  });
});

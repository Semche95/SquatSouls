import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { emptySession, draw, responses, setup, mountApp, press } from './app.js';

const counted = (count) => [{ gameId: 'er', eventId: 'death', count }];

// The backend gives the set `id` and the session holding it.
function drawsNext(id, count) {
  responses.draw = { ...draw, id, capped: false };
  responses.get_session = emptySession({ pending: [{ ...draw, id }], triggers: counted(count) });
}

describe('session cues', () => {
  it('lights up the set added by clicking an event and grows its counter', async () => {
    setup();
    const w = await mountApp();
    drawsNext(7, 1);
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(w.find('.card.added').exists()).toBe(true);
    expect(w.find('.event .count').classes()).toContain('added');
    expect(w.findAll('.event')[1].find('.count').exists()).toBe(false);
  });

  it('lights up the set added by the key of an event', async () => {
    setup();
    const w = await mountApp();
    drawsNext(7, 1);
    await press('Digit1');
    expect(w.find('.card.added').exists()).toBe(true);
    expect(w.find('.event .count.added').exists()).toBe(true);
  });

  it('ends each cue with its animation and replays it at the next selection', async () => {
    setup();
    const w = await mountApp();
    drawsNext(7, 1);
    await press('Digit1');
    await w.find('.card').trigger('animationend');
    await w.find('.event .count').trigger('animationend');
    expect(w.find('.card.added').exists()).toBe(false);
    expect(w.find('.count.added').exists()).toBe(false);
    drawsNext(8, 2);
    await press('Digit1');
    expect(w.find('.card.added').exists()).toBe(true);
    expect(w.find('.event .count.added').exists()).toBe(true);
  });

  it('moves nothing when the limit is reached', async () => {
    setup({ get_session: emptySession({ pending: [draw], triggers: counted(15) }), draw: { capped: true, eventName: 'Death', maxPerSession: 15 } });
    const w = await mountApp();
    await press('Digit1');
    expect(w.find('.added').exists()).toBe(false);
  });

  it('moves nothing on a resumed session', async () => {
    setup({ get_session: emptySession({ pending: [draw], triggers: counted(1) }) });
    const w = await mountApp();
    expect(w.find('.added').exists()).toBe(false);
  });

  it('does not replay on coming back to Play', async () => {
    setup();
    const w = await mountApp();
    drawsNext(7, 1);
    await press('Digit1');
    await w.findAll('.tabs button')[2].trigger('click');
    await w.findAll('.tabs button')[0].trigger('click');
    await flushPromises();
    expect(w.find('.card').exists()).toBe(true);
    expect(w.find('.added').exists()).toBe(false);
  });
});

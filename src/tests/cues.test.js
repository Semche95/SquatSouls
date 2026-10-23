import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { emptySession, idleSession, draw, responses, setup, mountApp, press, expectTip } from './app.js';

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

describe('limit cue', () => {
  // The backend refuses the event: the session keeps its set and its counter stuck at the limit.
  const atLimit = () => setup({ get_session: emptySession({ sets: 1, triggers: counted(15) }), draw: { capped: true, eventName: 'Death', maxPerSession: 15 } });

  it('says the limit is reached on the tile clicked, without lighting up anything', async () => {
    atLimit();
    const w = await mountApp();
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(w.find('.event .limit-reached').text()).toBe('Limit reached');
    expect(w.find('.event .count').classes()).toContain('refused');
    expect(w.findAll('.limit-reached')).toHaveLength(1);
    expect(w.find('.added').exists()).toBe(false);
    expect(w.find('.banner').exists()).toBe(false);
    expect(w.find('[aria-live]').text()).toBe('Death: session limit of 15 reached.');
  });

  it('says it on the key of the event too', async () => {
    atLimit();
    const w = await mountApp();
    await press('Digit1');
    expect(w.find('.event .limit-reached').exists()).toBe(true);
  });

  it('ends the cue with its animation and replays it at the next selection', async () => {
    atLimit();
    const w = await mountApp();
    await press('Digit1');
    await w.find('.limit-reached').trigger('animationend');
    expect(w.find('.limit-reached').exists()).toBe(false);
    expect(w.find('.count.refused').exists()).toBe(false);
    expect(w.find('[aria-live]').text()).toBe('');
    await press('Digit1');
    expect(w.find('.limit-reached').exists()).toBe(true);
  });

  it('fades the counter of an event at its limit, without any cue on a resumed session', async () => {
    atLimit();
    const w = await mountApp();
    expect(w.find('.event .count').classes()).toContain('full');
    expect(w.findAll('.event')[1].find('.count').exists()).toBe(false);
    expect(w.find('.limit-reached').exists()).toBe(false);
    await expectTip(w.find('.event'), 'Limit reached: this event asks for nothing more until the next session.');
  });

  it('does not replay on coming back to Play', async () => {
    atLimit();
    const w = await mountApp();
    await press('Digit1');
    await w.findAll('.tabs button')[2].trigger('click');
    await w.findAll('.tabs button')[0].trigger('click');
    await flushPromises();
    expect(w.find('.limit-reached').exists()).toBe(false);
  });

  it('clears the cue when the session is cancelled', async () => {
    atLimit();
    responses.cancel_session = idleSession();
    const w = await mountApp();
    await press('Digit1');
    await w.find('.session-actions .danger').trigger('click');
    await flushPromises();
    expect(w.find('.limit-reached').exists()).toBe(false);
  });
});

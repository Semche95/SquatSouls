import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { emptySession, draw, responses, setup, mountApp, press, expectTip } from './app.js';

describe('keyboard shortcuts', () => {
  it('triggers the event matching the digit', async () => {
    setup({ draw: { capped: false } });
    await mountApp();
    await press('Digit2');
    expect(invoke).toHaveBeenCalledWith('draw', { gameId: 'er', eventId: 'boss', intensity: 'normal' });
    await press('Numpad1');
    expect(invoke).toHaveBeenCalledWith('draw', { gameId: 'er', eventId: 'death', intensity: 'normal' });
  });

  it('ignores a digit without an event and key repeats', async () => {
    setup({ draw: { capped: false } });
    await mountApp();
    await press('Digit9');
    await press('Digit1', { repeat: true });
    expect(invoke).not.toHaveBeenCalledWith('draw', expect.anything());
  });

  it('acts on the oldest set', async () => {
    const other = { ...draw, id: 8 };
    setup({ get_session: emptySession({ pending: [draw, other], jokers: 3 }), complete_exercise: () => responses.get_session, dismiss: () => responses.get_session, halve: () => responses.get_session, skip_exercise: () => responses.get_session });
    await mountApp();
    await press('Space');
    expect(invoke).not.toHaveBeenCalledWith('complete_exercise', expect.anything());
    await press('Enter');
    expect(invoke).toHaveBeenCalledWith('complete_exercise', { drawId: 7, exerciseId: 'squat' });
    await press('Backspace');
    expect(invoke).toHaveBeenCalledWith('dismiss', { drawId: 7 });
    await press('KeyH');
    expect(invoke).toHaveBeenCalledWith('halve', { drawId: 7, exerciseId: 'squat' });
    await press('KeyJ');
    expect(invoke).toHaveBeenCalledWith('skip_exercise', { drawId: 7, exerciseId: 'squat' });
  });

  it('targets the first remaining exercise of the set', async () => {
    const started = { ...draw, items: [{ ...draw.items[0], done: true }, { ...draw.items[0], exerciseId: 'push-ups', name: 'Push-ups' }] };
    setup({ get_session: emptySession({ pending: [started] }), complete_exercise: () => responses.get_session });
    await mountApp();
    await press('Enter');
    expect(invoke).toHaveBeenCalledWith('complete_exercise', { drawId: 7, exerciseId: 'push-ups' });
  });

  it('explains the joker shortcuts on the joker counter', async () => {
    setup({ get_session: emptySession({ pending: [draw], jokers: 3 }) });
    const wrapper = await mountApp();
    await expectTip(wrapper.find('.jokers'), 'H : Moitié, J : Sauter, sur le premier exercice restant');
  });

  it('does not use a joker without enough jokers', async () => {
    setup({ get_session: emptySession({ pending: [draw], jokers: 1 }) });
    await mountApp();
    await press('KeyJ');
    expect(invoke).not.toHaveBeenCalledWith('skip_exercise', expect.anything());
  });

  it('is inactive outside the Jouer tab and after the session ends', async () => {
    setup({ draw: { capped: false } });
    const w = await mountApp();
    await w.findAll('.tabs button')[2].trigger('click');
    await press('Digit1');
    expect(invoke).not.toHaveBeenCalledWith('draw', expect.anything());

  });

  it('ignores keys typed in a field', async () => {
    setup({ draw: { capped: false } });
    const w = await mountApp();
    w.find('select').element.dispatchEvent(new KeyboardEvent('keydown', { code: 'Digit1', bubbles: true }));
    await flushPromises();
    expect(invoke).not.toHaveBeenCalledWith('draw', expect.anything());
  });
});

import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, idleSession, draw, rejection, responses, setup, mountApp, press, expectTip, layOutControls, confirmDialog, confirmDialogOpen, confirmDialogClosed } from './app.js';
import { rt, tm } from '../i18n/i18n.js';
import { intersect } from './setup.js';

vi.mock('../session/sound.js', () => ({ chime: vi.fn(), recoveryEnd: vi.fn(), sideSignal: vi.fn(), startSignal: vi.fn() }));

describe('session', () => {
  layOutControls();


  it('draws a set on clicking an event', async () => {
    setup({ draw: { id: 7, capped: false } });
    const w = await mountApp();
    responses.get_session = emptySession({ pending: [draw], triggers: [{ gameId: 'er', eventId: 'death', count: 1 }] });
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('draw', { gameId: 'er', eventId: 'death', intensity: 'normal' });
    expect(w.find('.card.current').text()).toContain('Squats');
    expect(w.find('.event .count').text()).toBe('1 / 15');
    expect(w.find('#play-game').element.disabled).toBe(true);
  });

  it('only offers to dismiss queued sets', async () => {
    const both = { ...draw, items: [...draw.items, { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps', halveCost: 1, skipCost: 2 }] };
    setup({ get_session: emptySession({ pending: [draw, { ...both, id: 8 }], jokers: 5 }) });
    const w = await mountApp();
    const queued = w.find('.queue .card');
    expect(queued.find('.item-actions').exists()).toBe(false);
    expect(queued.findAll('.card-actions button').map((b) => b.text())).toEqual(['Cancel']);
  });

  it('marks each exercise of a set done', async () => {
    setup({
      get_session: emptySession({ pending: [draw], jokers: 1 }),
      complete_exercise: emptySession(),
    });
    const w = await mountApp();
    expect(w.findAll('.card-actions button').map((b) => b.text())).toEqual(['Cancel Backspace']);
    await w.find('.item-actions .primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('complete_exercise', { drawId: 7, exerciseId: 'squat' });
    expect(w.find('.card').exists()).toBe(false);
  });

  it('strikes through done exercises and targets the first remaining one', async () => {
    const pushUps = { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps', halved: false, done: false, halveCost: 1, skipCost: 2 };
    const started = { ...draw, items: [{ ...draw.items[0], done: true }, pushUps] };
    setup({ get_session: emptySession({ pending: [started], jokers: 4 }) });
    const w = await mountApp();
    const [done, left] = w.findAll('.items li');
    expect(done.classes()).toContain('done');
    expect(done.text()).toContain('done');
    expect(done.find('button').exists()).toBe(false);
    expect(left.findAll('button').map((b) => b.text())).toEqual(['Done Enter', 'Halve H', 'Skip J']);
    expect(w.text()).not.toContain('Skip the set');
  });

  it('only offers Done on the first remaining exercise', async () => {
    const pushUps = { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps', halved: false, done: false, halveCost: 1, skipCost: 2 };
    setup({ get_session: emptySession({ pending: [{ ...draw, items: [...draw.items, pushUps] }], jokers: 4 }) });
    const w = await mountApp();
    const [first, second] = w.findAll('.items li');
    expect(first.findAll('button').map((b) => b.text())).toEqual(['Done Enter', 'Halve H', 'Skip J']);
    expect(second.findAll('button').map((b) => b.text())).toEqual(['Halve', 'Skip']);
  });

  it('dismisses a set not started in one click', async () => {
    setup({ get_session: emptySession({ pending: [draw] }), dismiss: emptySession() });
    const w = await mountApp();
    await w.find('.card-actions .ghost').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('dismiss', { drawId: 7 });
  });

  it('stops a started set after confirmation', async () => {
    const started = { ...draw, items: [{ ...draw.items[0], done: true }, { ...draw.items[0], exerciseId: 'push-ups', name: 'Push-ups' }] };
    setup({ get_session: emptySession({ pending: [started] }), dismiss: emptySession() });
    const w = await mountApp();
    expect(w.find('.card-actions .ghost').text()).toBe('Stop Backspace');
    await press('Backspace');
    expect(w.find('.card .banner.confirm').text()).toContain('stay in the history');
    await press('Escape');
    expect(w.find('.card .banner.confirm').exists()).toBe(false);
    await w.find('.card-actions .ghost').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('dismiss', expect.anything());
    await w.find('.card .banner.confirm .primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('dismiss', { drawId: 7 });
    expect(w.find('.card').exists()).toBe(false);
  });

  describe('jokers', () => {
    const second = { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps', halved: false, halveCost: 1, skipCost: 2 };
    const both = { ...draw, items: [...draw.items, second] };
    const jokerButtons = (w) => w.findAll('.item-actions .ghost');

    it('are hidden without a joker', async () => {
      setup({ get_session: emptySession({ pending: [both] }) });
      const w = await mountApp();
      expect(jokerButtons(w)).toHaveLength(0);
      expect(w.text()).not.toContain('Skip the set');
    });

    it('halve or skip an exercise depending on available jokers', async () => {
      setup({ get_session: emptySession({ pending: [both], jokers: 1 }), halve: emptySession() });
      const w = await mountApp();
      const [half, skip] = jokerButtons(w);
      expect([half.text(), skip.text()]).toEqual(['Halve H', 'Skip J']);
      expect(half.attributes('disabled')).toBeUndefined();
      expect(skip.attributes('disabled')).toBeDefined();
      expect(w.find('.card-actions .ghost').text()).toBe('Skip the set (4 jokers)');
      expect(w.find('.card-actions .ghost').attributes('disabled')).toBeDefined();
      await half.trigger('click');
      expect(invoke).toHaveBeenCalledWith('halve', { drawId: 7, exerciseId: 'squat' });
    });

    it('skip the whole set with enough jokers', async () => {
      setup({ get_session: emptySession({ pending: [both], jokers: 4 }), skip_set: emptySession() });
      const w = await mountApp();
      await w.find('.card-actions .ghost').trigger('click');
      expect(invoke).toHaveBeenCalledWith('skip_set', { drawId: 7 });
    });

    it('no longer offer to halve an exercise already halved', async () => {
      const halved = { ...draw, items: [{ ...draw.items[0], amount: 5, halved: true, halveCost: null, skipCost: 1 }] };
      setup({ get_session: emptySession({ pending: [halved], jokers: 1 }) });
      const w = await mountApp();
      expect(jokerButtons(w).map((b) => b.text())).toEqual(['Skip J']);
      expect(w.find('.card .what').text()).toContain('halved');
    });
  });

  it('reloads the session if an action fails', async () => {
    setup({ get_session: emptySession({ pending: [draw] }), complete_exercise: rejection('set_not_pending') });
    const w = await mountApp();
    responses.get_session = emptySession();
    await w.find('.item-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('This set is no longer pending.');
    expect(w.find('.card').exists()).toBe(false);
  });

  it('locks the game during the session', async () => {
    setup({ get_session: emptySession({ gameId: 'sekiro' }) });
    const w = await mountApp();
    const select = w.find('select');
    expect(select.element.disabled).toBe(true);
    expect(select.element.value).toBe('sekiro');
  });

  it('keeps showing the session\'s game once it has no event left', async () => {
    const games = [{ ...config.games[0], events: [] }, config.games[1]];
    setup({ get_config: { config: { ...config, games } }, get_session: emptySession({ sets: 1 }) });
    const w = await mountApp();
    const select = w.find('#play-game');
    expect(select.element.value).toBe('er');
    expect(select.element.disabled).toBe(true);
    expect(w.find('.session-actions .primary').element.disabled).toBe(false);
  });

  describe('intensity', () => {
    const levels = (w) => w.findAll('.segmented button');

    it('draws at the chosen intensity, normal by default without reading the saved one', async () => {
      setup({ draw: { id: 7, capped: false }, set_intensity: undefined });
      const w = await mountApp();
      expect(invoke).not.toHaveBeenCalledWith('get_intensity');
      expect(w.find('.segmented .active').text()).toBe(tm('play.intensities').normal);
      await levels(w)[2].trigger('click');
      await w.find('.event').trigger('click');
      await flushPromises();
      expect(invoke).toHaveBeenCalledWith('draw', { gameId: 'er', eventId: 'death', intensity: 'hard' });
      expect(invoke).not.toHaveBeenCalledWith('set_intensity', expect.anything());
    });

    it('is read and saved when remembered', async () => {
      setup({ get_config: { config: { ...config, rememberIntensity: true } }, get_intensity: 'easy', set_intensity: undefined });
      const w = await mountApp();
      expect(w.find('.segmented .active').text()).toBe(tm('play.intensities').easy);
      await levels(w)[2].trigger('click');
      await flushPromises();
      expect(invoke).toHaveBeenCalledWith('set_intensity', { intensity: 'hard' });
    });

    it('changes during the session and applies to its sets to do', async () => {
      setup({
        get_session: emptySession({ gameId: 'sekiro', intensity: 'hard' }),
        set_session_intensity: emptySession({ gameId: 'sekiro', intensity: 'easy' }),
      });
      const w = await mountApp();
      expect(w.find('.segmented .active').text()).toBe(tm('play.intensities').hard);
      expect(levels(w).some((b) => b.element.disabled)).toBe(false);
      await levels(w)[0].trigger('click');
      await flushPromises();
      expect(invoke).toHaveBeenCalledWith('set_session_intensity', { intensity: 'easy' });
      expect(w.find('.segmented .active').text()).toBe(tm('play.intensities').easy);
    });

    it('starts the session at the chosen intensity', async () => {
      setup({ get_session: idleSession(), start_session: emptySession({ intensity: 'hard' }) });
      const w = await mountApp();
      await levels(w)[2].trigger('click');
      await w.find('.session-actions .primary').trigger('click');
      await flushPromises();
      expect(invoke).toHaveBeenCalledWith('start_session', { gameId: 'er', intensity: 'hard' });
      expect(w.find('.segmented .active').text()).toBe(tm('play.intensities').hard);
    });

    it('explains on hover how each level scales the exercises', async () => {
      setup();
      const w = await mountApp();
      await expectTip(levels(w)[0], 'Half the exercises requested, rounded down');
      await expectTip(levels(w)[1], 'The exercises requested by the program');
      await expectTip(levels(w)[2], 'One and a half times the exercises requested, rounded up');
    });
  });

  it('greys out a game without event in the game choice, and does not keep it chosen', async () => {
    const games = [...config.games, { id: 'nioh', name: 'Nioh', events: [] }];
    setup({ get_config: { config: { ...config, games } }, get_last_game: 'nioh' });
    const w = await mountApp();
    const nioh = w.findAll('#play-game option').find((o) => o.element.value === 'nioh');
    expect(nioh.text()).toBe('Nioh (no event)');
    expect(nioh.element.disabled).toBe(true);
    expect(w.find('#play-game').element.value).toBe('');
  });

  it('only offers to end the session with sets done and none pending', async () => {
    setup({ get_session: emptySession() });
    let w = await mountApp();
    expect(w.find('.session-actions .primary').element.disabled).toBe(true);
    await expectTip(w.find('.session-actions .primary'), 'No set done');
    setup({ get_session: emptySession({ sets: 1, pending: [draw] }) });
    w = await mountApp();
    expect(w.find('.session-actions .primary').element.disabled).toBe(true);
    await expectTip(w.find('.session-actions .primary'), 'There are sets left to do');
  });

  it('waits for the start button before starting the session', async () => {
    setup({ get_session: idleSession(), start_session: emptySession() });
    const w = await mountApp();
    expect(w.find('.session-actions .danger').element.disabled).toBe(true);
    const start = w.find('.session-actions .primary');
    expect(start.text()).toBe('Start the session');
    await start.trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('start_session', { gameId: 'er', intensity: 'normal' });
    expect(w.find('.session-actions .primary').text()).toBe('End the session');
    expect(w.find('.session-actions .danger').element.disabled).toBe(false);
  });

  it('cannot start a session before a game is chosen, then locks it from the start', async () => {
    setup({ get_last_game: '', get_session: idleSession() });
    const w = await mountApp();
    const start = () => w.find('.session-actions .primary');
    expect(start().element.disabled).toBe(true);
    await expectTip(start(), 'Choose a game');
    await w.find('#play-game').setValue('sekiro');
    expect(start().element.disabled).toBe(false);
    responses.start_session = emptySession({ gameId: 'sekiro' });
    await start().trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('start_session', { gameId: 'sekiro', intensity: 'normal' });
    expect(w.find('#play-game').element.disabled).toBe(true);
    await expectTip(w.find('#play-game'), 'The game can\'t be changed during a session');
  });

  it('cannot start a session while no game has an event', async () => {
    const games = config.games.map((g) => ({ ...g, events: [] }));
    setup({ get_config: { config: { ...config, games } }, get_session: idleSession() });
    const w = await mountApp();
    const start = w.find('.session-actions .primary');
    expect(start.element.disabled).toBe(true);
    await expectTip(start, 'No game with events');
  });

  it('cannot start a session nor trigger an event without an included exercise', async () => {
    const exercises = config.exercises.map((ex) => ({ ...ex, enabled: false }));
    setup({ get_config: { config: { ...config, exercises } }, get_session: idleSession() });
    const w = await mountApp();
    const start = w.find('.session-actions .primary');
    expect(start.element.disabled).toBe(true);
    await expectTip(start, 'No exercise included');
    const event = w.find('.event');
    expect(event.element.disabled).toBe(true);
    await expectTip(event, 'No exercise included');
    await press('Digit1');
    expect(invoke).not.toHaveBeenCalledWith('draw', expect.anything());
    const empty = w.findAll('.play .empty').find((p) => p.text().startsWith('No exercise included yet.'));
    expect(empty.text()).toBe('No exercise included yet. Exercises are created and included in Program.');
    await empty.find('button').trigger('click');
    expect(w.find('.cfg-exercises').exists()).toBe(true);
  });

  it('asks for confirmation before ending the session, then waits for the next one', async () => {
    setup({ get_session: emptySession({ sets: 1 }), end_session: idleSession() });
    const w = await mountApp();
    await w.find('.session-actions .primary').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('end_session', undefined);
    await confirmDialogOpen();
    expect(confirmDialog().text()).toContain('End the session?');
    await confirmDialog().find('.ghost').trigger('click');
    await confirmDialogClosed();
    await w.find('.session-actions .primary').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('end_session', undefined);
    await confirmDialogClosed();
    expect(w.find('.session-actions .primary').text()).toBe('Start the session');
    expect(w.find('.event').element.disabled).toBe(false);
  });

  it('shows a refused end and keeps the session', async () => {
    setup({ get_session: emptySession({ sets: 1 }), end_session: rejection('end_with_pending_sets') });
    const w = await mountApp();
    await w.find('.session-actions .primary').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toContain('There are sets left to do');
    expect(w.find('.farewell').exists()).toBe(false);
    expect(w.find('.session-actions .primary').text()).toBe('End the session');
  });

  it('disables cancelling while the session is not started', async () => {
    setup({ get_session: idleSession() });
    const w = await mountApp();
    expect(w.find('.session-actions .danger').element.disabled).toBe(true);
  });

  it('cancels the session without confirmation while it has no set', async () => {
    setup({ get_session: emptySession(), cancel_session: idleSession() });
    const w = await mountApp();
    await w.find('.session-actions .danger').trigger('click');
    await flushPromises();
    expect(document.querySelector('.confirm-modal-content')).toBeNull();
    expect(invoke).toHaveBeenCalledWith('cancel_session', undefined);
  });

  it('asks for confirmation before cancelling a session with a set to do', async () => {
    setup({ get_session: emptySession({ pending: [draw] }) });
    const w = await mountApp();
    await w.find('.session-actions .danger').trigger('click');
    await confirmDialogOpen();
    expect(invoke).not.toHaveBeenCalledWith('cancel_session', undefined);
  });

  it('asks for confirmation before cancelling a session with a set done', async () => {
    const log = [{ at: 0, amount: 10, unit: 'reps', exercise: 'Squats', event: 'Death' }];
    setup({ get_session: emptySession({ sets: 1, log }), cancel_session: idleSession() });
    const w = await mountApp();
    await w.find('.session-actions .danger').trigger('click');
    await confirmDialogOpen();
    expect(confirmDialog().text()).toContain('Cancel the session?');
    await confirmDialog().find('.ghost').trigger('click');
    await confirmDialogClosed();
    expect(invoke).not.toHaveBeenCalledWith('cancel_session', undefined);
    await w.find('.session-actions .danger').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.danger').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('cancel_session', undefined);
    expect(invoke).not.toHaveBeenCalledWith('end_session', undefined);
  });

  it('marks the session actions stuck only while the window cuts them off at its bottom', async () => {
    setup();
    const w = await mountApp();
    const bar = w.find('.session-actions');
    expect(bar.classes()).not.toContain('stuck');
    intersect(0.95);
    expect(bar.classes()).toContain('stuck');
    intersect();
    expect(bar.classes()).not.toContain('stuck');
  });

  it('encourages next to the session actions once it ends, until the next one starts', async () => {
    setup({ get_session: emptySession({ sets: 1 }), end_session: idleSession({ ended: { saved: true, cause: null } }), start_session: emptySession() });
    const w = await mountApp();
    expect(w.find('.farewell').exists()).toBe(false);
    await w.find('.session-actions .primary').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.primary').trigger('click');
    await flushPromises();
    expect(tm('session.farewells').map((m) => rt(m))).toContain(w.find('.session-actions .farewell').text());
    await w.find('.session-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.farewell').exists()).toBe(false);
  });
});

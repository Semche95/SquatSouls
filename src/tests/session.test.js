import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, draw, rejection, responses, setup, mountApp, press, expectTip } from './app.js';
import { rt, tm } from '../i18n.js';
import { chime, recoveryEnd, startSignal } from '../sound.js';

vi.mock('../sound.js', () => ({ chime: vi.fn(), recoveryEnd: vi.fn(), startSignal: vi.fn() }));

describe('session', () => {
  // Lets the countdown finish its leave transition before looking for it.
  async function sealShown(w) {
    await vi.advanceTimersByTimeAsync(50);
    return w.find('.countdown-seal').exists();
  }

  it('draws a set on clicking an event', async () => {
    setup({ draw: { id: 7, capped: false } });
    const w = await mountApp();
    responses.get_session = emptySession({ pending: [draw], triggers: [{ gameId: 'er', eventId: 'death', count: 1 }] });
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('draw', { gameId: 'er', eventId: 'death', intensity: 'normal' });
    expect(w.find('.card.current').text()).toContain('Squats');
    expect(w.find('.event .count').text()).toBe('1 / 15');
  });

  it('only offers to dismiss queued sets', async () => {
    const both = { ...draw, items: [...draw.items, { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps', halveCost: 1, skipCost: 2 }] };
    setup({ get_session: emptySession({ pending: [draw, { ...both, id: 8 }], jokers: 5 }) });
    const w = await mountApp();
    const queued = w.find('.queue .card');
    expect(queued.find('.item-actions').exists()).toBe(false);
    expect(queued.findAll('.card-actions button').map((b) => b.text())).toEqual(['Annuler']);
  });

  it('warns when the per-session limit is reached', async () => {
    setup({ draw: { capped: true, eventName: 'Death', maxPerSession: 15 } });
    const w = await mountApp();
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(w.find('.banner.info').text()).toBe('Death : limite de 15 atteinte pour la séance, pas de série.');
  });

  it('marks each exercise of a set done', async () => {
    setup({
      get_session: emptySession({ pending: [draw], jokers: 1 }),
      complete_exercise: emptySession(),
    });
    const w = await mountApp();
    expect(w.findAll('.card-actions button').map((b) => b.text())).toEqual(['Annuler Retour arr.']);
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
    expect(done.text()).toContain('fait');
    expect(done.find('button').exists()).toBe(false);
    expect(left.findAll('button').map((b) => b.text())).toEqual(['Fait Entrée', 'Moitié H', 'Sauter J']);
    expect(w.text()).not.toContain('Sauter la série');
  });

  it('only offers Fait on the first remaining exercise', async () => {
    const pushUps = { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps', halved: false, done: false, halveCost: 1, skipCost: 2 };
    setup({ get_session: emptySession({ pending: [{ ...draw, items: [...draw.items, pushUps] }], jokers: 4 }) });
    const w = await mountApp();
    const [first, second] = w.findAll('.items li');
    expect(first.findAll('button').map((b) => b.text())).toEqual(['Fait Entrée', 'Moitié H', 'Sauter J']);
    expect(second.findAll('button').map((b) => b.text())).toEqual(['Moitié', 'Sauter']);
  });

  it('dismisses a set not started in one click', async () => {
    setup({ get_session: emptySession({ pending: [draw] }), dismiss: emptySession() });
    const w = await mountApp();
    await w.find('.card-actions .ghost').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('dismiss', { drawId: 7 });
  });

  it('asks for confirmation before dismissing a started set', async () => {
    const started = { ...draw, items: [{ ...draw.items[0], done: true }, { ...draw.items[0], exerciseId: 'push-ups', name: 'Push-ups' }] };
    setup({ get_session: emptySession({ pending: [started] }), dismiss: emptySession() });
    const w = await mountApp();
    await press('Backspace');
    expect(w.find('.card .banner.confirm').text()).toContain('ne seront pas comptés');
    await press('Escape');
    expect(w.find('.card .banner.confirm').exists()).toBe(false);
    await w.find('.card-actions .ghost').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('dismiss', expect.anything());
    await w.find('.card .banner.confirm .danger').trigger('click');
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
      expect(w.text()).not.toContain('Sauter la série');
    });

    it('halve or skip an exercise depending on available jokers', async () => {
      setup({ get_session: emptySession({ pending: [both], jokers: 1 }), halve: emptySession() });
      const w = await mountApp();
      const [half, skip] = jokerButtons(w);
      expect([half.text(), skip.text()]).toEqual(['Moitié H', 'Sauter J']);
      expect(half.attributes('disabled')).toBeUndefined();
      expect(skip.attributes('disabled')).toBeDefined();
      expect(w.find('.card-actions .ghost').text()).toBe('Sauter la série (4 jokers)');
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
      expect(jokerButtons(w).map((b) => b.text())).toEqual(['Sauter J']);
      expect(w.find('.card .what').text()).toContain('moitié');
    });
  });

  describe('recovery after an exercise is done', () => {
    const pushUps = { exerciseId: 'push-ups', name: 'Push-ups', amount: 8, unit: 'reps' };
    const next = { ...draw, id: 8 };
    const doneButtons = (w) => w.findAll('.item-actions .primary');

    // Squats follow the global recovery, Push-ups have their own recovery; the first one of the set is marked done.
    async function completeOne(first = pushUps, recovery = config.recovery, own = 90, settings = {}) {
      vi.useFakeTimers();
      vi.mocked(recoveryEnd).mockClear();
      const both = { ...draw, items: first === pushUps ? [pushUps, draw.items[0]] : [...draw.items, pushUps] };
      setup({
        get_config: { config: { ...config, ...settings, recovery, exercises: [...config.exercises, { id: 'push-ups', name: 'Push-ups', unit: 'reps', min: 8, max: 12, recovery: own }] } },
        get_session: emptySession({ pending: [both, next] }),
        complete_exercise: emptySession({ pending: [both, next] }),
      });
      const w = await mountApp();
      await doneButtons(w)[0].trigger('click');
      await flushPromises();
      return w;
    }

    it('takes the exercise\'s own recovery and blocks every exercise until it ends', async () => {
      const w = await completeOne();
      expect(w.find('.countdown-left').text()).toBe('1:30');
      expect(doneButtons(w).every((b) => b.element.disabled)).toBe(true);
      await expectTip(doneButtons(w)[0], 'Récupération en cours');
      await press('Enter');
      expect(invoke.mock.calls.filter(([cmd]) => cmd === 'complete_exercise')).toHaveLength(1);
      await vi.advanceTimersByTimeAsync(30_000);
      expect(w.find('.countdown-left').text()).toBe('1:00');
      expect(recoveryEnd).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(60_000);
      expect(await sealShown(w)).toBe(false);
      expect(recoveryEnd).toHaveBeenCalledOnce();
      expect(doneButtons(w).some((b) => b.element.disabled)).toBe(false);
    });

    it('ends silently when sounds are off', async () => {
      await completeOne(pushUps, config.recovery, 90, { sound: false });
      await vi.advanceTimersByTimeAsync(90_000);
      expect(recoveryEnd).not.toHaveBeenCalled();
    });

    it('takes the global recovery without an own recovery', async () => {
      const w = await completeOne(draw.items[0]);
      expect(w.find('.countdown-left').text()).toBe('1:00');
    });

    it('is skipped with the button or Escape, without a sound', async () => {
      const w = await completeOne();
      await w.find('.countdown-seal').trigger('click');
      expect(await sealShown(w)).toBe(false);
      await doneButtons(w)[0].trigger('click');
      await flushPromises();
      expect(await sealShown(w)).toBe(true);
      await press('Escape');
      expect(await sealShown(w)).toBe(false);
      await vi.advanceTimersByTimeAsync(90_000);
      expect(recoveryEnd).not.toHaveBeenCalled();
    });

    it('stops when the set that started it is dismissed, not another one', async () => {
      const w = await completeOne();
      responses.dismiss = emptySession({ pending: [responses.get_session.pending[0]] });
      await w.findAll('.card')[1].find('.card-actions button').trigger('click');
      await flushPromises();
      expect(await sealShown(w)).toBe(true);
      responses.dismiss = emptySession();
      await w.findAll('.card')[0].find('.card-actions button').trigger('click');
      await flushPromises();
      expect(await sealShown(w)).toBe(false);
    });

    it('stays shown once no set is left to do', async () => {
      const w = await completeOne();
      responses.dismiss = emptySession();
      await w.findAll('.card')[1].find('.card-actions button').trigger('click');
      await flushPromises();
      expect(w.find('.empty .countdown-seal').exists()).toBe(true);
    });

    it('does not start a recovery when every recovery is 0', async () => {
      const w = await completeOne(pushUps, 0, 0);
      expect(await sealShown(w)).toBe(false);
    });
  });

  describe('timer of a timed exercise', () => {
    const timerButton = (w) => w.findAll('.item-actions button').find((b) => b.text() === 'Minuteur T');
    const doneButton = (w) => w.find('.item-actions .primary');
    const seal = (w) => [w.find('.countdown-seal .seal-label').text(), w.find('.countdown-left').text()];

    // Squats timed in `timed`, with the timer settings of `settings`; the set is the only one to do.
    async function mountTimed(timed = 'seconds', settings = {}, item = {}) {
      vi.useFakeTimers();
      vi.mocked(chime).mockClear();
      vi.mocked(startSignal).mockClear();
      const pending = [{ ...draw, items: [{ ...draw.items[0], ...item }] }];
      setup({
        get_config: { config: { ...config, ...settings, exercises: [{ ...config.exercises[0], timed }] } },
        get_session: emptySession({ pending, jokers: 2 }),
        complete_exercise: emptySession(),
      });
      return mountApp();
    }

    it('is only offered for a timed exercise', async () => {
      expect(timerButton(await mountTimed(null))).toBeUndefined();
      expect(timerButton(await mountTimed())).toBeDefined();
    });

    it('gets ready, sounds the start, runs, then sounds the end and marks the exercise done, which starts the recovery', async () => {
      const w = await mountTimed();
      await timerButton(w).trigger('click');
      expect(seal(w)).toEqual(['Prêt', '0:03']);
      expect(startSignal).not.toHaveBeenCalled();
      expect(doneButton(w).element.disabled).toBe(true);
      expect(timerButton(w).element.disabled).toBe(true);
      await expectTip(doneButton(w), 'Minuteur en cours');
      await press('Enter');
      expect(invoke).not.toHaveBeenCalledWith('complete_exercise', expect.anything());
      await vi.advanceTimersByTimeAsync(3000);
      expect(seal(w)).toEqual(['Minuteur', '0:10']);
      expect(startSignal).toHaveBeenCalledOnce();
      expect(chime).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(10_000);
      await flushPromises();
      expect(chime).toHaveBeenCalledOnce();
      expect(invoke).toHaveBeenCalledWith('complete_exercise', { drawId: 7, exerciseId: 'squat' });
      expect(seal(w)).toEqual(['Récup', '1:00']);
    });

    it('holds and resumes on a click on the countdown or Space', async () => {
      const w = await mountTimed('seconds', { timerPrep: 0 });
      await timerButton(w).trigger('click');
      await vi.advanceTimersByTimeAsync(4000);
      await w.find('.countdown-seal').trigger('click');
      expect(seal(w)).toEqual(['En pause', '0:06']);
      expect(w.find('.countdown-seal kbd').text()).toBe('Espace');
      await vi.advanceTimersByTimeAsync(20_000);
      expect(seal(w)).toEqual(['En pause', '0:06']);
      await press('Space');
      expect(seal(w)).toEqual(['Minuteur', '0:06']);
      await press('Space');
      expect(seal(w)).toEqual(['En pause', '0:06']);
      expect(w.find('.countdown-seal').attributes('aria-label')).toBe('Reprendre (Espace), Échap pour arrêter');
    });

    it('counts minutes and only ends silently when sound and auto done are off', async () => {
      const w = await mountTimed('minutes', { timerPrep: 0, sound: false, timerAutoDone: false }, { amount: 2 });
      await press('KeyT');
      expect(seal(w)).toEqual(['Minuteur', '2:00']);
      await vi.advanceTimersByTimeAsync(120_000);
      await flushPromises();
      expect(startSignal).not.toHaveBeenCalled();
      expect(chime).not.toHaveBeenCalled();
      expect(invoke).not.toHaveBeenCalledWith('complete_exercise', expect.anything());
      expect(await sealShown(w)).toBe(false);
      expect(doneButton(w).element.disabled).toBe(false);
    });

    it('stops with Escape, or when a joker changes its exercise', async () => {
      const w = await mountTimed();
      await timerButton(w).trigger('click');
      await press('Escape');
      expect(await sealShown(w)).toBe(false);
      await timerButton(w).trigger('click');
      responses.halve = emptySession({ pending: [{ ...draw, items: [{ ...draw.items[0], amount: 5, halved: true }] }], jokers: 1 });
      await press('KeyH');
      expect(await sealShown(w)).toBe(false);
      await vi.advanceTimersByTimeAsync(20_000);
      expect(invoke).not.toHaveBeenCalledWith('complete_exercise', expect.anything());
    });

    it('cannot start during the recovery', async () => {
      vi.useFakeTimers();
      const plank = { ...draw.items[0], exerciseId: 'plank', name: 'Plank' };
      const both = { ...draw, items: [draw.items[0], plank] };
      setup({
        get_config: { config: { ...config, exercises: [...config.exercises, { id: 'plank', name: 'Plank', enabled: true, unit: 's', min: 20, max: 45, timed: 'seconds' }] } },
        get_session: emptySession({ pending: [both] }),
        complete_exercise: emptySession({ pending: [{ ...both, items: [{ ...draw.items[0], done: true }, plank] }] }),
      });
      const w = await mountApp();
      await doneButton(w).trigger('click');
      await flushPromises();
      expect(timerButton(w).element.disabled).toBe(true);
      await press('KeyT');
      expect(seal(w)).toEqual(['Récup', '1:00']);
    });
  });

  it('reloads the session if an action fails', async () => {
    setup({ get_session: emptySession({ pending: [draw] }), complete_exercise: rejection('set_not_pending') });
    const w = await mountApp();
    responses.get_session = emptySession();
    await w.find('.item-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Cette série n\'est plus en attente.');
    expect(w.find('.card').exists()).toBe(false);
  });

  it('locks the game during the session', async () => {
    setup({ get_session: emptySession({ gameId: 'sekiro' }) });
    const w = await mountApp();
    const select = w.find('select');
    expect(select.element.disabled).toBe(true);
    expect(select.element.value).toBe('sekiro');
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
  });

  it('only offers to end the session with sets done and none pending', async () => {
    setup({ get_session: emptySession() });
    let w = await mountApp();
    expect(w.find('.session-actions .primary').element.disabled).toBe(true);
    await expectTip(w.find('.session-actions .primary'), 'Aucune série faite');
    setup({ get_session: emptySession({ sets: 1, pending: [draw] }) });
    w = await mountApp();
    expect(w.find('.session-actions .primary').element.disabled).toBe(true);
    await expectTip(w.find('.session-actions .primary'), 'Des séries sont encore à faire');
  });

  it('waits for the start button before starting the session', async () => {
    setup({ get_session: emptySession({ startedAt: null }), start_session: emptySession() });
    const w = await mountApp();
    expect(w.find('.session-actions .danger').element.disabled).toBe(true);
    const start = w.find('.session-actions .primary');
    expect(start.text()).toBe('Démarrer la séance');
    await start.trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('start_session', undefined);
    expect(w.find('.session-actions .primary').text()).toBe('Terminer la séance');
    expect(w.find('.session-actions .danger').element.disabled).toBe(false);
  });

  it('asks for confirmation before ending the session, then waits for the next one', async () => {
    setup({ get_session: emptySession({ sets: 1 }), end_session: emptySession({ startedAt: null }) });
    const w = await mountApp();
    await w.find('.session-actions .primary').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('end_session', undefined);
    expect(w.text()).toContain('Terminer la séance ?');
    await w.find('.confirm .ghost').trigger('click');
    expect(w.find('.confirm').exists()).toBe(false);
    await w.find('.session-actions .primary').trigger('click');
    await w.find('.confirm .primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('end_session', undefined);
    expect(w.find('.confirm').exists()).toBe(false);
    expect(w.find('.session-actions .primary').text()).toBe('Démarrer la séance');
    expect(w.find('.event').element.disabled).toBe(false);
  });

  it('shows a refused end and keeps the session', async () => {
    setup({ get_session: emptySession({ sets: 1 }), end_session: rejection('end_with_pending_sets') });
    const w = await mountApp();
    await w.find('.session-actions .primary').trigger('click');
    await w.find('.confirm .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toContain('Des séries sont encore à faire');
    expect(w.find('.farewell').exists()).toBe(false);
    expect(w.find('.session-actions .primary').text()).toBe('Terminer la séance');
  });

  it('disables cancelling while the session is not started', async () => {
    setup({ get_session: emptySession({ startedAt: null }) });
    const w = await mountApp();
    expect(w.find('.session-actions .danger').element.disabled).toBe(true);
  });

  it('asks for confirmation before cancelling the session', async () => {
    setup({ get_session: emptySession({ gameId: 'er' }), cancel_session: emptySession() });
    const w = await mountApp();
    await w.find('.session-actions .danger').trigger('click');
    expect(w.text()).toContain('Annuler la séance ?');
    await w.find('.confirm .ghost').trigger('click');
    expect(w.find('.confirm').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('cancel_session', undefined);
    await w.find('.session-actions .danger').trigger('click');
    await w.find('.confirm .danger').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('cancel_session', undefined);
    expect(invoke).not.toHaveBeenCalledWith('end_session', undefined);
  });

  it('encourages next to the session actions once it ends, until the next one starts', async () => {
    setup({ get_session: emptySession({ sets: 1 }), end_session: emptySession({ startedAt: null }), start_session: emptySession() });
    const w = await mountApp();
    expect(w.find('.farewell').exists()).toBe(false);
    await w.find('.session-actions .primary').trigger('click');
    await w.find('.confirm .primary').trigger('click');
    await flushPromises();
    expect(tm('session.farewells').map((m) => rt(m))).toContain(w.find('.session-actions .farewell').text());
    await w.find('.session-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.farewell').exists()).toBe(false);
  });
});

import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, idleSession, draw, responses, setup, mountApp, press, expectTip, layOutControls, confirmDialog, confirmDialogOpen, confirmDialogClosed, save } from './app.js';
import { chime, recoveryEnd, sideSignal, startSignal } from '../session/sound.js';

vi.mock('../session/sound.js', () => ({ chime: vi.fn(), recoveryEnd: vi.fn(), sideSignal: vi.fn(), startSignal: vi.fn() }));

describe('countdown', () => {
  layOutControls();

  // Lets the countdown finish its leave transition before looking for it.
  async function sealShown(w) {
    await vi.advanceTimersByTimeAsync(50);
    return w.find('.countdown-seal').exists();
  }

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
      await expectTip(doneButtons(w)[0], 'Recovery in progress');
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

    it('keeps running when the set that started it stops', async () => {
      const w = await completeOne();
      responses.dismiss = emptySession({ pending: [responses.get_session.pending[1]] });
      await w.findAll('.card')[0].find('.card-actions button').trigger('click');
      await flushPromises();
      expect(await sealShown(w)).toBe(true);
    });

    it('stops when the session ends without its button, its game removed', async () => {
      const w = await completeOne();
      expect(await sealShown(w)).toBe(true);
      // The confirmation dialog waits on real timers.
      vi.useRealTimers();
      responses.save_config = (args) => ({ config: args.config });
      responses.get_session = idleSession();
      await w.findAll('.tabs button')[2].trigger('click');
      await w.find('.cfg-game-bar .danger').trigger('click');
      await confirmDialogOpen();
      await confirmDialog().find('.danger').trigger('click');
      await flushPromises();
      await confirmDialogClosed();
      await w.findAll('.tabs button')[0].trigger('click');
      await vi.waitFor(() => expect(w.find('.countdown-seal').exists()).toBe(false));
      expect(recoveryEnd).not.toHaveBeenCalled();
    });

    it('stays shown once no set is left to do', async () => {
      const w = await completeOne();
      responses.dismiss = emptySession();
      await w.find('.queued button').trigger('click');
      await flushPromises();
      expect(w.find('.empty .countdown-seal').exists()).toBe(true);
    });

    // Saves the global recovery `seconds` in the Program tab, then goes back to Play.
    async function saveGlobalRecovery(w, seconds) {
      await w.findAll('.tabs button')[2].trigger('click');
      await w.findAll('.subtabs button')[1].trigger('click');
      await w.find('#cfg-recovery').setValue(seconds);
      await save(w).trigger('click');
      await flushPromises();
      await w.findAll('.tabs button')[0].trigger('click');
    }

    it('takes a saved recovery as if it had started with it, keeping the time elapsed', async () => {
      const w = await completeOne(draw.items[0]);
      responses.save_config = (args) => ({ config: args.config });
      await vi.advanceTimersByTimeAsync(10_000);
      await saveGlobalRecovery(w, 30);
      expect(w.find('.countdown-left').text()).toBe('0:20');
      // The ring empties over the new recovery: 10 s elapsed out of 30.
      expect(Number(w.find('.seal-ring').attributes('stroke-dashoffset'))).toBeCloseTo(-1 / 3);
      await saveGlobalRecovery(w, 120);
      expect(w.find('.countdown-left').text()).toBe('1:50');
      await saveGlobalRecovery(w, 0);
      expect(await sealShown(w)).toBe(false);
      expect(recoveryEnd).not.toHaveBeenCalled();
    });

    it('ends with the bell when a saved recovery is already past', async () => {
      const w = await completeOne(draw.items[0]);
      responses.save_config = (args) => ({ config: args.config });
      await vi.advanceTimersByTimeAsync(40_000);
      await saveGlobalRecovery(w, 30);
      expect(await sealShown(w)).toBe(false);
      expect(recoveryEnd).toHaveBeenCalledOnce();
    });

    it('stops without the bell when the saved recovery is already past but the save also ends the session', async () => {
      const w = await completeOne(draw.items[0]);
      responses.save_config = (args) => ({ config: args.config });
      responses.get_session = idleSession({ ended: { saved: true, cause: 'noExercise' } });
      await vi.advanceTimersByTimeAsync(40_000);
      await saveGlobalRecovery(w, 30);
      expect(await sealShown(w)).toBe(false);
      expect(recoveryEnd).not.toHaveBeenCalled();
    });

    it('keeps the exercise\'s own recovery when the global recovery changes', async () => {
      const w = await completeOne();
      responses.save_config = (args) => ({ config: args.config });
      await saveGlobalRecovery(w, 30);
      expect(w.find('.countdown-left').text()).toBe('1:30');
    });

    it('does not start a recovery when every recovery is 0', async () => {
      const w = await completeOne(pushUps, 0, 0);
      expect(await sealShown(w)).toBe(false);
    });
  });

  describe('timer of a timed exercise', () => {
    const timerButton = (w) => w.findAll('.item-actions button').find((b) => b.text() === 'Timer T');
    const doneButton = (w) => w.find('.item-actions .primary');
    const seal = (w) => [w.find('.countdown-seal .seal-label').text(), w.find('.countdown-left').text()];

    // Squats timed in `timed`, with the timer settings of `settings` and the exercise settings of `exercise`;
    // the set is the only one to do.
    async function mountTimed(timed = 'seconds', settings = {}, item = {}, exercise = {}) {
      vi.useFakeTimers();
      vi.mocked(chime).mockClear();
      vi.mocked(sideSignal).mockClear();
      vi.mocked(startSignal).mockClear();
      const pending = [{ ...draw, items: [{ ...draw.items[0], ...item }] }];
      setup({
        get_config: { config: { ...config, ...settings, exercises: [{ ...config.exercises[0], timed, ...exercise }] } },
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
      expect(seal(w)).toEqual(['Ready', '0:03']);
      expect(startSignal).not.toHaveBeenCalled();
      expect(doneButton(w).element.disabled).toBe(true);
      expect(timerButton(w).element.disabled).toBe(true);
      await expectTip(doneButton(w), 'Timer running');
      await press('Enter');
      expect(invoke).not.toHaveBeenCalledWith('complete_exercise', expect.anything());
      await vi.advanceTimersByTimeAsync(3000);
      expect(seal(w)).toEqual(['Timer', '0:10']);
      expect(startSignal).toHaveBeenCalledOnce();
      expect(chime).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(10_000);
      await flushPromises();
      expect(chime).toHaveBeenCalledOnce();
      expect(invoke).toHaveBeenCalledWith('complete_exercise', { drawId: 7, exerciseId: 'squat' });
      expect(seal(w)).toEqual(['Recovery', '1:00']);
    });

    it('holds and resumes on a click on the countdown or Space', async () => {
      const w = await mountTimed('seconds', { timerPrep: 0 });
      await timerButton(w).trigger('click');
      await vi.advanceTimersByTimeAsync(4000);
      await w.find('.countdown-seal').trigger('click');
      expect(seal(w)).toEqual(['Paused', '0:06']);
      expect(w.find('.countdown-seal kbd').text()).toBe('Space');
      await vi.advanceTimersByTimeAsync(20_000);
      expect(seal(w)).toEqual(['Paused', '0:06']);
      await press('Space');
      expect(seal(w)).toEqual(['Timer', '0:06']);
      await press('Space');
      expect(seal(w)).toEqual(['Paused', '0:06']);
      expect(w.find('.countdown-seal').attributes('aria-label')).toBe('Resume (Space), Esc to stop');
    });

    it('counts minutes and only ends silently when sound and auto done are off', async () => {
      const w = await mountTimed('minutes', { timerPrep: 0, sound: false, timerAutoDone: false }, { amount: 2 });
      await press('KeyT');
      expect(seal(w)).toEqual(['Timer', '2:00']);
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

    it('runs each side in turn, getting ready again in between without any recovery', async () => {
      const w = await mountTimed('seconds', {}, {}, { eachSide: true });
      await timerButton(w).trigger('click');
      expect(seal(w)).toEqual(['Ready', '0:03']);
      await vi.advanceTimersByTimeAsync(3000);
      expect(seal(w)).toEqual(['Side 1/2', '0:10']);
      await vi.advanceTimersByTimeAsync(10_000);
      expect(sideSignal).toHaveBeenCalledOnce();
      expect(chime).not.toHaveBeenCalled();
      expect(invoke).not.toHaveBeenCalledWith('complete_exercise', expect.anything());
      expect(seal(w)).toEqual(['Other side', '0:03']);
      await vi.advanceTimersByTimeAsync(3000);
      expect(seal(w)).toEqual(['Side 2/2', '0:10']);
      expect(startSignal).toHaveBeenCalledTimes(2);
      await vi.advanceTimersByTimeAsync(10_000);
      await flushPromises();
      expect(sideSignal).toHaveBeenCalledOnce();
      expect(chime).toHaveBeenCalledOnce();
      expect(invoke).toHaveBeenCalledWith('complete_exercise', { drawId: 7, exerciseId: 'squat' });
    });

    it('resumes at the side in progress after Escape, and forgets the first side once a joker changes the exercise', async () => {
      const w = await mountTimed('seconds', { timerPrep: 0 }, {}, { eachSide: true });
      await timerButton(w).trigger('click');
      await vi.advanceTimersByTimeAsync(4000);
      await press('Escape');
      await timerButton(w).trigger('click');
      expect(seal(w)).toEqual(['Side 1/2', '0:10']);
      await vi.advanceTimersByTimeAsync(12_000);
      await press('Escape');
      expect(await sealShown(w)).toBe(false);
      await timerButton(w).trigger('click');
      expect(seal(w)).toEqual(['Side 2/2', '0:10']);
      await press('Escape');
      responses.halve = emptySession({ pending: [{ ...draw, items: [{ ...draw.items[0], amount: 5, halved: true }] }], jokers: 1 });
      await press('KeyH');
      await flushPromises();
      await timerButton(w).trigger('click');
      expect(seal(w)).toEqual(['Side 1/2', '0:05']);
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
      expect(seal(w)).toEqual(['Recovery', '1:00']);
    });
  });
});

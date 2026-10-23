import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, idleSession, draw, responses, setup, mountApp, expectTip, layOutControls, openSettings, save, savedConfig } from './app.js';

describe('session limit', () => {
  layOutControls();
  const kind = (w) => w.find('.play-limit select');
  const value = (w) => w.find('#play-limit');
  const gauge = (w) => w.find('.session-limit');
  const exercises = { kind: 'exercises', value: 3 };

  it('is sent with the draw and the start of the session, none by default', async () => {
    setup({ draw: { id: 7, capped: false, limitReached: false }, get_session: idleSession(), start_session: (args) => emptySession({ limit: args.limit }) });
    const w = await mountApp();
    expect(invoke).not.toHaveBeenCalledWith('get_limit');
    expect(kind(w).element.value).toBe('none');
    expect(value(w).exists()).toBe(false);
    await kind(w).setValue('trainingTime');
    expect(value(w).element.value).toBe('30');
    await value(w).setValue('45');
    await w.find('.session-actions .primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('start_session', { gameId: 'er', intensity: 'normal', limit: { kind: 'trainingTime', value: 45 } });
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('draw', expect.objectContaining({ limit: { kind: 'trainingTime', value: 45 } }));
    expect(invoke).not.toHaveBeenCalledWith('set_limit', expect.anything());
  });

  it('starts each kind from its own value', async () => {
    setup();
    const w = await mountApp();
    for (const [k, start] of [['exercises', '20'], ['playTime', '60'], ['trainingTime', '30']]) {
      await kind(w).setValue(k);
      expect(value(w).element.value).toBe(start);
    }
  });

  it('keeps the last value filled correctly while the field is badly filled', async () => {
    setup({ draw: { id: 7, capped: false, limitReached: false } });
    const w = await mountApp();
    await kind(w).setValue('exercises');
    await value(w).setValue('12');
    await value(w).setValue('0');
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('draw', expect.objectContaining({ limit: { kind: 'exercises', value: 12 } }));
  });

  it('changes during the session, and is removed by choosing none', async () => {
    setup({ get_session: emptySession({ limit: exercises, asked: 1 }), set_session_limit: (args) => emptySession({ limit: args.limit, asked: 1 }) });
    const w = await mountApp();
    expect(kind(w).element.value).toBe('exercises');
    expect(value(w).element.value).toBe('3');
    await value(w).setValue('5');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_session_limit', { limit: { kind: 'exercises', value: 5 } });
    expect(gauge(w).text()).toBe('1 / 5 exercises');
    await kind(w).setValue('none');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_session_limit', { limit: null });
    expect(gauge(w).exists()).toBe(false);
    expect(value(w).exists()).toBe(false);
  });

  it('shows the progress, and once reached says so on the events', async () => {
    setup({ get_session: emptySession({ limit: exercises, asked: 3 }), draw: { ...draw, items: [], capped: false, limitReached: true } });
    const w = await mountApp();
    expect(gauge(w).text()).toBe('3 / 3 exercises');
    expect(gauge(w).classes()).toContain('full');
    await expectTip(gauge(w), 'Session limit reached: events ask for nothing more. Raising it brings sets back.');
    await expectTip(w.find('.event'), 'Session limit reached: events ask for nothing more. Raising it brings sets back.');
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(w.find('.limit-reached').exists()).toBe(true);
    expect(w.find('[aria-live]').text()).toBe('Death: session limit reached.');
  });

  it('counts the play time as it passes, without the time the app was closed', async () => {
    vi.useFakeTimers({ now: 1000 * 1000 });
    try {
      setup({ get_session: emptySession({ countedFrom: 1000 - 9 * 60, limit: { kind: 'playTime', value: 10 } }) });
      const w = await mountApp();
      expect(gauge(w).text()).toBe('9 / 10 min');
      await expectTip(gauge(w), 'Time of the session, without the time the app was closed.');
      await vi.advanceTimersByTimeAsync(60 * 1000);
      expect(gauge(w).text()).toBe('10 / 10 min');
      expect(gauge(w).classes()).toContain('full');
    } finally {
      vi.useRealTimers();
    }
  });

  it('counts the estimated training time, reached once a set did not fit', async () => {
    setup({ get_session: emptySession({ limit: { kind: 'trainingTime', value: 30 }, askedTime: 25 * 60, limitFull: true }) });
    const w = await mountApp();
    expect(gauge(w).text()).toBe('25 / 30 min');
    expect(gauge(w).classes()).toContain('full');
  });

  it('is read and saved when remembered', async () => {
    setup({ get_config: { config: { ...config, rememberLimit: true } }, get_limit: { kind: 'playTime', value: 60 }, set_limit: undefined });
    const w = await mountApp();
    expect(kind(w).element.value).toBe('playTime');
    expect(value(w).element.value).toBe('60');
    await value(w).setValue('90');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_limit', { limit: { kind: 'playTime', value: 90 } });
  });

  it('is saved as remembered from Settings', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await mountApp();
    await openSettings(w);
    await w.findAll('.check').find((c) => c.text() === 'Remember the session limit between launches').find('input').setValue(true);
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().rememberLimit).toBe(true);
  });

  describe('once a resumed session ends', () => {
    const resumed = (extra = {}) => setup({ get_session: emptySession({ limit: exercises }), cancel_session: idleSession(), ...extra });
    const cancel = async (w) => {
      await w.find('.session-actions .danger').trigger('click');
      await flushPromises();
    };

    it('goes back to none', async () => {
      resumed();
      const w = await mountApp();
      expect(value(w).element.value).toBe('3');
      await cancel(w);
      expect(kind(w).element.value).toBe('none');
    });

    it('keeps the remembered limit', async () => {
      resumed({ get_config: { config: { ...config, rememberLimit: true } }, get_limit: exercises, set_limit: undefined });
      const w = await mountApp();
      await cancel(w);
      expect(value(w).element.value).toBe('3');
    });

    it('keeps a limit chosen during the session', async () => {
      resumed({ set_session_limit: (args) => emptySession({ limit: args.limit }) });
      const w = await mountApp();
      await value(w).setValue('8');
      await flushPromises();
      responses.cancel_session = idleSession();
      await cancel(w);
      expect(value(w).element.value).toBe('8');
    });
  });
});

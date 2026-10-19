import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, draw, emptySession, idleSession, rejection, setup, mountApp, layOutControls, confirmDialog, confirmDialogOpen, confirmDialogClosed } from './app.js';
import { tm } from '../i18n.js';

describe('loading', () => {
  it('asks for the game on first launch', async () => {
    setup({ get_last_game: '' });
    const w = await mountApp();
    expect(w.find('select').element.value).toBe('');
    expect(w.find('.events').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('set_last_game', expect.anything());
  });

  it('shows the events of the last chosen game', async () => {
    setup();
    const w = await mountApp();
    expect(invoke).toHaveBeenCalledWith('get_config', undefined);
    const names = w.findAll('.event-name').map((e) => e.text());
    expect(names).toEqual(['Death', 'Boss defeated +2 jokers']);
    expect(w.text()).toContain('Nothing pending');
  });

  it('restores the last chosen game', async () => {
    setup({ get_last_game: 'sekiro' });
    const w = await mountApp();
    expect(w.find('select').element.value).toBe('sekiro');
    expect(w.findAll('.event-name').map((e) => e.text())).toEqual(['Bonfire']);
  });

  it('asks for the game again if the remembered game no longer exists', async () => {
    setup({ get_last_game: 'unknown' });
    const w = await mountApp();
    // "Choose a game" is shown, not an empty choice.
    expect(w.find('select').element.selectedIndex).toBe(0);
    expect(w.find('.events').exists()).toBe(false);
  });

  it('saves the chosen game', async () => {
    setup();
    const w = await mountApp();
    await w.find('select').setValue('sekiro');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_last_game', { gameId: 'sekiro' });
  });

  it('shows the backend error', async () => {
    setup({ get_config: rejection('internal') });
    const w = await mountApp();
    expect(w.find('.banner.error').text()).toBe('Internal error, the details are in the application logs.');
  });

  it('reports an invalid config outside the Program and Settings tabs', async () => {
    setup({ get_config: { config, error: { code: 'config_ignored', params: { cause: { code: 'exercise_empty', params: {} } } } } });
    const w = await mountApp();
    expect(w.find('.banner.error').text()).toContain('Saved configuration ignored, default configuration used. An exercise has an empty id or name.');
    await w.find('.banner.error button').trigger('click');
    expect(w.find('.cfg-events').exists()).toBe(true);
  });
  it('resumes the session in progress with its game and its own intensity', async () => {
    setup({
      get_config: { config: { ...config, rememberIntensity: true } },
      get_intensity: 'easy',
      get_last_game: 'sekiro',
      get_session: emptySession({ gameId: 'er', intensity: 'hard', pending: [draw] }),
    });
    const w = await mountApp();
    expect(w.find('select').element.value).toBe('er');
    expect(w.find('select').element.disabled).toBe(true);
    expect(w.find('.segmented .active').text()).toBe(tm('play.intensities').hard);
    expect(w.text()).toContain('Squats');
  });

  describe('once a resumed session ends', () => {
    layOutControls();
    const active = (w) => w.find('.segmented .active').text();
    const resumed = (extra = {}) => setup({ get_session: emptySession({ intensity: 'hard' }), cancel_session: idleSession(), end_session: idleSession(), ...extra });
    const cancel = async (w) => {
      await w.find('.session-actions .danger').trigger('click');
      await flushPromises();
    };

    it('goes back to normal when cancelled', async () => {
      resumed();
      const w = await mountApp();
      await cancel(w);
      expect(active(w)).toBe(tm('play.intensities').normal);
    });

    it('goes back to normal when ended', async () => {
      resumed({ get_session: emptySession({ intensity: 'hard', sets: 1 }) });
      const w = await mountApp();
      await w.find('.session-actions .primary').trigger('click');
      await confirmDialogOpen();
      await confirmDialog().find('.primary').trigger('click');
      await flushPromises();
      await confirmDialogClosed();
      expect(active(w)).toBe(tm('play.intensities').normal);
    });

    it('keeps the remembered intensity', async () => {
      resumed({ get_config: { config: { ...config, rememberIntensity: true } }, get_intensity: 'hard', set_intensity: undefined });
      const w = await mountApp();
      await cancel(w);
      expect(active(w)).toBe(tm('play.intensities').hard);
    });

    it('keeps an intensity chosen during the session', async () => {
      resumed({ set_session_intensity: emptySession({ intensity: 'easy' }) });
      const w = await mountApp();
      await w.findAll('.segmented button')[0].trigger('click');
      await flushPromises();
      await cancel(w);
      expect(active(w)).toBe(tm('play.intensities').easy);
    });
  });

  it('tells a session in progress that could not be resumed', async () => {
    setup({ get_session: idleSession({ ended: { saved: false, cause: 'unreadable' } }) });
    const w = await mountApp();
    expect(w.text()).toContain('The session ended: it could not be resumed. Its exercises done stay in the history.');
  });
});

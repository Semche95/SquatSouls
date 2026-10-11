import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, setup, mountApp } from './app.js';

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
    expect(w.text()).toContain('Rien en attente');
  });

  it('restores the last chosen game', async () => {
    setup({ get_last_game: 'sekiro' });
    const w = await mountApp();
    expect(w.find('select').element.value).toBe('sekiro');
    expect(w.findAll('.event-name').map((e) => e.text())).toEqual(['Bonfire']);
  });

  it('asks for the game again if the remembered game no longer exists', async () => {
    setup({ get_last_game: 'inconnu' });
    const w = await mountApp();
    // "Choisir un jeu" is shown, not an empty choice.
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
    setup({ get_config: new Error('config unreadable') });
    const w = await mountApp();
    expect(w.find('.banner.error').text()).toContain('config unreadable');
  });

  it('reports an invalid config outside the Configuration tab', async () => {
    setup({ get_config: { config, error: 'invalid JSON' } });
    const w = await mountApp();
    expect(w.find('.banner.error').text()).toContain('invalid JSON');
    await w.find('.banner.error button').trigger('click');
    expect(w.find('.cfg-events').exists()).toBe(true);
  });
});

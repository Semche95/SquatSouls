import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, setup, mountApp } from './app.js';

describe('chargement', () => {
  it('demande le jeu au premier lancement', async () => {
    setup({ get_last_game: '' });
    const w = await mountApp();
    expect(w.find('select').element.value).toBe('');
    expect(w.find('.events').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('set_last_game', expect.anything());
  });

  it('affiche les événements du dernier jeu choisi', async () => {
    setup();
    const w = await mountApp();
    expect(invoke).toHaveBeenCalledWith('get_config', undefined);
    const names = w.findAll('.event-name').map((e) => e.text());
    expect(names).toEqual(['Mort', 'Boss vaincu +2 jokers']);
    expect(w.text()).toContain('Rien en attente');
  });

  it('reprend le dernier jeu choisi', async () => {
    setup({ get_last_game: 'sekiro' });
    const w = await mountApp();
    expect(w.find('select').element.value).toBe('sekiro');
    expect(w.findAll('.event-name').map((e) => e.text())).toEqual(['Feu de camp']);
  });

  it('redemande le jeu si le jeu mémorisé n\'existe plus', async () => {
    setup({ get_last_game: 'inconnu' });
    const w = await mountApp();
    // « Choisir un jeu » est affiché, et non un choix vide.
    expect(w.find('select').element.selectedIndex).toBe(0);
    expect(w.find('.events').exists()).toBe(false);
  });

  it('enregistre le jeu choisi', async () => {
    setup();
    const w = await mountApp();
    await w.find('select').setValue('sekiro');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_last_game', { gameId: 'sekiro' });
  });

  it('affiche l\'erreur du backend', async () => {
    setup({ get_config: new Error('config illisible') });
    const w = await mountApp();
    expect(w.find('.banner.error').text()).toContain('config illisible');
  });

  it('signale une config invalide hors de l\'onglet Configuration', async () => {
    setup({ get_config: { config, error: 'JSON invalide' } });
    const w = await mountApp();
    expect(w.find('.banner.error').text()).toContain('JSON invalide');
    await w.find('.banner.error button').trigger('click');
    expect(w.find('.cfg-events').exists()).toBe(true);
  });
});

import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { rejection, setup, mountApp, expectTip, openSettings, save, discard, savedConfig } from './app.js';
import { shownGame, gameButton, rename } from './program.js';

describe('program and settings', () => {
  const remember = (w) => w.findAll('.check').find((c) => c.text() === 'Remember the intensity between launches').find('input');

  async function editBoth() {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await mountApp();
    await w.findAll('.tabs button')[2].trigger('click');
    await rename(w, 'Elden Ring DLC');
    await openSettings(w);
    await remember(w).setValue(true);
    return w;
  }

  it('saves only the changes of the Settings tab, the Program ones stay pending', async () => {
    const w = await editBoth();
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().rememberIntensity).toBe(true);
    expect(savedConfig().games[0].name).toBe('Elden Ring');
    await w.findAll('.tabs button')[2].trigger('click');
    expect(shownGame(w)).toBe('Elden Ring DLC');
    expect(save(w).element.disabled).toBe(false);
  });

  it('saves only the changes of the Program tab, the Settings ones stay pending', async () => {
    const w = await editBoth();
    await w.findAll('.tabs button')[2].trigger('click');
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().games[0].name).toBe('Elden Ring DLC');
    expect(savedConfig().rememberIntensity).toBeFalsy();
    await openSettings(w);
    expect(remember(w).element.checked).toBe(true);
  });

  it('saves the sound and timer settings with the Settings tab', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await mountApp();
    await openSettings(w);
    const check = (text) => w.findAll('.check').find((c) => c.text() === text).find('input');
    await check('Play sounds').setValue(false);
    await check('Mark the exercise done when the timer ends').setValue(false);
    await w.find('#timer-prep').setValue('4');
    await w.find('.stepper button[aria-label="More"]').trigger('click');
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig()).toMatchObject({ sound: false, timerAutoDone: false, timerPrep: 5 });
  });

  it('steps the countdown down to 0 and says there are unsaved changes', async () => {
    setup();
    const w = await mountApp();
    await openSettings(w);
    const less = () => w.find('.stepper button[aria-label="Less"]');
    for (let i = 0; i < 3; i++) await less().trigger('click');
    expect(w.find('#timer-prep').element.value).toBe('0');
    expect(less().element.disabled).toBe(true);
    await expectTip(less(), 'Already at 0');
    expect(w.find('.unsaved').text()).toBe('Unsaved changes');
  });

  it('resets only the changes of its own tab', async () => {
    const w = await editBoth();
    await discard(w).trigger('click');
    expect(remember(w).element.checked).toBe(false);
    expect(save(w).element.disabled).toBe(true);
    await w.findAll('.tabs button')[2].trigger('click');
    expect(shownGame(w)).toBe('Elden Ring DLC');
  });
});

describe('language', () => {
  async function mountSettings() {
    const w = await mountApp();
    await openSettings(w);
    return w;
  }
  const choices = (w) => w.findAll('.cfg-settings [role="radio"]');
  const chosen = (w) => w.find('.cfg-settings [aria-checked="true"]').text();
  const choose = (w, i) => choices(w)[i].trigger('click');

  it('offers the system language, then English and French written in their own language', async () => {
    setup();
    const w = await mountSettings();
    expect(w.find('.cfg-settings .setting').text()).toContain('Language');
    expect(choices(w).map((b) => b.text())).toEqual(['System language', 'English', 'Français']);
    expect(chosen(w)).toBe('System language');
  });

  it('switches right away without saving nor touching the pending changes', async () => {
    setup();
    const w = await mountSettings();
    await w.findAll('.tabs button')[2].trigger('click');
    await rename(w, 'Elden Ring DLC');
    await openSettings(w);
    await choose(w, 2);
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_language', { setting: 'fr' });
    expect(savedConfig()).toBeUndefined();
    expect(w.findAll('.tabs button').map((b) => b.text())).toEqual(['Jouer', 'Historique', 'Programme', 'Réglages', 'À propos']);
    expect(w.find('.cfg-settings .setting').text()).toContain('Langue');
    expect(choices(w)[0].text()).toBe('Langue du système');
    expect(chosen(w)).toBe('Français');
    expect(document.documentElement.lang).toBe('fr');
    await w.findAll('.tabs button')[2].trigger('click');
    expect(shownGame(w)).toBe('Elden Ring DLC');
    expect(gameButton(w, 'Renommer').exists()).toBe(true);
    expect(save(w).text()).toBe('Enregistrer');
  });

  it('shows the error and keeps the saved setting when the backend refuses', async () => {
    setup({ set_language: rejection('invalid_language', { value: 'fr' }) });
    const w = await mountSettings();
    await choose(w, 2);
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Unknown language: fr.');
    expect(chosen(w)).toBe('System language');
    expect(w.findAll('.tabs button')[0].text()).toBe('Play');
  });
});

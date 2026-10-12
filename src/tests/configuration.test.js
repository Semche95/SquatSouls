import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, rejection, responses, setup, mountApp, shownTip, expectTip } from './app.js';
import { shareExercises, shareGame } from '../share.js';

describe('configuration', () => {
  async function openConfig() {
    const w = await mountApp();
    await w.findAll('.tabs button')[2].trigger('click');
    return w;
  }

  // Without pending changes, the save and reset buttons stay shown but disabled.
  function expectUnchanged(w) {
    expect(w.findAll('.cfg-actions button').map((b) => b.element.disabled)).toEqual([true, true]);
  }

  it('shows the game being edited in the list', async () => {
    setup();
    const w = await openConfig();
    expect(w.find('.cfg-game-list .active').text()).toBe('Elden Ring');
  });

  it('edits the configuration with fields and applies it on save', async () => {
    vi.useFakeTimers();
    setup();
    const w = await openConfig();
    const save = () => w.find('.cfg-actions .primary');
    expectUnchanged(w);
    await expectTip(save(), 'Aucune modification en cours.');
    await w.find('input[aria-label="Nom de l\'événement"]').setValue('Brutal death');
    responses.save_config = (args) => ({ config: args.config });
    await save().trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.games[0].events[0]).toMatchObject({ id: 'death', name: 'Brutal death' });
    expect(w.find('.banner.ok').text()).toBe('Configuration enregistrée.');
    await vi.runAllTimersAsync();
    expect(w.find('.banner.ok').exists()).toBe(false);
    await w.findAll('.tabs button')[0].trigger('click');
    expect(w.find('.event').text()).toContain('Brutal death');
  });

  it('only renames the game in the list on save', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await w.find('input[aria-label="Nom du jeu"]').setValue('Elden Ring DLC');
    expect(w.find('.cfg-game-list .active').text()).toBe('Elden Ring');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.cfg-game-list .active').text()).toBe('Elden Ring DLC');
  });

  it('adds an exercise with an id derived from the name', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    expect(w.find('.cfg-game-list').exists()).toBe(false);
    const add = w.find('.cfg-exercises > button');
    await add.trigger('click');
    await add.trigger('click');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises.map((e) => e.id)).toEqual(['squat', 'nouvel-exercice', 'nouvel-exercice-2']);
  });

  it('creates an empty game or one with events copied from another game and saves it right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const savedGames = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config.games;
    async function create(name, source) {
      await w.find('.group-head .primary').trigger('click');
      await flushPromises();
      expect(w.find('form.cfg-game .primary').element.disabled).toBe(true);
      const nameInput = w.find('input[aria-label="Nom du nouveau jeu"]');
      expect(document.activeElement).toBe(nameInput.element);
      await nameInput.setValue(name);
      await w.find('select[aria-label="Événements du nouveau jeu"]').setValue(source);
      await w.find('form.cfg-game').trigger('submit');
      await flushPromises();
      expect(w.find('.cfg-game-list .active').text()).toBe(name);
      expectUnchanged(w);
      return savedGames().at(-1);
    }
    expect(await create('Nioh', 'sekiro')).toMatchObject({ id: 'nioh', events: config.games[1].events });
    expect(await create('Lies of P', '')).toMatchObject({ id: 'lies-of-p', events: [{ name: 'Nouvel événement', draws: 1 }] });
    expect(savedGames().map((g) => g.id)).toEqual(['er', 'sekiro', 'nioh', 'lies-of-p']);
    expect(w.findAll('.cfg-game-list button').map((b) => b.text())).toEqual(['Elden Ring', 'Lies of P', 'Nioh', 'Sekiro']);
  });

  it('refuses to create a game with a name already taken', async () => {
    setup();
    const w = await openConfig();
    await w.find('.group-head .primary').trigger('click');
    await flushPromises();
    await w.find('input[aria-label="Nom du nouveau jeu"]').setValue(' sekiro ');
    expect(w.find('form.cfg-game').text()).toContain('Un jeu porte déjà ce nom.');
    expect(w.find('form.cfg-game .primary').element.disabled).toBe(true);
    await expectTip(w.find('form.cfg-game .primary'), 'Un jeu porte déjà ce nom.');
    await w.find('input[aria-label="Nom du nouveau jeu"]').setValue(' ');
    await expectTip(w.find('form.cfg-game .primary'), 'Donner un nom au jeu');
  });

  it('asks for confirmation before removing a game, then saves right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const remove = () => w.find('.cfg-game .field .danger');
    await remove().trigger('click');
    await w.find('.cfg-game .confirm .ghost').trigger('click');
    expect(w.find('.cfg-game .confirm').exists()).toBe(false);
    expect(w.findAll('.cfg-game-list button')).toHaveLength(2);
    await remove().trigger('click');
    await w.find('.cfg-game .confirm .danger').trigger('click');
    await flushPromises();
    expect(w.findAll('.cfg-game-list button').map((b) => b.text())).toEqual(['Sekiro']);
    expect(invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config.games.map((g) => g.id)).toEqual(['sekiro']);
    expectUnchanged(w);
    expect(w.find('.cfg-game .confirm').exists()).toBe(false);
  });

  it('asks for the game again when the session\'s game is removed', async () => {
    setup({ get_session: emptySession({ gameId: 'er', sets: 1 }), save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    // The backend cancels the session of the removed game.
    responses.get_session = emptySession();
    await w.find('.cfg-game .field .danger').trigger('click');
    await w.find('.cfg-game .confirm .danger').trigger('click');
    await flushPromises();
    await w.findAll('.tabs button')[0].trigger('click');
    const select = w.find('select');
    expect(select.element.value).toBe('');
    expect(select.element.disabled).toBe(false);
    expect(w.find('.events').exists()).toBe(false);
  });

  it('sets an exercise\'s range and keeps at least one exercise', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    expect(w.find('.cfg-exercises .icon').element.disabled).toBe(true);
    await expectTip(w.find('.cfg-exercises .icon'), 'Il faut au moins un exercice');
    await w.find('input[aria-label="Maximum"]').setValue('');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises[0]).toMatchObject({ min: 10, max: 0 });
  });

  it('disables an exercise to keep it out of the draw without removing it', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    const row = w.find('.cfg-table tbody tr');
    expect(row.classes()).not.toContain('cfg-off');
    await w.find('input[aria-label="Proposer cet exercice"]').setValue(false);
    expect(row.classes()).toContain('cfg-off');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises).toEqual([expect.objectContaining({ id: 'squat', enabled: false })]);
  });

  it('sets the global pause and an exercise\'s own pause', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    const own = () => w.find('input[aria-label="Pause"]');
    await w.find('.cfg-pause input').setValue(45);
    expect(own().attributes('placeholder')).toBe('45');
    await own().setValue('90');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent()).toMatchObject({ pause: 45, exercises: [{ id: 'squat', pause: 90 }] });
    await own().setValue('');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent().exercises[0]).not.toHaveProperty('pause');
  });

  it('clears the per-session limit to remove it', async () => {
    setup();
    const w = await openConfig();
    await w.find('.cfg-params input[placeholder="aucune"]').setValue('');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.games[0].events[0]).not.toHaveProperty('maxPerSession');
  });

  it('resets unsaved changes', async () => {
    setup();
    const w = await openConfig();
    const name = () => w.find('input[aria-label="Nom du jeu"]');
    await name().setValue('Other');
    await w.find('.cfg-actions .ghost').trigger('click');
    expect(name().element.value).toBe('Elden Ring');
    expectUnchanged(w);
  });

  it('shows the save error', async () => {
    setup({ save_config: rejection('exercise_range', { name: 'Squats', min: 20, max: 15 }) });
    const w = await openConfig();
    await w.find('input[aria-label="Nom du jeu"]').setValue('Other');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Plage invalide pour « Squats » (20 à 15).');
  });

  it('reports an invalid format while typing', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    const min = w.find('input[aria-label="Minimum"]');
    await min.trigger('focus');
    await min.setValue('abc');
    await vi.waitFor(() => expect(shownTip()).toBe('il faut un nombre entier, 0 ou plus'));
    await min.trigger('blur');
    await vi.waitFor(() => expect(shownTip()).toBeUndefined());
    await min.trigger('mouseenter');
    await vi.waitFor(() => expect(shownTip()).toBe('il faut un nombre entier, 0 ou plus'));
    await min.trigger('mouseleave');
    await min.trigger('focus');
    expect(min.attributes('aria-invalid')).toBe('true');
    expect(w.find('.banner.error').exists()).toBe(false);
    await min.setValue('12');
    expect(min.attributes('aria-invalid')).toBe('false');
    await vi.waitFor(() => expect(shownTip()).toBeUndefined());
  });

  it('brings a cleared required field back to 0 when leaving it', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    const min = w.find('input[aria-label="Minimum"]');
    await min.setValue('');
    await min.trigger('change');
    expect(min.element.value).toBe('0');
  });

  it('disables saving while a field is badly filled and gives the reason', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    const min = w.find('input[aria-label="Minimum"]');
    const save = () => w.find('.cfg-actions .primary');
    await min.setValue('-2');
    expect(save().attributes('disabled')).toBeDefined();
    await expectTip(save(), 'Squats › minimum : il faut un nombre entier, 0 ou plus.');
    await min.setValue('2');
    expect(save().attributes('disabled')).toBeUndefined();
  });

  describe('sharing', () => {
    const savedConfig = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;

    it('copies the saved game and the exercises to the clipboard', async () => {
      const writeText = vi.fn();
      vi.stubGlobal('navigator', { clipboard: { writeText } });
      setup();
      const w = await openConfig();
      await w.find('input[aria-label="Nom du jeu"]').setValue('Unsaved');
      await w.find('.cfg-game .field .ghost').trigger('click');
      await flushPromises();
      expect(writeText).toHaveBeenLastCalledWith(shareGame(config.games[0]));
      expect(w.find('.banner.ok').text()).toBe('Copié dans le presse-papiers, prêt à partager.');
      await w.findAll('.subtabs button')[1].trigger('click');
      await w.find('.cfg-exercises .group-head .ghost').trigger('click');
      expect(writeText).toHaveBeenLastCalledWith(shareExercises(config.exercises));
      vi.unstubAllGlobals();
    });

    it('imports a shared game, replaces the one with the same name and saves it right away', async () => {
      setup({ save_config: (args) => ({ config: args.config }) });
      const w = await openConfig();
      const open = () => w.find('.group-head .ghost');
      await open().trigger('click');
      await flushPromises();
      const text = w.find('textarea[aria-label="Texte partagé"]');
      expect(document.activeElement).toBe(text.element);
      const submit = () => w.find('.cfg-import .primary');
      expect(submit().element.disabled).toBe(true);
      await text.setValue('not a game');
      expect(w.find('.cfg-import .banner.error').text()).toBe('Ce texte n\'est pas un jeu partagé valide.');
      expect(text.attributes('aria-invalid')).toBe('true');
      expect(submit().element.disabled).toBe(true);
      await text.setValue(shareGame({ name: 'sekiro', events: [{ name: 'Death', draws: 3, jokers: 1 }] }));
      expect(w.find('.cfg-import .banner.warn').text()).toBe('sekiro existe déjà : il sera remplacé.');
      await w.find('.cfg-import').trigger('submit');
      await flushPromises();
      expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro']);
      expect(savedConfig().games[1]).toEqual({ id: 'sekiro', name: 'sekiro', events: [{ id: 'death', name: 'Death', draws: 1, jokers: 1 }] });
      expect(w.find('.banner.ok').text()).toBe('sekiro importé, limité à 1 exercice par événement.');
      expect(w.find('.cfg-game-list .active').text()).toBe('sekiro');
      expectUnchanged(w);
      await open().trigger('click');
      await w.find('textarea').setValue(shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1 }] }));
      await w.find('.cfg-import').trigger('submit');
      await flushPromises();
      expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro', 'nioh']);
      expect(w.find('.banner.ok').text()).toBe('Nioh importé.');
    });

    it('imports shared exercises without losing pending changes', async () => {
      setup({ save_config: (args) => ({ config: args.config }) });
      const w = await openConfig();
      await w.find('input[aria-label="Nom du jeu"]').setValue('Elden Ring DLC');
      await w.findAll('.subtabs button')[1].trigger('click');
      await w.findAll('.cfg-exercises .group-head .ghost')[1].trigger('click');
      await w.find('textarea').setValue(
        shareExercises([
          { name: 'squats', unit: 'reps', min: 12, max: 20 },
          { name: 'Push-ups', unit: 'reps', min: 8, max: 12 },
        ]),
      );
      await w.find('.cfg-import').trigger('submit');
      await flushPromises();
      expect(savedConfig().exercises).toEqual([
        { id: 'squat', name: 'Squats', enabled: true, unit: 'reps', min: 12, max: 20 },
        { id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12 },
      ]);
      expect(savedConfig().games[0].name).toBe('Elden Ring');
      expect(w.find('.banner.ok').text()).toBe('Exercices importés : 1 ajouté, 1 mis à jour.');
      expect(w.find('.cfg-import').exists()).toBe(false);
      expect(w.findAll('input[aria-label="Nom de l\'exercice"]')).toHaveLength(2);
      await w.find('.cfg-actions .primary').trigger('click');
      await flushPromises();
      expect(savedConfig().games[0].name).toBe('Elden Ring DLC');
      expect(savedConfig().exercises).toHaveLength(2);
    });
  });
});

describe('language', () => {
  async function openSettings() {
    const w = await mountApp();
    await w.findAll('.tabs button')[2].trigger('click');
    await w.findAll('.subtabs button')[2].trigger('click');
    return w;
  }
  const select = (w) => w.find('.cfg-settings select');

  it('offers the system language, then English and French written in their own language', async () => {
    setup();
    const w = await openSettings();
    expect(w.find('.cfg-settings .setting').text()).toContain('Langue');
    expect(select(w).findAll('option').map((o) => [o.element.value, o.text()])).toEqual([
      ['system', 'Langue du système'],
      ['en', 'English'],
      ['fr', 'Français'],
    ]);
    expect(select(w).element.value).toBe('system');
  });

  it('switches right away without saving nor touching the pending changes', async () => {
    setup();
    const w = await openSettings();
    await w.findAll('.subtabs button')[0].trigger('click');
    await w.find('input[aria-label="Nom du jeu"]').setValue('Elden Ring DLC');
    await w.findAll('.subtabs button')[2].trigger('click');
    await select(w).setValue('en');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_language', { setting: 'en' });
    expect(invoke.mock.calls.some(([cmd]) => cmd === 'save_config')).toBe(false);
    expect(w.findAll('.tabs button').map((b) => b.text())).toEqual(['Play', 'Summary', 'Configuration']);
    expect(w.find('.cfg-settings .setting').text()).toContain('Language');
    expect(select(w).find('option').text()).toBe('System language');
    expect(document.documentElement.lang).toBe('en');
    await w.findAll('.subtabs button')[0].trigger('click');
    expect(w.find('input[aria-label="Name of the game"]').element.value).toBe('Elden Ring DLC');
    expect(w.find('.cfg-actions .primary').text()).toBe('Save');
  });

  it('shows the error and goes back to the saved setting when the backend refuses', async () => {
    setup({ set_language: rejection('invalid_language', { value: 'en' }) });
    const w = await openSettings();
    await select(w).setValue('en');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Langue inconnue : en.');
    expect(select(w).element.value).toBe('system');
    expect(w.findAll('.tabs button')[0].text()).toBe('Jouer');
  });
});

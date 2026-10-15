import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, rejection, responses, setup, mountApp, shownTip, expectTip, openSettings, layOutControls, formDialog, formDialogOpen, formDialogClosed } from './app.js';
import { shareExercises, shareGame } from '../share.js';

// Game shown in the select of the Program tab, and the games it offers.
const shownGame = (w) => w.find('#cfg-game option:checked').text();
const gameNames = (w) => w.findAll('#cfg-game option').map((o) => o.text());
const gameButton = (w, text) => w.findAll('.group > .field button').find((b) => b.text() === text);
// Renames the edited game with Rename then Done: the new name waits for Save.
async function rename(w, name) {
  await gameButton(w, 'Rename').trigger('click');
  await w.find('input[aria-label="Name of the game"]').setValue(name);
  await w.find('.group > form.field').trigger('submit');
}

describe('configuration', () => {
  layOutControls();
  async function openConfig() {
    const w = await mountApp();
    await w.findAll('.tabs button')[2].trigger('click');
    return w;
  }
  // Exercises sub-tab, with the first exercise's card opened unless asked otherwise.
  async function openExercises({ card = true } = {}) {
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    if (card) await w.find('.cfg-exercise .cfg-event-head').trigger('click');
    return w;
  }

  // Without pending changes, the save and reset buttons stay shown but disabled.
  function expectUnchanged(w) {
    expect(w.findAll('.cfg-actions button').map((b) => b.element.disabled)).toEqual([true, true]);
  }

  it('shows the game being edited in the select and switches games with it', async () => {
    setup();
    const w = await openConfig();
    expect(shownGame(w)).toBe('Elden Ring');
    await w.find('#cfg-game').setValue('sekiro');
    expect(shownGame(w)).toBe('Sekiro');
    expect(w.find('.cfg-event-head strong').text()).toBe(config.games[1].events[0].name);
  });

  it('edits the configuration with fields and applies it on save', async () => {
    vi.useFakeTimers();
    setup();
    const w = await openConfig();
    const save = () => w.find('.cfg-actions .primary');
    expectUnchanged(w);
    await expectTip(save(), 'No pending changes.');
    await w.find('input[aria-label="Name of the event"]').setValue('Brutal death');
    responses.save_config = (args) => ({ config: args.config });
    await save().trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.games[0].events[0]).toMatchObject({ id: 'death', name: 'Brutal death' });
    expect(w.find('.banner.ok').text()).toBe('Configuration saved.');
    await vi.runAllTimersAsync();
    expect(w.find('.banner.ok').exists()).toBe(false);
    await w.findAll('.tabs button')[0].trigger('click');
    expect(w.find('.event').text()).toContain('Brutal death');
  });

  it('renames the game from the end of its name, the new name waiting for save', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await gameButton(w, 'Rename').trigger('click');
    await flushPromises();
    const input = w.find('input[aria-label="Name of the game"]');
    expect(document.activeElement).toBe(input.element);
    expect([input.element.selectionStart, input.element.selectionEnd]).toEqual([10, 10]);
    expect(w.find('#cfg-game').element.tagName).toBe('INPUT');
    const done = () => w.find('.group > form.field .primary');
    await input.setValue(' ');
    await expectTip(done(), 'Give the game a name');
    await input.setValue(' sekiro ');
    await expectTip(done(), 'A game already has this name.');
    await input.setValue('Other');
    await input.trigger('keydown', { key: 'Escape' });
    expect(shownGame(w)).toBe('Elden Ring');
    expectUnchanged(w);
    await rename(w, ' Elden Ring DLC ');
    expect(shownGame(w)).toBe('Elden Ring DLC');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config.games[0].name).toBe('Elden Ring DLC');
  });

  it('adds an exercise with an id derived from the name', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    expect(w.find('#cfg-game').exists()).toBe(false);
    const add = w.find('.cfg-exercises > button');
    await add.trigger('click');
    await add.trigger('click');
    const cards = w.findAll('.cfg-exercise');
    expect(cards.map((c) => c.classes().includes('open'))).toEqual([false, true, true]);
    const name = cards[2].find('input[aria-label="Name of the exercise"]').element;
    expect(document.activeElement).toBe(name);
    expect([name.selectionStart, name.selectionEnd]).toEqual([0, name.value.length]);
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises.map((e) => e.id)).toEqual(['squat', 'new-exercise', 'new-exercise-2']);
  });

  it('creates an empty game or one with events copied from another game in a modal and saves it right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const savedGames = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config.games;
    async function create(name, source) {
      await w.find('.group-head .primary').trigger('click');
      await formDialogOpen();
      expect(formDialog().find('.primary').element.disabled).toBe(true);
      const nameInput = formDialog().find('input[aria-label="Name of the new game"]');
      expect(document.activeElement).toBe(nameInput.element);
      await nameInput.setValue(name);
      await formDialog().find('select[aria-label="Events of the new game"]').setValue(source);
      await formDialog().find('form').trigger('submit');
      await flushPromises();
      await formDialogClosed();
      expect(shownGame(w)).toBe(name);
      expectUnchanged(w);
      return savedGames().at(-1);
    }
    expect(await create('Nioh', 'sekiro')).toMatchObject({ id: 'nioh', events: config.games[1].events });
    expect(await create('Lies of P', '')).toMatchObject({ id: 'lies-of-p', events: [] });
    expect(savedGames().map((g) => g.id)).toEqual(['er', 'sekiro', 'nioh', 'lies-of-p']);
    expect(gameNames(w)).toEqual(['Elden Ring', 'Lies of P', 'Nioh', 'Sekiro']);
  });

  it('offers to copy the events of each game, and closes the modal without creating anything on Cancel', async () => {
    setup();
    const w = await openConfig();
    await w.find('.group-head .primary').trigger('click');
    await formDialogOpen();
    expect(formDialog().text()).toContain('Copy the events of');
    expect(formDialog().findAll('select option').map((o) => o.text())).toEqual(['Copy nothing', 'Elden Ring', 'Sekiro']);
    await formDialog().find('input').setValue('Nioh');
    await formDialog().findAll('button').find((b) => b.text().startsWith('Cancel')).trigger('click');
    await formDialogClosed();
    expect(invoke.mock.calls.some(([cmd]) => cmd === 'save_config')).toBe(false);
    expect(shownGame(w)).toBe('Elden Ring');
  });

  it('refuses to create a game with a name already taken', async () => {
    setup();
    const w = await openConfig();
    await w.find('.group-head .primary').trigger('click');
    await formDialogOpen();
    await formDialog().find('input[aria-label="Name of the new game"]').setValue(' sekiro ');
    expect(formDialog().text()).toContain('A game already has this name.');
    expect(formDialog().find('.primary').element.disabled).toBe(true);
    await expectTip(formDialog().find('.primary'), 'A game already has this name.');
    await formDialog().find('input[aria-label="Name of the new game"]').setValue(' ');
    await expectTip(formDialog().find('.primary'), 'Give the game a name');
  });

  it('asks for confirmation before removing a game, then saves right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const remove = () => w.find('.group > .field .danger');
    await remove().trigger('click');
    await w.find('.cfg-game .confirm .ghost').trigger('click');
    expect(w.find('.cfg-game .confirm').exists()).toBe(false);
    expect(gameNames(w)).toHaveLength(2);
    await remove().trigger('click');
    await w.find('.cfg-game .confirm .danger').trigger('click');
    await flushPromises();
    expect(gameNames(w)).toEqual(['Sekiro']);
    expect(invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config.games.map((g) => g.id)).toEqual(['sekiro']);
    expectUnchanged(w);
    expect(w.find('.cfg-game .confirm').exists()).toBe(false);
  });

  it('asks for the game again when the session\'s game is removed', async () => {
    setup({ get_session: emptySession({ gameId: 'er', sets: 1 }), save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    // The backend cancels the session of the removed game.
    responses.get_session = emptySession();
    await w.find('.group > .field .danger').trigger('click');
    await w.find('.cfg-game .confirm .danger').trigger('click');
    await flushPromises();
    await w.findAll('.tabs button')[0].trigger('click');
    const select = w.find('select');
    expect(select.element.value).toBe('');
    expect(select.element.disabled).toBe(false);
    expect(w.find('.events').exists()).toBe(false);
  });

  it('sets an exercise\'s range from its closed card and keeps at least one exercise', async () => {
    setup();
    const w = await openExercises({ card: false });
    expect(w.find('.cfg-exercise').classes()).not.toContain('open');
    expect(w.find('.cfg-exercise .cfg-range').text()).toContain('reps');
    await w.find('.cfg-exercise .cfg-event-head').trigger('click');
    const remove = w.find('.cfg-exercise .danger');
    expect(remove.element.disabled).toBe(true);
    await expectTip(remove, 'At least one exercise is needed');
    await w.find('input[aria-label="Maximum"]').setValue('');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises[0]).toMatchObject({ min: 10, max: 0 });
  });

  it('disables an exercise to keep it out of the draw without removing it', async () => {
    setup();
    const w = await openExercises({ card: false });
    const row = w.find('.cfg-exercise');
    expect(row.classes()).not.toContain('cfg-off');
    await w.find('input[aria-label="Include this exercise"]').setValue(false);
    expect(row.classes()).toContain('cfg-off');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises).toEqual([expect.objectContaining({ id: 'squat', enabled: false })]);
  });

  it('sets the global recovery and an exercise\'s own recovery', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const own = () => w.find('input[aria-label="Recovery"]');
    await w.find('.cfg-recovery input').setValue(45);
    expect(own().attributes('placeholder')).toBe('45');
    await own().setValue('90');
    expect(w.find('.cfg-exercise .cfg-event-head').text()).toContain('recovery 90 s');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent()).toMatchObject({ recovery: 45, exercises: [{ id: 'squat', recovery: 90 }] });
    await own().setValue('');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent().exercises[0]).not.toHaveProperty('recovery');
  });

  it('times an exercise in seconds or minutes, or no longer', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const timed = w.find('select[aria-label="Timer"]');
    expect(timed.findAll('option').map((o) => o.text())).toEqual(['Not timed', 'Seconds', 'Minutes']);
    await timed.setValue('minutes');
    expect(w.find('.cfg-timed-icon').attributes('aria-label')).toBe('Timed in minutes');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent().exercises[0].timed).toBe('minutes');
    timed.element.selectedIndex = 0;
    await timed.trigger('change');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent().exercises[0]).not.toHaveProperty('timed');
  });

  it('clears the per-session limit to remove it', async () => {
    setup();
    const w = await openConfig();
    await w.find('.cfg-params input[placeholder="none"]').setValue('');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.games[0].events[0]).not.toHaveProperty('maxPerSession');
  });

  it('resets unsaved changes', async () => {
    setup();
    const w = await openConfig();
    await rename(w, 'Other');
    await w.find('.cfg-actions .ghost').trigger('click');
    expect(shownGame(w)).toBe('Elden Ring');
    expectUnchanged(w);
  });

  it('shows the save error', async () => {
    setup({ save_config: rejection('exercise_range', { name: 'Squats', min: 20, max: 15 }) });
    const w = await openConfig();
    await rename(w, 'Other');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Invalid range for "Squats" (20 to 15).');
  });

  it('reports an invalid format while typing', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    const min = w.find('input[aria-label="Minimum"]');
    await min.trigger('focus');
    await min.setValue('abc');
    await vi.waitFor(() => expect(shownTip()).toBe('a whole number is needed, 0 or more'));
    await min.trigger('blur');
    await vi.waitFor(() => expect(shownTip()).toBeUndefined());
    await min.trigger('mouseenter');
    await vi.waitFor(() => expect(shownTip()).toBe('a whole number is needed, 0 or more'));
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
    await expectTip(save(), 'Squats › minimum: a whole number is needed, 0 or more.');
    await min.setValue('2');
    expect(save().attributes('disabled')).toBeUndefined();
  });

  describe('sharing', () => {
    const savedConfig = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;

    it('does not share a game without event', async () => {
      setup({ get_config: { config: { ...config, games: [{ ...config.games[0], events: [] }, config.games[1]] } } });
      const w = await openConfig();
      const share = gameButton(w, 'Share');
      expect(share.element.disabled).toBe(true);
      await expectTip(share, 'Add an event to share it');
    });

    it('copies the saved game and the exercises to the clipboard', async () => {
      const writeText = vi.fn();
      vi.stubGlobal('navigator', { clipboard: { writeText } });
      setup();
      const w = await openConfig();
      await rename(w, 'Unsaved');
      await gameButton(w, 'Share').trigger('click');
      await flushPromises();
      expect(writeText).toHaveBeenLastCalledWith(shareGame(config.games[0]));
      expect(w.find('.banner.ok').text()).toBe('Copied to the clipboard, ready to share.');
      await w.findAll('.subtabs button')[1].trigger('click');
      await w.find('.cfg-exercises .group-head .ghost').trigger('click');
      expect(writeText).toHaveBeenLastCalledWith(shareExercises(config.exercises));
      vi.unstubAllGlobals();
    });

    it('imports a shared game in a modal, replaces the one with the same name and saves it right away', async () => {
      setup({ save_config: (args) => ({ config: args.config }) });
      const w = await openConfig();
      const open = () => w.find('.group-head .ghost');
      await open().trigger('click');
      await formDialogOpen();
      const text = formDialog().find('textarea[aria-label="Shared text"]');
      expect(document.activeElement).toBe(text.element);
      const submit = () => formDialog().find('.cfg-import .primary');
      expect(submit().element.disabled).toBe(true);
      await text.setValue('not a game');
      expect(formDialog().find('.cfg-import .banner.error').text()).toBe('This text is not a valid shared game.');
      expect(text.attributes('aria-invalid')).toBe('true');
      expect(submit().element.disabled).toBe(true);
      await text.setValue(shareGame({ name: 'sekiro', events: [{ name: 'Death', draws: 3, jokers: 1 }] }));
      expect(formDialog().find('.cfg-import .banner.warn').text()).toBe('sekiro already exists: it will be replaced.');
      await formDialog().find('.cfg-import').trigger('submit');
      await flushPromises();
      await formDialogClosed();
      expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro']);
      expect(savedConfig().games[1]).toEqual({ id: 'sekiro', name: 'sekiro', events: [{ id: 'death', name: 'Death', draws: 1, jokers: 1 }] });
      expect(w.find('.banner.ok').text()).toBe('sekiro imported, limited to 1 exercise per event.');
      expect(shownGame(w)).toBe('sekiro');
      expectUnchanged(w);
      await open().trigger('click');
      await formDialogOpen();
      await formDialog().find('textarea').setValue(shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1 }] }));
      await formDialog().find('.cfg-import').trigger('submit');
      await flushPromises();
      await formDialogClosed();
      expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro', 'nioh']);
      expect(w.find('.banner.ok').text()).toBe('Nioh imported.');
    });

    it('imports shared exercises in a modal without losing pending changes', async () => {
      setup({ save_config: (args) => ({ config: args.config }) });
      const w = await openConfig();
      await rename(w, 'Elden Ring DLC');
      await w.findAll('.subtabs button')[1].trigger('click');
      await w.findAll('.cfg-exercises .group-head .ghost')[1].trigger('click');
      await formDialogOpen();
      expect(formDialog().find('h3').text()).toBe('Import exercises');
      expect(document.activeElement).toBe(formDialog().find('textarea').element);
      await formDialog().find('textarea').setValue(
        shareExercises([
          { name: 'squats', unit: 'reps', min: 12, max: 20 },
          { name: 'Push-ups', unit: 'reps', min: 8, max: 12 },
        ]),
      );
      await formDialog().find('.cfg-import').trigger('submit');
      await flushPromises();
      await formDialogClosed();
      expect(savedConfig().exercises).toEqual([
        { id: 'squat', name: 'Squats', enabled: true, unit: 'reps', min: 12, max: 20 },
        { id: 'push-ups', name: 'Push-ups', enabled: true, unit: 'reps', min: 8, max: 12 },
      ]);
      expect(savedConfig().games[0].name).toBe('Elden Ring');
      expect(w.find('.banner.ok').text()).toBe('Exercises imported: 1 added, 1 updated.');
      expect(w.findAll('.cfg-exercise')).toHaveLength(2);
      await w.find('.cfg-actions .primary').trigger('click');
      await flushPromises();
      expect(savedConfig().games[0].name).toBe('Elden Ring DLC');
      expect(savedConfig().exercises).toHaveLength(2);
    });
  });
});

describe('program and settings', () => {
  const remember = (w) => w.findAll('.check').find((c) => c.text() === 'Remember the intensity between launches').find('input');
  const sent = () => invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;

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
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent().rememberIntensity).toBe(true);
    expect(sent().games[0].name).toBe('Elden Ring');
    await w.findAll('.tabs button')[2].trigger('click');
    expect(shownGame(w)).toBe('Elden Ring DLC');
    expect(w.find('.cfg-actions .primary').element.disabled).toBe(false);
  });

  it('saves only the changes of the Program tab, the Settings ones stay pending', async () => {
    const w = await editBoth();
    await w.findAll('.tabs button')[2].trigger('click');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent().games[0].name).toBe('Elden Ring DLC');
    expect(sent().rememberIntensity).toBeFalsy();
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
    await check('Countdown before the timer (s)').setValue('5');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent()).toMatchObject({ sound: false, timerAutoDone: false, timerPrep: 5 });
  });

  it('resets only the changes of its own tab', async () => {
    const w = await editBoth();
    await w.find('.cfg-actions .ghost').trigger('click');
    expect(remember(w).element.checked).toBe(false);
    expect(w.find('.cfg-actions .primary').element.disabled).toBe(true);
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
  const select = (w) => w.find('.cfg-settings select');

  it('offers the system language, then English and French written in their own language', async () => {
    setup();
    const w = await mountSettings();
    expect(w.find('.cfg-settings .setting').text()).toContain('Language');
    expect(select(w).findAll('option').map((o) => [o.element.value, o.text()])).toEqual([
      ['system', 'System language'],
      ['en', 'English'],
      ['fr', 'Français'],
    ]);
    expect(select(w).element.value).toBe('system');
  });

  it('switches right away without saving nor touching the pending changes', async () => {
    setup();
    const w = await mountSettings();
    await w.findAll('.tabs button')[2].trigger('click');
    await rename(w, 'Elden Ring DLC');
    await openSettings(w);
    await select(w).setValue('fr');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('set_language', { setting: 'fr' });
    expect(invoke.mock.calls.some(([cmd]) => cmd === 'save_config')).toBe(false);
    expect(w.findAll('.tabs button').map((b) => b.text())).toEqual(['Jouer', 'Historique', 'Programme', 'Réglages', 'À propos']);
    expect(w.find('.cfg-settings .setting').text()).toContain('Langue');
    expect(select(w).find('option').text()).toBe('Langue du système');
    expect(document.documentElement.lang).toBe('fr');
    await w.findAll('.tabs button')[2].trigger('click');
    expect(shownGame(w)).toBe('Elden Ring DLC');
    expect(gameButton(w, 'Renommer').exists()).toBe(true);
    expect(w.find('.cfg-actions .primary').text()).toBe('Enregistrer');
  });

  it('shows the error and goes back to the saved setting when the backend refuses', async () => {
    setup({ set_language: rejection('invalid_language', { value: 'fr' }) });
    const w = await mountSettings();
    await select(w).setValue('fr');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Unknown language: fr.');
    expect(select(w).element.value).toBe('system');
    expect(w.findAll('.tabs button')[0].text()).toBe('Play');
  });
});

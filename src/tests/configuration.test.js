import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, rejection, responses, setup, openConfig, shownGame, gameButton, rename, expectUnchanged, shownTip, expectTip, layOutControls, formDialog, formDialogOpen, formDialogClosed, confirmDialog, confirmDialogOpen, confirmDialogClosed, editDialog, editDialogOpen, editDialogClosed } from './app.js';

// Games offered by the select of the Program tab.
const gameNames = (w) => w.findAll('#cfg-game option').map((o) => o.text());

describe('configuration', () => {
  layOutControls();
  // Exercises sub-tab, with the first exercise's dialog open unless asked otherwise.
  async function openExercises({ dialog = true } = {}) {
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    if (dialog) await openTile(w.find('.tile-slot .tile'));
    return w;
  }
  // Opens the edit dialog of an event or an exercise from its tile.
  async function openTile(tile) {
    await tile.trigger('click');
    await editDialogOpen();
  }
  async function closeDialog() {
    await editDialog().find('.edit-foot .primary').trigger('click');
    await editDialogClosed();
  }
  const save = (w) => w.find('.cfg-actions .primary');
  // Closes the dialog, saves, then opens the same tile again: the dialog hides the save bar.
  async function saveAndReopen(w, tile) {
    await closeDialog();
    await save(w).trigger('click');
    await flushPromises();
    await openTile(tile());
  }

  it('shows the game being edited in the select and switches games with it', async () => {
    setup();
    const w = await openConfig();
    expect(shownGame(w)).toBe('Elden Ring');
    await w.find('#cfg-game').setValue('sekiro');
    expect(shownGame(w)).toBe('Sekiro');
    expect(w.findAll('.cfg-events .tile-name').map((n) => n.text())).toEqual([config.games[1].events[0].name]);
  });

  it('edits the configuration with fields and applies it on save', async () => {
    setup();
    const w = await openConfig();
    expectUnchanged(w);
    await expectTip(save(w), 'No pending changes.');
    await openTile(w.find('.cfg-events .tile'));
    expect(document.activeElement).toBe(editDialog().find('.edit-foot .primary').element);
    await editDialog().find('input[aria-label="Name of the event"]').setValue('Brutal death');
    await closeDialog();
    expect(w.find('.cfg-events .tile-name').text()).toBe('Brutal death');
    vi.useFakeTimers();
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
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
    const done = () => w.find('.cfg-game-bar form .primary');
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

  it('adds an exercise with an id derived from the name, its dialog open with the name selected', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    expect(w.find('#cfg-game').exists()).toBe(false);
    for (let i = 0; i < 2; i++) {
      await openTile(w.find('.cfg-exercises .tile-new'));
      const name = editDialog().find('input[aria-label="Name of the exercise"]').element;
      expect(document.activeElement).toBe(name);
      expect([name.selectionStart, name.selectionEnd]).toEqual([0, name.value.length]);
      await closeDialog();
    }
    expect(w.findAll('.cfg-exercises .tile-name').map((n) => n.text())).toEqual(['Squats', 'New exercise', 'New exercise']);
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises.map((e) => e.id)).toEqual(['squat', 'new-exercise', 'new-exercise-2']);
  });

  it('opens an added event in its dialog with its name selected', async () => {
    setup();
    const w = await openConfig();
    await openTile(w.find('.cfg-events .tile-new'));
    const name = editDialog().find('input[aria-label="Name of the event"]').element;
    expect(name.value).toBe('New event');
    expect(document.activeElement).toBe(name);
    expect([name.selectionStart, name.selectionEnd]).toEqual([0, name.value.length]);
    await closeDialog();
    expect(w.findAll('.cfg-events .tile-name').map((n) => n.text())).toEqual(['Death', 'Boss defeated', 'New event']);
  });

  it('deletes an event from its dialog, waiting for save', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await openTile(w.findAll('.cfg-events .tile')[1]);
    await editDialog().find('.edit-foot .danger').trigger('click');
    await editDialogClosed();
    expect(w.findAll('.cfg-events .tile-name').map((n) => n.text())).toEqual(['Death']);
    await save(w).trigger('click');
    await flushPromises();
    expect(invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config.games[0].events.map((e) => e.id)).toEqual(['death']);
  });

  it('refuses an empty name: the dialog says so and save is disabled', async () => {
    setup();
    const w = await openConfig();
    await openTile(w.find('.cfg-events .tile'));
    const name = editDialog().find('input[aria-label="Name of the event"]');
    await name.setValue(' ');
    expect(name.attributes('aria-invalid')).toBe('true');
    expect(editDialog().find('.field-error').text()).toBe('Give the event a name');
    await closeDialog();
    expect(w.find('.cfg-events .tile-name').text()).toBe('(unnamed)');
    expect(save(w).element.disabled).toBe(true);
    await expectTip(save(w), 'Elden Ring › event name: it cannot be empty.');
  });

  it('creates an empty game or one with events copied from another game in a modal and saves it right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const savedGames = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config.games;
    async function create(name, source) {
      await w.find('.cfg-game-bar > .primary').trigger('click');
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
    await w.find('.cfg-game-bar > .primary').trigger('click');
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
    await w.find('.cfg-game-bar > .primary').trigger('click');
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
    const remove = () => w.find('.cfg-game-bar .danger');
    await remove().trigger('click');
    await confirmDialogOpen();
    expect(confirmDialog().text()).toContain('Delete Elden Ring and all its events?');
    await confirmDialog().find('.ghost').trigger('click');
    await confirmDialogClosed();
    expect(gameNames(w)).toHaveLength(2);
    await remove().trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.danger').trigger('click');
    await flushPromises();
    expect(gameNames(w)).toEqual(['Sekiro']);
    expect(invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config.games.map((g) => g.id)).toEqual(['sekiro']);
    expectUnchanged(w);
    await confirmDialogClosed();
  });

  it('asks for the game again when the session\'s game is removed', async () => {
    setup({ get_session: emptySession({ gameId: 'er', sets: 1 }), save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    // The backend cancels the session of the removed game.
    responses.get_session = emptySession();
    await w.find('.cfg-game-bar .danger').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.danger').trigger('click');
    await flushPromises();
    await w.findAll('.tabs button')[0].trigger('click');
    const select = w.find('select');
    expect(select.element.value).toBe('');
    expect(select.element.disabled).toBe(false);
    expect(w.find('.events').exists()).toBe(false);
  });

  it('shows an exercise\'s range on its tile, sets it in its dialog and keeps at least one exercise', async () => {
    setup();
    const w = await openExercises({ dialog: false });
    expect(w.find('.tile-slot .tile-foot').text()).toBe('10–15 reps');
    await openTile(w.find('.tile-slot .tile'));
    const remove = editDialog().find('.edit-foot .danger');
    expect(remove.element.disabled).toBe(true);
    await expectTip(remove, 'At least one exercise is needed');
    await editDialog().find('input[aria-label="Maximum"]').setValue('');
    await editDialog().findAll('button[aria-label="More"]')[0].trigger('click');
    expect(w.find('.tile-slot .tile-foot').text()).toBe('11–0 reps');
    await closeDialog();
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises[0]).toMatchObject({ min: 11, max: 0 });
  });

  it('disables an exercise from its tile or its dialog to keep it out of the draw without removing it', async () => {
    setup();
    const w = await openExercises({ dialog: false });
    const tile = w.find('.tile-slot');
    expect(tile.classes()).not.toContain('cfg-off');
    await w.find('input[aria-label="Include this exercise"]').setValue(false);
    expect(tile.classes()).toContain('cfg-off');
    await openTile(tile.find('.tile'));
    const included = editDialog().find('.edit-aside input');
    expect(included.element.checked).toBe(false);
    await included.setValue(true);
    expect(tile.classes()).not.toContain('cfg-off');
    await included.setValue(false);
    await closeDialog();
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises).toEqual([expect.objectContaining({ id: 'squat', enabled: false })]);
  });

  it('sets the global recovery and an exercise\'s own recovery', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const own = () => editDialog().find('#ex-recovery');
    await w.find('#cfg-recovery').setValue(45);
    expect(own().attributes('placeholder')).toBe('45');
    await own().setValue('90');
    expect(w.find('.tile-slot .tile').text()).toContain('recovery 90 s');
    const tile = () => w.find('.tile-slot .tile');
    await saveAndReopen(w, tile);
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent()).toMatchObject({ recovery: 45, exercises: [{ id: 'squat', recovery: 90 }] });
    await own().setValue('');
    await saveAndReopen(w, tile);
    expect(sent().exercises[0]).not.toHaveProperty('recovery');
    // Empty, + starts from the global recovery.
    await editDialog().findAll('button[aria-label="More"]').at(-1).trigger('click');
    expect(own().element.value).toBe('46');
  });

  it('times an exercise in seconds or minutes, or no longer', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const timer = (text) => editDialog().findAll('[role="radio"]').find((b) => b.text() === text);
    expect(editDialog().findAll('[role="radio"]').map((b) => b.text())).toEqual(['Not timed', 'Seconds', 'Minutes']);
    expect(timer('Not timed').attributes('aria-checked')).toBe('true');
    await timer('Minutes').trigger('click');
    expect(w.find('.timer-icon').attributes('aria-label')).toBe('Timed in minutes');
    const tile = () => w.find('.tile-slot .tile');
    await saveAndReopen(w, tile);
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent().exercises[0].timed).toBe('minutes');
    await timer('Not timed').trigger('click');
    expect(w.find('.timer-icon').exists()).toBe(false);
    await saveAndReopen(w, tile);
    expect(sent().exercises[0]).not.toHaveProperty('timed');
  });

  it('repeats a timed exercise for the other side, explained, until it is no longer timed', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const eachSide = () => editDialog().find('#ex-each-side');
    const timer = (text) => editDialog().findAll('[role="radio"]').find((b) => b.text() === text);
    expect(eachSide().exists()).toBe(false);
    await timer('Seconds').trigger('click');
    expect(editDialog().text()).toContain('Repeat for the other sideThe duration is for one side: the timer runs again for the other side.');
    await eachSide().setValue(true);
    expect(w.find('.timer-icon').attributes('aria-label')).toBe('Timed in seconds, on each side');
    const tile = () => w.find('.tile-slot .tile');
    await saveAndReopen(w, tile);
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent().exercises[0]).toMatchObject({ timed: 'seconds', eachSide: true });
    await timer('Not timed').trigger('click');
    expect(eachSide().exists()).toBe(false);
    await saveAndReopen(w, tile);
    expect(sent().exercises[0].eachSide).toBe(false);
  });

  it('clears the per-session limit to remove it, with the field or its - button', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config.games[0].events;
    await openTile(w.find('.cfg-events .tile'));
    expect(w.find('.cfg-events .tile').text()).toContain('15 times max per session');
    await editDialog().find('#ev-limit').setValue('');
    expect(w.find('.cfg-events .tile').text()).not.toContain('max per session');
    // Below 1, - removes the limit, here of the second event.
    await saveAndReopen(w, () => w.findAll('.cfg-events .tile')[1]);
    expect(sent()[0]).not.toHaveProperty('maxPerSession');
    const less = () => editDialog().findAll('button[aria-label="Less"]')[1];
    await editDialog().findAll('button[aria-label="More"]')[1].trigger('click');
    expect(editDialog().find('#ev-limit').element.value).toBe('1');
    await less().trigger('click');
    expect(editDialog().find('#ev-limit').element.value).toBe('');
    expect(less().element.disabled).toBe(true);
    await expectTip(less(), 'Already empty');
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
    const w = await openExercises();
    const min = editDialog().find('input[aria-label="Minimum"]');
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
    const w = await openExercises();
    const min = editDialog().find('input[aria-label="Minimum"]');
    await min.setValue('');
    await min.trigger('change');
    expect(min.element.value).toBe('0');
  });

  it('disables saving while a field is badly filled and gives the reason', async () => {
    setup();
    const w = await openExercises();
    const min = editDialog().find('input[aria-label="Minimum"]');
    await min.setValue('-2');
    expect(save(w).attributes('disabled')).toBeDefined();
    await expectTip(save(w), 'Squats › minimum: a whole number is needed, 0 or more.');
    await min.setValue('2');
    expect(save(w).attributes('disabled')).toBeUndefined();
  });
});

import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, rejection, responses, setup, openConfig, shownGame, gameButton, rename, expectUnchanged, shownTip, expectTip, layOutControls, formDialog, formDialogOpen, formDialogClosed, confirmDialog, confirmDialogOpen, confirmDialogClosed } from './app.js';

// Games offered by the select of the Program tab.
const gameNames = (w) => w.findAll('#cfg-game option').map((o) => o.text());

describe('configuration', () => {
  layOutControls();
  // Exercises sub-tab, with the first exercise's card opened unless asked otherwise.
  async function openExercises({ card = true } = {}) {
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    if (card) await w.find('.cfg-exercise .cfg-event-head').trigger('click');
    return w;
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

  it('opens and reveals an added event with its name selected', async () => {
    setup();
    const w = await openConfig();
    const scroll = vi.spyOn(Element.prototype, 'scrollIntoView');
    await w.find('.cfg-game > button').trigger('click');
    await flushPromises();
    const added = w.findAll('.cfg-event').at(-1);
    expect(added.element.open).toBe(true);
    expect(scroll.mock.contexts).toEqual([added.element]);
    const name = added.find('input[aria-label="Name of the event"]').element;
    expect(document.activeElement).toBe(name);
    expect([name.selectionStart, name.selectionEnd]).toEqual([0, name.value.length]);
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
    await w.find('.group > .field .danger').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.danger').trigger('click');
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

  it('repeats a timed exercise for the other side, explained, until it is no longer timed', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const eachSide = () => w.findAll('.cfg-exercise label.check').find((l) => l.text() === 'Repeat for the other side');
    expect(eachSide()).toBeUndefined();
    const timed = w.find('select[aria-label="Timer"]');
    await timed.setValue('seconds');
    expect(w.find('.cfg-exercise .hint').exists()).toBe(false);
    await eachSide().find('input').setValue(true);
    expect(w.find('.cfg-exercise .hint').text()).toBe('The duration is for one side: the timer runs again for the other side.');
    expect(w.find('.cfg-timed-icon').attributes('aria-label')).toBe('Timed in seconds, on each side');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent().exercises[0]).toMatchObject({ timed: 'seconds', eachSide: true });
    timed.element.selectedIndex = 0;
    await timed.trigger('change');
    expect(eachSide()).toBeUndefined();
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent().exercises[0].eachSide).toBe(false);
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
});

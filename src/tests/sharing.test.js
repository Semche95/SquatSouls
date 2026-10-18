import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { config, setup, mountApp, openConfig, openSettings, expectUnchanged, expectTip, layOutControls, editDialog, formDialog, formDialogOpen, formDialogClosed, save, discard, savedConfig } from './app.js';
import { shownGame, gameButton, importButton, rename, openExercises, openTile, closeDialog } from './program.js';
import { shareExercise, shareExercises, shareGame } from '../share.js';

describe('sharing', () => {
  layOutControls();

  it('does not share a game without event', async () => {
    setup({ get_config: { config: { ...config, games: [{ ...config.games[0], events: [] }, config.games[1]] } } });
    const w = await openConfig();
    const share = gameButton(w, 'Share');
    expect(share.element.disabled).toBe(true);
    await expectTip(share, 'Add an event before sharing this game');
  });

  it('copies the saved game and the exercises to the clipboard', async () => {
    const writeText = vi.fn();
    vi.stubGlobal('navigator', { clipboard: { writeText } });
    setup();
    const w = await openConfig();
    await gameButton(w, 'Share').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenLastCalledWith(shareGame(config.games[0]));
    expect(w.find('.banner.ok').text()).toBe('Copied to the clipboard, ready to share.');
    await w.findAll('.subtabs button')[1].trigger('click');
    await w.find('.cfg-exercises .group-head .ghost').trigger('click');
    expect(writeText).toHaveBeenLastCalledWith(shareExercises(config.exercises));
    vi.unstubAllGlobals();
  });

  it('copies the saved exercise from its dialog, not once it is edited or before it is saved', async () => {
    const writeText = vi.fn();
    vi.stubGlobal('navigator', { clipboard: { writeText } });
    setup();
    const w = await openExercises();
    const share = () => editDialog().findAll('.edit-foot button').find((b) => b.text() === 'Share');
    await share().trigger('click');
    expect(writeText).toHaveBeenLastCalledWith(shareExercise(config.exercises[0]));
    await editDialog().find('input[aria-label="Name of the exercise"]').setValue('Unsaved');
    expect(share().element.disabled).toBe(true);
    await expectTip(share(), 'Save or discard the changes first');
    await closeDialog();
    await discard(w).trigger('click');
    await openTile(w.find('.cfg-exercises .tile-new'));
    expect(share().element.disabled).toBe(true);
    await expectTip(share(), 'Save or discard the changes first');
    vi.unstubAllGlobals();
  });

  it('neither shares nor imports while Program has pending changes, until they are saved or discarded', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const exercisesHead = () => w.findAll('.cfg-exercises .group-head button');
    await rename(w, 'Elden Ring DLC');
    for (const button of [gameButton(w, 'Share'), importButton(w)]) {
      expect(button.element.disabled).toBe(true);
      await expectTip(button, 'Save or discard the changes first');
    }
    await w.findAll('.subtabs button')[1].trigger('click');
    for (const button of exercisesHead()) {
      expect(button.element.disabled).toBe(true);
      await expectTip(button, 'Save or discard the changes first');
    }
    await discard(w).trigger('click');
    expect(exercisesHead().map((b) => b.element.disabled)).toEqual([false, false]);
    await w.findAll('.subtabs button')[0].trigger('click');
    await rename(w, 'Elden Ring DLC');
    await save(w).trigger('click');
    await flushPromises();
    expect([gameButton(w, 'Share'), importButton(w)].map((b) => b.element.disabled)).toEqual([false, false]);
  });

  it('imports a shared game in a modal, replaces the one with the same name and saves it right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const open = () => importButton(w);
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
    await text.setValue(shareGame({ name: 'sekiro', events: [{ name: 'Death', draws: 101, jokers: 1 }] }));
    expect(formDialog().find('#import-game').text()).toBe('sekiro already exists.');
    expect(submit().text()).toBe('Import');
    await formDialog().find('.cfg-import').trigger('submit');
    await flushPromises();
    await formDialogClosed();
    expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro']);
    expect(savedConfig().games[1]).toEqual({ id: 'sekiro', name: 'Sekiro', events: [{ id: 'death', name: 'Death', draws: 100, jokers: 1 }] });
    expect(w.find('.banner.ok').text()).toBe('Sekiro imported, limited to 100 exercises per event.');
    expect(shownGame(w)).toBe('Sekiro');
    expectUnchanged(w);
    await open().trigger('click');
    await formDialogOpen();
    await formDialog().find('textarea').setValue(shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1 }] }));
    expect(formDialog().find('#import-game').text()).toBe('Nioh');
    await formDialog().find('.cfg-import').trigger('submit');
    await flushPromises();
    await formDialogClosed();
    expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro', 'nioh']);
    expect(w.find('.banner.ok').text()).toBe('Nioh imported.');
  });

  it('imports shared exercises in a modal without losing the pending changes of Settings', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await mountApp();
    await openSettings(w);
    await w.findAll('.check').find((c) => c.text() === 'Play sounds').find('input').setValue(false);
    await w.findAll('.tabs button')[2].trigger('click');
    expectUnchanged(w);
    await w.findAll('.subtabs button')[1].trigger('click');
    await w.findAll('.cfg-exercises .group-head button')[1].trigger('click');
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
    expect(savedConfig().sound).toBe(config.sound);
    expect(w.find('.banner.ok').text()).toBe('Exercises imported: 1 added, 1 updated.');
    expect(w.findAll('.cfg-exercises .tile-slot')).toHaveLength(2);
    expectUnchanged(w);
    await openSettings(w);
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().sound).toBe(false);
    expect(savedConfig().exercises).toHaveLength(2);
  });
});

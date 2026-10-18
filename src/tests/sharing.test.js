import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { config, setup, openConfig, expectUnchanged, expectTip, layOutControls, editDialog, formDialog, formDialogOpen, formDialogClosed, save, savedConfig } from './app.js';
import { shownGame, gameButton, rename, openExercises, openTile, closeDialog } from './program.js';
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

  it('copies the saved exercise from its dialog, not before it is saved', async () => {
    const writeText = vi.fn();
    vi.stubGlobal('navigator', { clipboard: { writeText } });
    setup();
    const w = await openExercises();
    const share = () => editDialog().findAll('.edit-foot button').find((b) => b.text() === 'Share');
    await editDialog().find('input[aria-label="Name of the exercise"]').setValue('Unsaved');
    await share().trigger('click');
    expect(writeText).toHaveBeenLastCalledWith(shareExercise(config.exercises[0]));
    await closeDialog();
    await openTile(w.find('.cfg-exercises .tile-new'));
    expect(share().element.disabled).toBe(true);
    await expectTip(share(), 'Save this exercise before sharing it');
    vi.unstubAllGlobals();
  });

  it('imports a shared game in a modal, replaces the one with the same name and saves it right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const open = () => w.findAll('.cfg-game-bar > button').find((b) => b.text() === 'Import');
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
    expect(formDialog().find('.cfg-import .banner.warn').text()).toBe('sekiro already exists: it will be replaced.');
    await formDialog().find('.cfg-import').trigger('submit');
    await flushPromises();
    await formDialogClosed();
    expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro']);
    expect(savedConfig().games[1]).toEqual({ id: 'sekiro', name: 'sekiro', events: [{ id: 'death', name: 'Death', draws: 100, jokers: 1 }] });
    expect(w.find('.banner.ok').text()).toBe('sekiro imported, limited to 100 exercises per event.');
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
    expect(savedConfig().games[0].name).toBe('Elden Ring');
    expect(w.find('.banner.ok').text()).toBe('Exercises imported: 1 added, 1 updated.');
    expect(w.findAll('.cfg-exercises .tile-slot')).toHaveLength(2);
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().games[0].name).toBe('Elden Ring DLC');
    expect(savedConfig().exercises).toHaveLength(2);
  });
});

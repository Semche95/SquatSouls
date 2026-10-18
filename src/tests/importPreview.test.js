import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { config, setup, openConfig, expectTip, layOutControls, formDialog, formDialogOpen, formDialogClosed, savedConfig } from './app.js';
import { shownGame, importButton } from './program.js';
import { shareExercises, shareGame } from '../share.js';

const saving = { save_config: (args) => ({ config: args.config }) };

async function openImport(w, button) {
  await button.trigger('click');
  await formDialogOpen();
}
const importGame = (w) => openImport(w, importButton(w));
async function importExercises(w) {
  await w.findAll('.subtabs button')[1].trigger('click');
  await openImport(w, w.findAll('.cfg-exercises .group-head button')[1]);
}
const paste = (text) => formDialog().find('textarea').setValue(text);
const submit = () => formDialog().find('.cfg-import .primary');
async function confirm() {
  await formDialog().find('.cfg-import').trigger('submit');
  await flushPromises();
  await formDialogClosed();
}

// Each group shown: its title, then each row with its box and its details.
const groups = () =>
  formDialog()
    .findAll('.import-group')
    .map((g) => [g.find('h4').text(), g.findAll('li').map((li) => [li.find('input').element.checked, li.find('.import-name').text(), li.findAll('.import-detail > span').map((s) => s.text())])]);
const row = (name) => formDialog().findAll('.import-group li').find((li) => li.find('.import-name').text() === name).find('input');
const groupAll = (title) => formDialog().findAll('.import-group').find((g) => g.find('h4').text().startsWith(title)).find('.import-all input');

const eldenRing = {
  name: 'Elden Ring',
  events: [
    { name: 'Death', draws: 1, maxPerSession: 15, jokers: 0 },
    { name: 'Boss defeated', draws: 1, jokers: 1 },
    { name: 'Grace', draws: 1, maxPerSession: 3 },
  ],
};

describe('import preview', () => {
  layOutControls();

  it('lists what a game import adds, changes or lacks, and applies only the chosen changes', async () => {
    setup(saving);
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame(eldenRing));
    expect(formDialog().find('#import-game').text()).toBe('Elden Ring already exists.');
    expect(groups()).toEqual([
      ['New 1', [[true, 'Grace', ['1 exercise per set', '3 times max per session']]]],
      ['Changed 1', [[true, 'Boss defeated', ['+2 jokers → +1 joker']]]],
    ]);
    expect(submit().text()).toBe('Import');
    await row('Boss defeated').setValue(false);
    await confirm();
    expect(savedConfig().games[0].events).toEqual([
      config.games[0].events[0],
      config.games[0].events[1],
      { id: 'grace', name: 'Grace', draws: 1, jokers: 0, maxPerSession: 3 },
    ]);
  });

  it('leaves the events missing from the import unchecked, and keeps those checked again', async () => {
    setup(saving);
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame({ name: 'Sekiro', events: [{ name: 'Death', draws: 1 }] }));
    expect(groups()).toEqual([
      ['New 1', [[true, 'Death', ['1 exercise per set']]]],
      ['Not in the import 1', [[false, 'Bonfire', ['1 exercise per set']]]],
    ]);
    expect(formDialog().find('.import-hint').text()).toBe('Checked events are kept.');
    await row('Bonfire').setValue(true);
    await confirm();
    expect(savedConfig().games[1].events.map((ev) => ev.name)).toEqual(['Death', 'Bonfire']);
  });

  it('imports a game whose name is taken as a new game, under a free name, with the chosen events', async () => {
    setup(saving);
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame(eldenRing));
    await formDialog().findAll('.import-mode button')[1].trigger('click');
    const name = formDialog().find('input[aria-label="Name of the imported game"]');
    expect(name.element.value).toBe('Elden Ring 2');
    expect(groups().map(([title, rows]) => [title, rows.map((r) => r[0])])).toEqual([['New 3', [true, true, true]]]);
    await name.setValue('sekiro ');
    expect(submit().element.disabled).toBe(true);
    await expectTip(submit(), 'A game already has this name.');
    await name.setValue('Elden Ring no hit');
    await row('Death').setValue(false);
    await confirm();
    expect(savedConfig().games.map((g) => g.name)).toEqual(['Elden Ring', 'Sekiro', 'Elden Ring no hit']);
    expect(savedConfig().games[2].events.map((ev) => ev.name)).toEqual(['Boss defeated', 'Grace']);
    expect(savedConfig().games[0]).toEqual(config.games[0]);
    expect(shownGame(w)).toBe('Elden Ring no hit');
    expect(w.find('.banner.ok').text()).toBe('Elden Ring no hit imported.');
  });

  it('keeps the switches as they are while the name of the new game is edited', async () => {
    setup(saving);
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame(eldenRing));
    await formDialog().findAll('.import-mode button')[1].trigger('click');
    await row('Death').setValue(false);
    await formDialog().find('input[aria-label="Name of the imported game"]').setValue('Elden Ring no hit');
    expect(row('Death').element.checked).toBe(false);
    await confirm();
    expect(savedConfig().games[2].events.map((ev) => ev.name)).toEqual(['Boss defeated', 'Grace']);
  });

  it('replaces the game chosen in place of a game whose name is free, keeping its name', async () => {
    setup(saving);
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame({ name: 'Sekiro no prosthetic', events: [{ name: 'Death', draws: 1 }] }));
    const mode = () => formDialog().findAll('.import-mode button');
    expect(mode().map((b) => b.attributes('aria-checked'))).toEqual(['false', 'true']);
    expect(formDialog().find('input[aria-label="Name of the imported game"]').element.value).toBe('Sekiro no prosthetic');
    await mode()[0].trigger('click');
    expect(formDialog().find('#import-game').text()).toBe('Sekiro no prosthetic');
    const target = formDialog().find('select[aria-label="Game to replace"]');
    expect(target.element.value).toBe('er');
    await target.setValue('sekiro');
    expect(groups()).toEqual([
      ['New 1', [[true, 'Death', ['1 exercise per set']]]],
      ['Not in the import 1', [[false, 'Bonfire', ['1 exercise per set']]]],
    ]);
    await confirm();
    expect(savedConfig().games.map((g) => g.name)).toEqual(['Elden Ring', 'Sekiro']);
    expect(savedConfig().games[1].events.map((ev) => ev.name)).toEqual(['Death']);
    expect(shownGame(w)).toBe('Sekiro');
    expect(w.find('.banner.ok').text()).toBe('Sekiro imported.');
  });

  it('disables replacing when there is no game', async () => {
    setup({ get_config: { config: { ...config, games: [] } } });
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame({ name: 'Nioh', events: [{ name: 'Death', draws: 1 }] }));
    const replace = formDialog().find('.import-mode button');
    expect(replace.element.disabled).toBe(true);
    await expectTip(replace, 'No game to replace.');
    expect(formDialog().find('#import-game').text()).toBe('New game: Nioh');
  });

  it('checks or unchecks a whole group at once', async () => {
    setup();
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame({ name: 'Nioh', events: eldenRing.events }));
    const all = groupAll('New');
    await all.setValue(false);
    expect(groups()[0][1].map((r) => r[0])).toEqual([false, false, false]);
    await expectTip(submit(), 'Check at least one change');
    await row('Grace').setValue(true);
    expect(all.element.checked).toBe(false);
    await all.setValue(true);
    expect(groups()[0][1].map((r) => r[0])).toEqual([true, true, true]);
  });

  it('shows only the changed fields of an exercise, and nothing to import when all are identical', async () => {
    setup(saving);
    const w = await openConfig();
    await importExercises(w);
    await paste(shareExercises([{ name: 'squats', unit: 'reps', min: 10, max: 15 }]));
    expect(formDialog().find('.import-preview').text()).toBe('Nothing to import: everything is already identical.');
    expect(submit().text()).toBe('Import');
    await expectTip(submit(), 'Nothing to import: everything is already identical.');
    await paste(
      shareExercises([
        { name: 'Squats', unit: 'reps', min: 12, max: 20, recovery: 30 },
        { name: 'Plank', unit: 's', min: 20, max: 40, timed: 'seconds', eachSide: true },
      ]),
    );
    expect(groups()).toEqual([
      ['New 1', [[true, 'Plank', ['20–40 s', 'Timed in seconds, on each side']]]],
      ['Changed 1', [[true, 'Squats', ['10–15 reps → 12–20 reps', 'general recovery → recovery 30 s']]]],
    ]);
    await row('Squats').setValue(false);
    await confirm();
    expect(savedConfig().exercises.map((ex) => [ex.name, ex.min])).toEqual([
      ['Squats', 10],
      ['Plank', 20],
    ]);
    expect(w.find('.banner.ok').text()).toBe('Exercises imported: 1 added, 0 updated.');
  });

  it('lists the longest imports in full', async () => {
    const events = (prefix) => Array.from({ length: 100 }, (_, i) => ({ id: `${prefix}-${i}`, name: `${prefix} ${i}`, draws: 1, jokers: 0 }));
    setup({ get_config: { config: { ...config, games: [{ id: 'er', name: 'Elden Ring', events: events('Old') }] } } });
    const w = await openConfig();
    await importGame(w);
    await paste(shareGame({ name: 'Elden Ring', events: events('New') }));
    expect(groups().map(([title, rows]) => [title, rows.length])).toEqual([
      ['New 100', 100],
      ['Not in the import 100', 100],
    ]);
    await groupAll('Not in the import').setValue(true);
    expect(groups()[1][1].every(([checked]) => checked)).toBe(true);
  });
});

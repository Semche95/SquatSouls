import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { deflateSync, strToU8 } from 'fflate';
import { config, setup, openConfig, expectTip, expectUnchanged, layOutControls, rejection, formDialog, formDialogOpen, formDialogClosed, savedConfig } from './app.js';
import { rename } from './program.js';
import { shareGame, toBase64 } from '../sharing/share.js';
import { wide } from '../ui/wide.js';

// Catalog of the latest release: a game of the program under its name, and two others, out of order.
const catalog = [
  shareGame({ name: 'Hollow Knight', steamAppId: 367520, events: [{ name: 'Death', draws: 1, jokers: 0 }] }),
  shareGame({ name: 'Elden Ring', steamAppId: 1245620, events: [...config.games[0].events, { name: 'Grace found', draws: 1, jokers: 0 }] }),
  shareGame({ name: 'Celeste', events: [{ name: 'Strawberry', draws: 1, jokers: 0 }, { name: 'Death', draws: 1, jokers: 0 }] }),
];

// The catalog's games, by name, with what is shown beside them.
const listed = () => formDialog().findAll('nav button').map((b) => [b.find('.cfg-game-name').text(), b.find('.cfg-game-count').text()]);
const listButton = (name) => formDialog().findAll('nav button').find((b) => b.find('.cfg-game-name').text() === name);
const dialogButton = (text) => formDialog().findAll('button').find((b) => b.text().startsWith(text));
const catalogCalls = () => invoke.mock.calls.filter(([cmd]) => cmd === 'game_catalog').length;
// Catalog button of the Games tab: beside Import in the narrow window, under it in the wide one.
const catalogButton = (w) => w.findAll('.cfg-game-bar button, .cfg-games button').find((b) => ['Catalog', 'Game catalog'].includes(b.text()));

async function openCatalog(over = {}) {
  setup({ game_catalog: catalog, save_config: (args) => ({ config: args.config }), ...over });
  const w = await openConfig();
  await catalogButton(w).trigger('click');
  await flushPromises();
  await formDialogOpen();
  return w;
}

describe('game catalog', () => {
  layOutControls();

  it('lists the catalog\'s games by name with their number of events, marking those already in the program', async () => {
    await openCatalog();
    expect(formDialog().find('h3').text()).toBe('Game catalog');
    expect(formDialog().find('textarea').exists()).toBe(false);
    expect(listed()).toEqual([
      ['Celeste', '2 events'],
      ['Elden Ring', 'Already added · 3 events'],
      ['Hollow Knight', '1 event'],
    ]);
    expect(document.activeElement).toBe(formDialog().find('.cfg-game-filter').element);
    await formDialog().find('.cfg-game-filter').setValue('ELDEN');
    expect(listed().map(([name]) => name)).toEqual(['Elden Ring']);
    await formDialog().find('.cfg-game-filter').setValue('Nioh');
    expect(formDialog().find('.cfg-game-none').text()).toBe('No game matches.');
  });

  it('opens from the wide window\'s list too, and fetches the catalog only once', async () => {
    wide.value = true;
    const w = await openCatalog();
    expect(catalogButton(w).text()).toBe('Game catalog');
    expect(listed()).toHaveLength(3);
    await dialogButton('Cancel').trigger('click');
    await formDialogClosed();
    await catalogButton(w).trigger('click');
    await formDialogOpen();
    expect(formDialog().find('.cfg-game-filter').element.value).toBe('');
    expect(listed()).toHaveLength(3);
    expect(catalogCalls()).toBe(1);
  });

  it('cannot be opened while Program has pending changes, in either window', async () => {
    for (const isWide of [false, true]) {
      wide.value = isWide;
      setup({ game_catalog: catalog });
      const w = await openConfig();
      await rename(w, 'Elden Ring DLC');
      expect(catalogButton(w).element.disabled).toBe(true);
      await expectTip(catalogButton(w), 'Save or discard the changes first');
      expect(catalogCalls()).toBe(0);
    }
  });

  it('says the catalog could not be fetched, and tries again', async () => {
    let fails = true;
    await openCatalog({ game_catalog: () => (fails ? rejection('catalog_unavailable')() : catalog) });
    expect(formDialog().find('.banner.error').text()).toBe('The game catalog could not be fetched. Check the connection, then try again.');
    fails = false;
    await dialogButton('Try again').trigger('click');
    await flushPromises();
    expect(formDialog().find('.banner.error').exists()).toBe(false);
    expect(listed()).toHaveLength(3);
  });

  it('asks for an update when no game of the catalog is readable by this version', async () => {
    const newer = `squatsouls/game/hades:${toBase64(deflateSync(strToU8(JSON.stringify({ version: 99, kind: 'game', name: 'Hades', events: [] }))))}`;
    await openCatalog({ game_catalog: [newer] });
    expect(formDialog().find('.banner.error').text()).toBe('This catalog needs a newer version of Squat Souls: update it to see its games.');
  });

  it('imports a game of the catalog with its Steam AppId, after going back to the list', async () => {
    const w = await openCatalog();
    await listButton('Hollow Knight').trigger('click');
    expect(formDialog().find('#import-game').text()).toBe('Hollow Knight');
    expect(formDialog().find('textarea').exists()).toBe(false);
    await dialogButton('← Back to the catalog').trigger('click');
    expect(listed()).toHaveLength(3);
    await listButton('Elden Ring').trigger('click');
    expect(formDialog().find('#import-game').text()).toBe('Elden Ring already exists.');
    expect(formDialog().findAll('.import-preview li')).toHaveLength(1);
    await formDialog().find('.cfg-import').trigger('submit');
    await flushPromises();
    await formDialogClosed();
    expect(savedConfig().games[0]).toEqual({ ...config.games[0], steamAppId: 1245620, events: [...config.games[0].events, { id: 'grace-found', name: 'Grace found', draws: 1, jokers: 0 }] });
    expect(w.find('.banner.ok').text()).toBe('Elden Ring imported.');
    expectUnchanged(w);
  });
});

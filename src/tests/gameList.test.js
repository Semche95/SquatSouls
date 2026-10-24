import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { config, setup, openConfig, expectTip, layOutControls, formDialog, formDialogOpen, formDialogClosed, savedConfig } from './app.js';
import { gameButton } from './program.js';
import { wide } from '../ui/wide.js';

// Games of the wide window's list, with their number of events, and the one being edited.
const listed = (w) => w.findAll('.cfg-game-list nav button').map((b) => [b.find('.cfg-game-name').text(), b.find('.cfg-game-count').text()]);
const active = (w) => w.find('.cfg-game-list nav button.active .cfg-game-name').text();
const listButton = (w, name) => w.findAll('.cfg-game-list nav button').find((b) => b.find('.cfg-game-name').text() === name);
const filter = (w) => w.find('.cfg-game-filter');
const title = (w) => w.find('.cfg-game-title').text();
const asideButton = (w, text) => w.findAll('.cfg-games > button, .cfg-games .tip button').find((b) => b.text() === text);
// Configuration with this many games, named from their number, the edited one ("Game 1") first in the list.
const numberedGames = (n, extra = []) => ({ ...config, games: [...Array.from({ length: n }, (_, i) => ({ id: `g${i + 1}`, name: `Game ${i + 1}`, events: [] })), ...extra] });

async function openWide(over = {}) {
  wide.value = true;
  setup(over);
  return openConfig();
}

describe('game list', () => {
  layOutControls();

  it('lists every game by name with its number of events beside the edited one, which a click changes', async () => {
    const w = await openWide();
    expect(w.find('#cfg-game').exists()).toBe(false);
    expect(listed(w)).toEqual([['Elden Ring', '2'], ['Sekiro', '1']]);
    expect(w.find('.cfg-game-count').attributes('aria-label')).toBe('2 events');
    expect([active(w), title(w)]).toEqual(['Elden Ring', 'Elden Ring']);
    await listButton(w, 'Sekiro').trigger('click');
    expect([active(w), title(w)]).toEqual(['Sekiro', 'Sekiro']);
    expect(w.findAll('.cfg-events .tile-name').map((n) => n.text())).toEqual(['Bonfire']);
  });

  it('leaves the renaming of a game when another one is chosen', async () => {
    const w = await openWide();
    await gameButton(w, 'Rename').trigger('click');
    expect(w.find('.cfg-game-head input').exists()).toBe(true);
    await listButton(w, 'Sekiro').trigger('click');
    expect(w.find('.cfg-game-head input').exists()).toBe(false);
    expect(title(w)).toBe('Sekiro');
  });

  it('filters the games by name, ignoring case, accents and surrounding spaces', async () => {
    const w = await openWide({ get_config: { config: numberedGames(10, [{ id: 'poke', name: 'Pokémon', events: [] }]) }, get_last_game: 'g1' });
    expect(listed(w)).toHaveLength(11);
    await filter(w).setValue(' POKEMON ');
    expect(listed(w)).toEqual([['Pokémon', '0']]);
    expect(title(w)).toBe('Game 1');
    await filter(w).setValue('zelda');
    expect(listed(w)).toEqual([]);
    expect(w.find('.cfg-game-none').text()).toBe('No game matches.');
  });

  it('clears the filter that hides the game just created, to show it chosen', async () => {
    const w = await openWide({ get_config: { config: numberedGames(2) }, get_last_game: 'g1', save_config: (args) => ({ config: args.config }) });
    await filter(w).setValue('Game 1');
    await asideButton(w, '+ New game').trigger('click');
    await formDialogOpen();
    await formDialog().find('input[aria-label="Name of the new game"]').setValue('Nioh');
    await formDialog().find('form').trigger('submit');
    await flushPromises();
    await formDialogClosed();
    expect(savedConfig().games.at(-1).name).toBe('Nioh');
    expect(filter(w).element.value).toBe('');
    expect([active(w), title(w)]).toEqual(['Nioh', 'Nioh']);
  });

  it('imports a game under the list, except while Program has pending changes', async () => {
    const w = await openWide();
    const importGame = () => asideButton(w, 'Import a game');
    await importGame().trigger('click');
    await formDialogOpen();
    expect(formDialog().find('textarea[aria-label="Shared text"]').exists()).toBe(true);
    await formDialog().find('.modal-close').trigger('click');
    await formDialogClosed();
    await gameButton(w, 'Rename').trigger('click');
    await w.find('.cfg-game-head input').setValue('Other');
    await w.find('.cfg-game-head form').trigger('submit');
    expect(importGame().element.disabled).toBe(true);
    await expectTip(importGame(), 'Save or discard the changes first');
  });

  it('offers only the catalog, creation and import without any game', async () => {
    const w = await openWide({ get_config: { config: { ...config, games: [] } } });
    expect(w.find('.cfg-game-list').exists()).toBe(false);
    expect(w.find('.config .empty').text()).toBe('No game yet. The catalog has ready-to-play games, with their events; a game can also be created or imported.');
    expect([asideButton(w, 'Game catalog'), asideButton(w, '+ New game'), asideButton(w, 'Import a game')].every(Boolean)).toBe(true);
  });
});

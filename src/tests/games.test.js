import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { config, emptySession, idleSession, responses, setup, openConfig, expectUnchanged, expectTip, layOutControls, formDialog, formDialogOpen, formDialogClosed, confirmDialog, confirmDialogOpen, confirmDialogClosed, save, savedConfig } from './app.js';
import { shownGame, gameButton, rename } from './program.js';

// Games offered by the select of the Program tab.
const gameNames = (w) => w.findAll('#cfg-game option').map((o) => o.text());

describe('games', () => {
  layOutControls();

  it('shows the game being edited in the select and switches games with it', async () => {
    setup();
    const w = await openConfig();
    expect(shownGame(w)).toBe('Elden Ring');
    await w.find('#cfg-game').setValue('sekiro');
    expect(shownGame(w)).toBe('Sekiro');
    expect(w.findAll('.cfg-events .tile-name').map((n) => n.text())).toEqual([config.games[1].events[0].name]);
  });

  it('filters the games of the select with a search field, which the select of Play does not have', async () => {
    setup();
    const w = await openConfig();
    await flushPromises();
    expect(w.find('#cfg-game').element.tomselect.control_input.placeholder).toBe('Filter the games');
    await w.findAll('.tabs button')[0].trigger('click');
    await flushPromises();
    expect(w.find('#play-game').element.tomselect.settings.plugins).toEqual([]);
  });

  it('renames the game in place of its title from the end of its name, the new name waiting for save', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await gameButton(w, 'Rename').trigger('click');
    await flushPromises();
    const input = w.find('input[aria-label="Name of the game"]');
    expect(document.activeElement).toBe(input.element);
    expect([input.element.selectionStart, input.element.selectionEnd]).toEqual([10, 10]);
    expect(w.find('.cfg-game-title').exists()).toBe(false);
    const done = () => w.find('.cfg-game-head form .primary');
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
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().games[0].name).toBe('Elden Ring DLC');
  });

  it('creates an empty game or one with events copied from another game in a modal and saves it right away', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const savedGames = () => savedConfig().games;
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
    expect(savedConfig()).toBeUndefined();
    expect(shownGame(w)).toBe('Elden Ring');
  });

  it('shows the new game form unchanged while the modal closes, the game being saved', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await w.find('.cfg-game-bar > .primary').trigger('click');
    await formDialogOpen();
    await formDialog().find('input[aria-label="Name of the new game"]').setValue('Nioh');
    await formDialog().find('form').trigger('submit');
    await flushPromises();
    expect(formDialog().find('form > p').exists()).toBe(false);
    await formDialogClosed();
    expect(savedConfig().games.map((g) => g.name)).toEqual(['Elden Ring', 'Sekiro', 'Nioh']);
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
    const remove = () => w.find('.cfg-game-head .danger');
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
    expect(savedConfig().games.map((g) => g.id)).toEqual(['sekiro']);
    expectUnchanged(w);
    await confirmDialogClosed();
  });

  it('asks for the game again when the session\'s game is removed, and tells why the session ended', async () => {
    setup({ get_session: emptySession({ sets: 1 }), save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    // The backend ends the session of the removed game, with its exercises done.
    responses.get_session = idleSession({ ended: { saved: true, cause: 'gameRemoved' } });
    await w.find('.cfg-game-head .danger').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.danger').trigger('click');
    await flushPromises();
    await w.findAll('.tabs button')[0].trigger('click');
    const select = w.find('select');
    expect(select.element.value).toBe('');
    expect(select.element.disabled).toBe(false);
    expect(w.find('.events').exists()).toBe(false);
    expect(w.find('.banner.info').text()).toBe('The session ended: its game was deleted.');
    expect(w.find('.session-actions .farewell').exists()).toBe(true);
  });

  it('removes the last game and says there is none, in Program as in Play, where the session cannot start', async () => {
    setup({ get_config: { config: { ...config, games: config.games.slice(0, 1) } }, get_session: idleSession(), save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await w.find('.cfg-game-head .danger').trigger('click');
    await confirmDialogOpen();
    await confirmDialog().find('.danger').trigger('click');
    await flushPromises();
    expect(savedConfig().games).toEqual([]);
    expect(w.find('#cfg-game').exists()).toBe(false);
    expect(w.find('.config .empty').text()).toBe('No game yet.');
    await w.findAll('.tabs button')[0].trigger('click');
    expect(w.find('#play-game').element.disabled).toBe(true);
    await expectTip(w.find('#play-game'), 'No game yet. Games are created and imported in Program.');
    expect(w.find('.play-head + .empty').text()).toBe('No game yet. Games are created and imported in Program.');
    const start = w.find('.session-actions .primary');
    expect(start.text()).toBe('Start the session');
    expect(start.element.disabled).toBe(true);
    await expectTip(start, 'No game with events');
    expect(w.find('.session-actions .danger').element.disabled).toBe(true);
    await w.find('.play-head + .empty .link').trigger('click');
    expect(w.find('.cfg-game-bar').exists()).toBe(true);
    await confirmDialogClosed();
  });
});

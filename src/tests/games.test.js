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
    expect(savedConfig().games.map((g) => g.id)).toEqual(['sekiro']);
    expectUnchanged(w);
    await confirmDialogClosed();
  });

  it('asks for the game again when the session\'s game is removed, and tells why the session ended', async () => {
    setup({ get_session: emptySession({ sets: 1 }), save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    // The backend ends the session of the removed game, with its exercises done.
    responses.get_session = idleSession({ ended: { saved: true, cause: 'gameRemoved' } });
    await w.find('.cfg-game-bar .danger').trigger('click');
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
});

import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, setup, mountApp, openSettings, layOutControls, formDialog, formDialogOpen, save, savedConfig } from './app.js';

// Configuration with detection enabled, Elden Ring carrying its Steam AppId.
const steamConfig = (extra = {}) => ({
  ...config,
  steamDetection: true,
  games: [{ ...config.games[0], steamAppId: 1245620 }, config.games[1]],
  ...extra,
});

function setupSteam({ cfg = steamConfig(), ...over } = {}) {
  setup({
    get_config: { config: cfg },
    get_last_game: 'sekiro',
    save_config: (args) => ({ config: args.config }),
    running_steam_app: 1245620,
    ...over,
  });
}

const banner = (w) => w.find('.banner.steam');

const status = (w) => w.find('.status').text();
// Settings checkbox, by its label.
const check = (w, label) => w.findAll('.check').find((c) => c.text() === label).find('input');
const notifications = () => invoke.mock.calls.filter(([cmd]) => cmd === 'notify').map(([, n]) => n);

describe('Steam game detection', () => {
  layOutControls();
  it('reads no process until the option is checked', async () => {
    setupSteam({ cfg: config });
    const w = await mountApp();
    expect(invoke.mock.calls.some(([cmd]) => cmd === 'running_steam_app')).toBe(false);
    expect(banner(w).exists()).toBe(false);
  });

  it('offers to select the running configured game', async () => {
    setupSteam();
    const w = await mountApp();
    expect(banner(w).text()).toContain('Elden Ring detected.');
    await banner(w).find('button').trigger('click');
    await flushPromises();
    expect(w.find('.play select').element.value).toBe('er');
    expect(banner(w).exists()).toBe(false);
    expect(savedConfig()).toBeUndefined();
  });

  it('offers each of the games carrying the detected AppId', async () => {
    const challenge = { ...config.games[0], id: 'er-nohit', name: 'Elden Ring no-hit', steamAppId: 1245620 };
    setupSteam({ cfg: steamConfig({ games: [...steamConfig().games, challenge] }) });
    const w = await mountApp();
    expect(banner(w).findAll('button').map((b) => b.text())).toEqual(['Elden Ring', 'Elden Ring no-hit']);
    await banner(w).findAll('button')[1].trigger('click');
    await flushPromises();
    expect(w.find('.play select').element.value).toBe('er-nohit');
    expect(banner(w).exists()).toBe(false);
  });

  it('shows the detected game in the settings, even when already selected', async () => {
    setupSteam({ get_last_game: 'er' });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
    await openSettings(w);
    expect(status(w)).toBe('Game detected: Elden Ring.');
  });

  it('shows in the settings that no Steam game is running', async () => {
    setupSteam({ running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    expect(status(w)).toBe('No Steam game detected.');
  });

  it('reports in the settings a Steam Store rate-limiting requests', async () => {
    setupSteam({
      running_steam_app: 367520,
      steam_game_name: () => {
        throw 'blocked';
      },
    });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
    await openSettings(w);
    expect(status(w)).toContain('the Steam Store is rate-limiting requests');
  });

  it('no longer reports the limit once the Steam Store is reachable again', async () => {
    vi.useFakeTimers();
    let blocked = true;
    setupSteam({
      running_steam_app: 367520,
      steam_game_name: () => {
        throw blocked ? 'blocked' : 'wait';
      },
    });
    const w = await mountApp();
    await openSettings(w);
    expect(status(w)).toContain('the Steam Store is rate-limiting requests');
    blocked = false;
    await vi.advanceTimersByTimeAsync(10000);
    vi.useRealTimers();
    expect(status(w)).toBe('Steam game detected, name not known yet.');
  });

  it('offers nothing when the session\'s game is already locked', async () => {
    setupSteam({ get_session: emptySession({ gameId: 'sekiro' }) });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('also offers the variants without AppId of a game carrying it, and gives them the AppId', async () => {
    const variant = { ...config.games[0], id: 'er-2', name: 'elden ring 2' };
    setupSteam({ cfg: steamConfig({ games: [...steamConfig().games, variant] }) });
    const w = await mountApp();
    expect(banner(w).findAll('button').map((b) => b.text())).toEqual(['Elden Ring', 'elden ring 2']);
    await banner(w).findAll('button')[1].trigger('click');
    await flushPromises();
    expect(w.find('.play select').element.value).toBe('er-2');
    expect(savedConfig().games.find((g) => g.id === 'er-2').steamAppId).toBe(1245620);
  });

  it('recognizes a game without AppId by its name and gives it the detected one', async () => {
    setupSteam({ cfg: { ...config, steamDetection: true }, steam_game_name: 'ELDEN RING' });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Elden Ring detected.');
    await banner(w).find('button').trigger('click');
    await flushPromises();
    expect(savedConfig().games[0].steamAppId).toBe(1245620);
  });

  it('offers nothing when the Steam Store does not answer', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: new Error('Steam Store unreachable') });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('adds an unknown game with its name and AppId', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Hollow Knight detected.');
    await banner(w).find('.primary').trigger('click');
    await formDialogOpen();
    expect(formDialog().find('input[aria-label="Name of the new game"]').element.value).toBe('Hollow Knight');
    await formDialog().find('form').trigger('submit');
    await flushPromises();
    expect(savedConfig().games.at(-1)).toMatchObject({ name: 'Hollow Knight', steamAppId: 367520 });
  });

  it('ignores an unknown game, lists it in the settings and can remove it from there', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    const w = await mountApp();
    await banner(w).find('.ghost').trigger('click');
    await flushPromises();
    expect(savedConfig().ignoredGames).toEqual([{ appId: 367520, name: 'Hollow Knight' }]);
    expect(banner(w).exists()).toBe(false);
    expect(invoke.mock.calls.filter(([cmd]) => cmd === 'steam_game_name')).toEqual([['steam_game_name', { appId: 367520, language: 'english' }]]);

    await openSettings(w);
    expect(status(w)).toBe('Game detected: Hollow Knight (ignored).');
    const folded = w.find('.ignored-games');
    expect(folded.element.open).toBe(false);
    expect(folded.find('summary').text()).toBe('Ignored games (1)');
    const ignored = () => w.findAll('.ignored li');
    expect(ignored().map((li) => li.find('.ignored-name').text())).toEqual(['Hollow Knight']);
    await ignored()[0].find('button').trigger('click');
    await flushPromises();
    expect(savedConfig().ignoredGames).toEqual([]);
    expect(ignored()).toHaveLength(0);
  });

  it('enables detection from the settings with Save', async () => {
    setupSteam({ cfg: config, running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    expect(w.find('.setting .small').exists()).toBe(false);
    await check(w, 'Detect the running Steam game').setValue(true);
    expect(savedConfig()).toBeUndefined();
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().steamDetection).toBe(true);
    expect(invoke.mock.calls.some(([cmd]) => cmd === 'running_steam_app')).toBe(true);
  });
});

describe('launched Steam game notification', () => {
  it('notifies a configured game only once, and again once relaunched', async () => {
    vi.useFakeTimers();
    let running = 1245620;
    setupSteam({ running_steam_app: () => running });
    await mountApp();
    await vi.advanceTimersByTimeAsync(10000);
    expect(notifications()).toEqual([{ name: 'Elden Ring', configured: true }]);
    running = null;
    await vi.advanceTimersByTimeAsync(10000);
    running = 1245620;
    await vi.advanceTimersByTimeAsync(10000);
    expect(notifications()).toHaveLength(2);
  });

  it('does not notify a game already selected, even once another one is selected', async () => {
    vi.useFakeTimers();
    setupSteam({ get_last_game: 'er' });
    const w = await mountApp();
    await w.find('.play select').setValue('sekiro');
    await vi.advanceTimersByTimeAsync(10000);
    expect(notifications()).toEqual([]);
  });

  it('notifies an unknown game once its name is known', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    await mountApp();
    expect(notifications()).toEqual([{ name: 'Hollow Knight', configured: false }]);
  });

  it('notifies neither an ignored game, nor a game with an unknown name, nor when notification is unchecked', async () => {
    setupSteam({ cfg: steamConfig({ ignoredGames: [{ appId: 367520, name: 'Hollow Knight' }] }), running_steam_app: 367520 });
    await mountApp();
    setupSteam({ running_steam_app: 1, steam_game_name: new Error('Steam Store unreachable') });
    await mountApp();
    setupSteam({ cfg: steamConfig({ steamNotify: false }) });
    await mountApp();
    expect(notifications()).toEqual([]);
  });

  it('shows the notification checkbox under the checked detection', async () => {
    setupSteam({ cfg: config, running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    const notify = () => w.findAll('.check').filter((c) => c.text() === 'Notify when a game is launched');
    expect(notify()).toHaveLength(0);
    await check(w, 'Detect the running Steam game').setValue(true);
    expect(notify()).toHaveLength(1);
  });
});

describe('notification area', () => {
  it('collapses its settings until the icon is checked, and saves them with Save', async () => {
    setupSteam({ cfg: config, running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    const hide = 'Closing the window hides it instead of quitting';
    expect(w.findAll('.check').some((c) => c.text() === hide)).toBe(false);
    await check(w, 'Show the icon in the notification area').setValue(true);
    await check(w, hide).setValue(true);
    await check(w, 'Start hidden').setValue(true);
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig()).toMatchObject({ tray: true, closeToTray: true, startHidden: true });
  });
});

import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, setup, mountApp, openSettings } from './app.js';

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
const lastSaved = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)?.[1].config;

describe('Steam game detection', () => {
  it('reads no process until the option is checked', async () => {
    setupSteam({ cfg: config });
    const w = await mountApp();
    expect(invoke.mock.calls.some(([cmd]) => cmd === 'running_steam_app')).toBe(false);
    expect(banner(w).exists()).toBe(false);
  });

  it('offers to select the running configured game', async () => {
    setupSteam();
    const w = await mountApp();
    expect(banner(w).text()).toContain('Elden Ring détecté.');
    await banner(w).find('button').trigger('click');
    await flushPromises();
    expect(w.find('.play select').element.value).toBe('er');
    expect(banner(w).exists()).toBe(false);
    expect(lastSaved()).toBeUndefined();
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
    expect(status(w)).toBe('Jeu détecté : Elden Ring.');
  });

  it('shows in the settings that no Steam game is running', async () => {
    setupSteam({ running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    expect(status(w)).toBe('Aucun jeu Steam détecté.');
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
    expect(status(w)).toContain('le Steam Store limite les requêtes');
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
    expect(status(w)).toContain('le Steam Store limite les requêtes');
    blocked = false;
    await vi.advanceTimersByTimeAsync(10000);
    vi.useRealTimers();
    expect(status(w)).toBe('Jeu Steam détecté, nom pas encore connu.');
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
    expect(lastSaved().games.find((g) => g.id === 'er-2').steamAppId).toBe(1245620);
  });

  it('recognizes a game without AppId by its name and gives it the detected one', async () => {
    setupSteam({ cfg: { ...config, steamDetection: true }, steam_game_name: 'ELDEN RING' });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Elden Ring détecté.');
    await banner(w).find('button').trigger('click');
    await flushPromises();
    expect(lastSaved().games[0].steamAppId).toBe(1245620);
  });

  it('offers nothing when the Steam Store does not answer', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: new Error('Steam Store unreachable') });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('adds an unknown game with its name and AppId', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Hollow Knight détecté.');
    await banner(w).find('.primary').trigger('click');
    expect(w.find('input[aria-label="Nom du nouveau jeu"]').element.value).toBe('Hollow Knight');
    await w.find('form.cfg-game').trigger('submit');
    await flushPromises();
    expect(lastSaved().games.at(-1)).toMatchObject({ name: 'Hollow Knight', steamAppId: 367520 });
  });

  it('ignores an unknown game, lists it in the settings and can remove or add it from there', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    const w = await mountApp();
    await banner(w).find('.ghost').trigger('click');
    await flushPromises();
    expect(lastSaved().ignoredGames).toEqual([{ appId: 367520, name: 'Hollow Knight' }]);
    expect(banner(w).exists()).toBe(false);
    expect(invoke.mock.calls.filter(([cmd]) => cmd === 'steam_game_name')).toEqual([['steam_game_name', { appId: 367520, language: 'french' }]]);

    await openSettings(w);
    expect(status(w)).toBe('Jeu détecté : Hollow Knight (ignoré).');
    const ignored = () => w.findAll('.ignored li');
    expect(ignored().map((li) => li.text())).toEqual(['Hollow Knight AjouterRetirer']);
    await ignored()[0].findAll('button')[1].trigger('click');
    await flushPromises();
    expect(lastSaved().ignoredGames).toEqual([]);
    expect(ignored()).toHaveLength(0);
  });

  it('adds an ignored game from the settings and removes it from the list', async () => {
    setupSteam({ cfg: steamConfig({ ignoredGames: [{ appId: 367520, name: 'Hollow Knight' }] }), running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    await w.find('.ignored button').trigger('click');
    await w.find('form.cfg-game').trigger('submit');
    await flushPromises();
    expect(lastSaved().games.at(-1)).toMatchObject({ name: 'Hollow Knight', steamAppId: 367520 });
    expect(lastSaved().ignoredGames).toEqual([]);
  });

  it('enables detection from the settings with Save', async () => {
    setupSteam({ cfg: config, running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    expect(w.find('.setting .small').exists()).toBe(false);
    await check(w, 'Détecter le jeu Steam en cours').setValue(true);
    expect(lastSaved()).toBeUndefined();
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(lastSaved().steamDetection).toBe(true);
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
    const notify = () => w.findAll('.check').filter((c) => c.text() === 'Notifier quand un jeu est lancé');
    expect(notify()).toHaveLength(0);
    await check(w, 'Détecter le jeu Steam en cours').setValue(true);
    expect(notify()).toHaveLength(1);
  });
});

describe('notification area', () => {
  it('collapses its settings until the icon is checked, and saves them with Save', async () => {
    setupSteam({ cfg: config, running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    const hide = 'Fermer la fenêtre la masque au lieu de quitter';
    expect(w.findAll('.check').some((c) => c.text() === hide)).toBe(false);
    await check(w, 'Afficher l\'icône dans la zone de notification').setValue(true);
    await check(w, hide).setValue(true);
    await check(w, 'Démarrer masqué').setValue(true);
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(lastSaved()).toMatchObject({ tray: true, closeToTray: true, startHidden: true });
  });
});

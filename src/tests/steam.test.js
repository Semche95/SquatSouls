import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, setup, mountApp } from './app.js';

// Configuration avec la détection activée, Elden Ring portant son AppId Steam.
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

async function openSettings(w) {
  await w.findAll('.tabs button')[2].trigger('click');
  await w.findAll('.subtabs button')[2].trigger('click');
}
const status = (w) => w.find('.setting .small').text();
// Case à cocher des réglages, d'après son libellé.
const check = (w, label) => w.findAll('.check').find((c) => c.text() === label).find('input');
const notifications = () => invoke.mock.calls.filter(([cmd]) => cmd === 'notify').map(([, n]) => n);
const lastSaved = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)?.[1].config;

describe('détection du jeu Steam', () => {
  it('ne lit aucun processus tant que l\'option n\'est pas cochée', async () => {
    setupSteam({ cfg: config });
    const w = await mountApp();
    expect(invoke.mock.calls.some(([cmd]) => cmd === 'running_steam_app')).toBe(false);
    expect(banner(w).exists()).toBe(false);
  });

  it('propose de sélectionner le jeu configuré qui tourne', async () => {
    setupSteam();
    const w = await mountApp();
    expect(banner(w).text()).toContain('Elden Ring détecté.');
    await banner(w).find('button').trigger('click');
    await flushPromises();
    expect(w.find('.play select').element.value).toBe('er');
    expect(banner(w).exists()).toBe(false);
    expect(lastSaved()).toBeUndefined();
  });

  it('propose chacun des jeux qui portent l\'AppId détecté', async () => {
    const challenge = { ...config.games[0], id: 'er-nohit', name: 'Elden Ring sans coup', steamAppId: 1245620 };
    setupSteam({ cfg: steamConfig({ games: [...steamConfig().games, challenge] }) });
    const w = await mountApp();
    expect(banner(w).findAll('button').map((b) => b.text())).toEqual(['Elden Ring', 'Elden Ring sans coup']);
    await banner(w).findAll('button')[1].trigger('click');
    await flushPromises();
    expect(w.find('.play select').element.value).toBe('er-nohit');
    expect(banner(w).exists()).toBe(false);
  });

  it('montre dans les réglages le jeu détecté, même déjà sélectionné', async () => {
    setupSteam({ get_last_game: 'er' });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
    await openSettings(w);
    expect(status(w)).toBe('Jeu détecté : Elden Ring.');
  });

  it('montre dans les réglages qu\'aucun jeu Steam ne tourne', async () => {
    setupSteam({ running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    expect(status(w)).toBe('Aucun jeu Steam détecté.');
  });

  it('signale dans les réglages un Steam Store qui limite les requêtes', async () => {
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

  it('ne signale plus la limite une fois le Steam Store de nouveau joignable', async () => {
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

  it('ne propose rien quand le jeu de la séance est déjà fixé', async () => {
    setupSteam({ get_session: emptySession({ gameId: 'sekiro' }) });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('propose aussi les variantes sans AppId d\'un jeu qui le porte, et leur donne l\'AppId', async () => {
    const variant = { ...config.games[0], id: 'er-2', name: 'elden ring 2' };
    setupSteam({ cfg: steamConfig({ games: [...steamConfig().games, variant] }) });
    const w = await mountApp();
    expect(banner(w).findAll('button').map((b) => b.text())).toEqual(['Elden Ring', 'elden ring 2']);
    await banner(w).findAll('button')[1].trigger('click');
    await flushPromises();
    expect(w.find('.play select').element.value).toBe('er-2');
    expect(lastSaved().games.find((g) => g.id === 'er-2').steamAppId).toBe(1245620);
  });

  it('reconnaît à son nom un jeu sans AppId et lui donne celui détecté', async () => {
    setupSteam({ cfg: { ...config, steamDetection: true }, steam_game_name: 'ELDEN RING' });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Elden Ring détecté.');
    await banner(w).find('button').trigger('click');
    await flushPromises();
    expect(lastSaved().games[0].steamAppId).toBe(1245620);
  });

  it('ne propose rien quand le Steam Store ne répond pas', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: new Error('Steam Store injoignable') });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('ajoute un jeu inconnu avec son nom et son AppId', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Hollow Knight détecté.');
    await banner(w).find('.primary').trigger('click');
    expect(w.find('input[aria-label="Nom du nouveau jeu"]').element.value).toBe('Hollow Knight');
    await w.find('form.cfg-game').trigger('submit');
    await flushPromises();
    expect(lastSaved().games.at(-1)).toMatchObject({ name: 'Hollow Knight', steamAppId: 367520 });
  });

  it('ignore un jeu inconnu, le liste dans les réglages et peut l\'en retirer ou l\'ajouter', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    const w = await mountApp();
    await banner(w).find('.ghost').trigger('click');
    await flushPromises();
    expect(lastSaved().ignoredGames).toEqual([{ appId: 367520, name: 'Hollow Knight' }]);
    expect(banner(w).exists()).toBe(false);
    expect(invoke.mock.calls.filter(([cmd]) => cmd === 'steam_game_name')).toHaveLength(1);

    await openSettings(w);
    expect(status(w)).toBe('Jeu détecté : Hollow Knight (ignoré).');
    const ignored = () => w.findAll('.ignored li');
    expect(ignored().map((li) => li.text())).toEqual(['Hollow Knight AjouterRetirer']);
    await ignored()[0].findAll('button')[1].trigger('click');
    await flushPromises();
    expect(lastSaved().ignoredGames).toEqual([]);
    expect(ignored()).toHaveLength(0);
  });

  it('ajoute un jeu ignoré depuis les réglages et le retire de la liste', async () => {
    setupSteam({ cfg: steamConfig({ ignoredGames: [{ appId: 367520, name: 'Hollow Knight' }] }), running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    await w.find('.ignored button').trigger('click');
    await w.find('form.cfg-game').trigger('submit');
    await flushPromises();
    expect(lastSaved().games.at(-1)).toMatchObject({ name: 'Hollow Knight', steamAppId: 367520 });
    expect(lastSaved().ignoredGames).toEqual([]);
  });

  it('active la détection depuis les réglages avec Enregistrer', async () => {
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

describe('notification d\'un jeu Steam lancé', () => {
  it('notifie une seule fois un jeu configuré, et de nouveau une fois relancé', async () => {
    vi.useFakeTimers();
    let running = 1245620;
    setupSteam({ running_steam_app: () => running });
    await mountApp();
    await vi.advanceTimersByTimeAsync(10000);
    expect(notifications()).toEqual([{ title: 'Elden Ring lancé', body: 'À sélectionner dans Jouer pour commencer la séance.' }]);
    running = null;
    await vi.advanceTimersByTimeAsync(10000);
    running = 1245620;
    await vi.advanceTimersByTimeAsync(10000);
    expect(notifications()).toHaveLength(2);
  });

  it('notifie un jeu inconnu une fois son nom connu', async () => {
    setupSteam({ running_steam_app: 367520, steam_game_name: 'Hollow Knight' });
    await mountApp();
    expect(notifications()).toEqual([
      { title: 'Hollow Knight lancé', body: 'Jeu pas encore configuré : l\'ajouter depuis Jouer.' },
    ]);
  });

  it('ne notifie ni un jeu ignoré, ni un jeu au nom inconnu, ni quand la notification est décochée', async () => {
    setupSteam({ cfg: steamConfig({ ignoredGames: [{ appId: 367520, name: 'Hollow Knight' }] }), running_steam_app: 367520 });
    await mountApp();
    setupSteam({ running_steam_app: 1, steam_game_name: new Error('Steam Store injoignable') });
    await mountApp();
    setupSteam({ cfg: steamConfig({ steamNotify: false }) });
    await mountApp();
    expect(notifications()).toEqual([]);
  });

  it('montre la case de notification sous la détection cochée', async () => {
    setupSteam({ cfg: config, running_steam_app: null });
    const w = await mountApp();
    await openSettings(w);
    const notify = () => w.findAll('.check').filter((c) => c.text() === 'Notifier quand un jeu est lancé');
    expect(notify()).toHaveLength(0);
    await check(w, 'Détecter le jeu Steam en cours').setValue(true);
    expect(notify()).toHaveLength(1);
  });
});

describe('zone de notification', () => {
  it('replie ses réglages tant que l\'icône n\'est pas cochée, et les enregistre avec Enregistrer', async () => {
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

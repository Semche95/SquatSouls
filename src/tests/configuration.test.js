import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, responses, setup, mountApp } from './app.js';
import { shareExercises, shareGame } from '../share.js';

describe('configuration', () => {
  async function openConfig() {
    const w = await mountApp();
    await w.findAll('.tabs button')[2].trigger('click');
    return w;
  }

  it('affiche dans la liste le jeu en cours d\'édition', async () => {
    setup();
    const w = await openConfig();
    expect(w.find('.cfg-game-list .active').text()).toBe('Elden Ring');
  });

  it('édite la configuration avec des champs et l\'applique dès l\'enregistrement', async () => {
    vi.useFakeTimers();
    setup();
    const w = await openConfig();
    const save = () => w.find('.cfg-actions .primary');
    expect(save().exists()).toBe(false);
    await w.find('input[aria-label="Nom de l\'événement"]').setValue('Mort brutale');
    responses.save_config = (args) => ({ config: args.config });
    await save().trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.games[0].events[0]).toMatchObject({ id: 'mort', name: 'Mort brutale' });
    expect(w.find('.banner.ok').text()).toBe('Configuration enregistrée.');
    await vi.runAllTimersAsync();
    expect(w.find('.banner.ok').exists()).toBe(false);
    await w.findAll('.tabs button')[0].trigger('click');
    expect(w.find('.event').text()).toContain('Mort brutale');
  });

  it('ne renomme le jeu dans la liste qu\'à l\'enregistrement', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await w.find('input[aria-label="Nom du jeu"]').setValue('Elden Ring DLC');
    expect(w.find('.cfg-game-list .active').text()).toBe('Elden Ring');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.cfg-game-list .active').text()).toBe('Elden Ring DLC');
  });

  it('ajoute un exercice avec un id dérivé du nom', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    expect(w.find('.cfg-game-list').exists()).toBe(false);
    const add = w.find('.cfg-exercises > button');
    await add.trigger('click');
    await add.trigger('click');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises.map((e) => e.id)).toEqual(['squat', 'nouvel-exercice', 'nouvel-exercice-2']);
  });

  it('crée un jeu vide ou avec les événements copiés d\'un autre jeu et l\'enregistre aussitôt', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const savedGames = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config.games;
    async function create(name, source) {
      await w.find('.group-head .primary').trigger('click');
      await flushPromises();
      expect(w.find('form.cfg-game .primary').element.disabled).toBe(true);
      const nameInput = w.find('input[aria-label="Nom du nouveau jeu"]');
      expect(document.activeElement).toBe(nameInput.element);
      await nameInput.setValue(name);
      await w.find('select[aria-label="Événements du nouveau jeu"]').setValue(source);
      await w.find('form.cfg-game').trigger('submit');
      await flushPromises();
      expect(w.find('.cfg-game-list .active').text()).toBe(name);
      expect(w.find('.cfg-actions').exists()).toBe(false);
      return savedGames().at(-1);
    }
    expect(await create('Nioh', 'sekiro')).toMatchObject({ id: 'nioh', events: config.games[1].events });
    expect(await create('Lies of P', '')).toMatchObject({ id: 'lies-of-p', events: [{ name: 'Nouvel événement', draws: 1 }] });
    expect(savedGames().map((g) => g.id)).toEqual(['er', 'sekiro', 'nioh', 'lies-of-p']);
    expect(w.findAll('.cfg-game-list button').map((b) => b.text())).toEqual(['Elden Ring', 'Lies of P', 'Nioh', 'Sekiro']);
  });

  it('refuse de créer un jeu au nom déjà pris', async () => {
    setup();
    const w = await openConfig();
    await w.find('.group-head .primary').trigger('click');
    await flushPromises();
    await w.find('input[aria-label="Nom du nouveau jeu"]').setValue(' sekiro ');
    expect(w.find('form.cfg-game').text()).toContain('Un jeu porte déjà ce nom.');
    expect(w.find('form.cfg-game .primary').element.disabled).toBe(true);
  });

  it('demande confirmation avant de supprimer un jeu, puis l\'enregistre aussitôt', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    const remove = () => w.find('.cfg-game .field .danger');
    await remove().trigger('click');
    await w.find('.cfg-game .confirm .ghost').trigger('click');
    expect(w.find('.cfg-game .confirm').exists()).toBe(false);
    expect(w.findAll('.cfg-game-list button')).toHaveLength(2);
    await remove().trigger('click');
    await w.find('.cfg-game .confirm .danger').trigger('click');
    await flushPromises();
    expect(w.findAll('.cfg-game-list button').map((b) => b.text())).toEqual(['Sekiro']);
    expect(invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config.games.map((g) => g.id)).toEqual(['sekiro']);
    expect(w.find('.cfg-actions').exists()).toBe(false);
    expect(w.find('.cfg-game .confirm').exists()).toBe(false);
  });

  it('redemande le jeu quand celui de la séance est supprimé', async () => {
    setup({ get_session: emptySession({ gameId: 'er', sets: 1 }), save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    // Le backend annule la séance du jeu supprimé.
    responses.get_session = emptySession();
    await w.find('.cfg-game .field .danger').trigger('click');
    await w.find('.cfg-game .confirm .danger').trigger('click');
    await flushPromises();
    await w.findAll('.tabs button')[0].trigger('click');
    const select = w.find('select');
    expect(select.element.value).toBe('');
    expect(select.element.disabled).toBe(false);
    expect(w.find('.events').exists()).toBe(false);
  });

  it('règle l\'intervalle d\'un exercice et garde au moins un exercice', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    expect(w.find('.cfg-exercises .icon').element.disabled).toBe(true);
    await w.find('input[aria-label="Maximum"]').setValue('');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.exercises[0]).toMatchObject({ min: 10, max: 0 });
  });

  it('règle la pause globale et la pause propre d\'un exercice', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    const own = () => w.find('input[aria-label="Pause"]');
    await w.find('.cfg-pause input').setValue(45);
    expect(own().attributes('placeholder')).toBe('45');
    await own().setValue('90');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;
    expect(sent()).toMatchObject({ pause: 45, exercises: [{ id: 'squat', pause: 90 }] });
    await own().setValue('');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(sent().exercises[0]).not.toHaveProperty('pause');
  });

  it('vide la limite par séance pour la retirer', async () => {
    setup();
    const w = await openConfig();
    await w.find('.cfg-params input[placeholder="aucune"]').setValue('');
    responses.save_config = (args) => ({ config: args.config });
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === 'save_config')[1].config;
    expect(sent.games[0].events[0]).not.toHaveProperty('maxPerSession');
  });

  it('réinitialise les modifications non enregistrées', async () => {
    setup();
    const w = await openConfig();
    const name = () => w.find('input[aria-label="Nom du jeu"]');
    await name().setValue('Autre');
    await w.find('.cfg-actions .ghost').trigger('click');
    expect(name().element.value).toBe('Elden Ring');
    expect(w.find('.cfg-actions').exists()).toBe(false);
  });

  it('affiche l\'erreur d\'enregistrement', async () => {
    setup({ save_config: new Error('champ manquant') });
    const w = await openConfig();
    await w.find('input[aria-label="Nom du jeu"]').setValue('Autre');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toContain('champ manquant');
  });

  it('refuse un nombre négatif ou décimal avec un message clair', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    await w.find('input[aria-label="Minimum"]').setValue('-2');
    await w.find('.cfg-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Squats › minimum : il faut un nombre entier, 0 ou plus.');
    expect(invoke).not.toHaveBeenCalledWith('save_config', expect.anything());
  });

  describe('partage', () => {
    const savedConfig = () => invoke.mock.calls.filter(([cmd]) => cmd === 'save_config').at(-1)[1].config;

    it('copie le jeu enregistré et les exercices dans le presse-papiers', async () => {
      const writeText = vi.fn();
      vi.stubGlobal('navigator', { clipboard: { writeText } });
      setup();
      const w = await openConfig();
      await w.find('input[aria-label="Nom du jeu"]').setValue('Non enregistré');
      await w.find('.cfg-game .field .ghost').trigger('click');
      await flushPromises();
      expect(writeText).toHaveBeenLastCalledWith(shareGame(config.games[0]));
      expect(w.find('.banner.ok').text()).toBe('Copié dans le presse-papiers, prêt à partager.');
      await w.findAll('.subtabs button')[1].trigger('click');
      await w.find('.cfg-exercises .group-head .ghost').trigger('click');
      expect(writeText).toHaveBeenLastCalledWith(shareExercises(config.exercises));
      vi.unstubAllGlobals();
    });

    it('importe un jeu partagé, remplace celui du même nom et l\'enregistre aussitôt', async () => {
      setup({ save_config: (args) => ({ config: args.config }) });
      const w = await openConfig();
      const open = () => w.find('.group-head .ghost');
      await open().trigger('click');
      await flushPromises();
      const text = w.find('textarea[aria-label="Texte partagé"]');
      expect(document.activeElement).toBe(text.element);
      const submit = () => w.find('.cfg-import .primary');
      expect(submit().element.disabled).toBe(true);
      await text.setValue('pas un jeu');
      expect(w.find('.cfg-import .banner.error').text()).toBe('Ce texte n\'est pas un jeu partagé valide.');
      expect(text.attributes('aria-invalid')).toBe('true');
      expect(submit().element.disabled).toBe(true);
      await text.setValue(shareGame({ name: 'sekiro', events: [{ name: 'Mort', draws: 3, jokers: 1 }] }));
      expect(w.find('.cfg-import .banner.warn').text()).toBe('sekiro existe déjà : il sera remplacé.');
      await w.find('.cfg-import').trigger('submit');
      await flushPromises();
      expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro']);
      expect(savedConfig().games[1]).toEqual({ id: 'sekiro', name: 'sekiro', events: [{ id: 'mort', name: 'Mort', draws: 1, jokers: 1 }] });
      expect(w.find('.banner.ok').text()).toBe('sekiro importé, tirages limités à 1 exercice.');
      expect(w.find('.cfg-game-list .active').text()).toBe('sekiro');
      expect(w.find('.cfg-actions').exists()).toBe(false);
      await open().trigger('click');
      await w.find('textarea').setValue(shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 1 }] }));
      await w.find('.cfg-import').trigger('submit');
      await flushPromises();
      expect(savedConfig().games.map((g) => g.id)).toEqual(['er', 'sekiro', 'nioh']);
      expect(w.find('.banner.ok').text()).toBe('Nioh importé.');
    });

    it('importe des exercices partagés sans perdre les modifications en cours', async () => {
      setup({ save_config: (args) => ({ config: args.config }) });
      const w = await openConfig();
      await w.find('input[aria-label="Nom du jeu"]').setValue('Elden Ring DLC');
      await w.findAll('.subtabs button')[1].trigger('click');
      await w.findAll('.cfg-exercises .group-head .ghost')[1].trigger('click');
      await w.find('textarea').setValue(
        shareExercises([
          { name: 'squats', unit: 'reps', min: 12, max: 20 },
          { name: 'Pompes', unit: 'rép.', min: 8, max: 12 },
        ]),
      );
      await w.find('.cfg-import').trigger('submit');
      await flushPromises();
      expect(savedConfig().exercises).toEqual([
        { id: 'squat', name: 'Squats', unit: 'reps', min: 12, max: 20 },
        { id: 'pompes', name: 'Pompes', unit: 'rép.', min: 8, max: 12 },
      ]);
      expect(savedConfig().games[0].name).toBe('Elden Ring');
      expect(w.find('.banner.ok').text()).toBe('Exercices importés : 1 ajouté, 1 mis à jour.');
      expect(w.find('.cfg-import').exists()).toBe(false);
      expect(w.findAll('input[aria-label="Nom de l\'exercice"]')).toHaveLength(2);
      await w.find('.cfg-actions .primary').trigger('click');
      await flushPromises();
      expect(savedConfig().games[0].name).toBe('Elden Ring DLC');
      expect(savedConfig().exercises).toHaveLength(2);
    });
  });
});

import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { config, emptySession, draw, responses, setup, mountApp, press, expectTip } from './app.js';

describe('séance', () => {
  it('tire une série au clic sur un événement', async () => {
    setup({ draw: { id: 7, capped: false } });
    const w = await mountApp();
    responses.get_session = emptySession({ pending: [draw], triggers: [{ gameId: 'er', eventId: 'mort', count: 1 }] });
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('draw', { gameId: 'er', eventId: 'mort' });
    expect(w.find('.card.current').text()).toContain('Squats');
    expect(w.find('.event .count').text()).toBe('1 / 15');
  });

  it('prévient quand la limite par séance est atteinte', async () => {
    setup({ draw: { capped: true, eventName: 'Mort', maxPerSession: 15 } });
    const w = await mountApp();
    await w.find('.event').trigger('click');
    await flushPromises();
    expect(w.find('.banner.info').text()).toBe('Mort : limite de 15 atteinte pour la séance, pas de série.');
  });

  it('note fait chaque exercice d\'une série', async () => {
    setup({
      get_session: emptySession({ pending: [draw], jokers: 1 }),
      complete_exercise: emptySession(),
    });
    const w = await mountApp();
    expect(w.findAll('.card-actions button').map((b) => b.text())).toEqual(['Annuler Retour arr.']);
    await w.find('.item-actions .primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('complete_exercise', { drawId: 7, exerciseId: 'squat' });
    expect(w.find('.card').exists()).toBe(false);
  });

  it('barre les exercices faits et vise le premier restant', async () => {
    const pompes = { exerciseId: 'pompes', name: 'Pompes', amount: 8, unit: 'reps', halved: false, done: false, halveCost: 1, skipCost: 2 };
    const started = { ...draw, items: [{ ...draw.items[0], done: true }, pompes] };
    setup({ get_session: emptySession({ pending: [started], jokers: 4 }) });
    const w = await mountApp();
    const [done, left] = w.findAll('.items li');
    expect(done.classes()).toContain('done');
    expect(done.text()).toContain('fait');
    expect(done.find('button').exists()).toBe(false);
    expect(left.findAll('button').map((b) => b.text())).toEqual(['Fait Entrée', 'Moitié H', 'Sauter J']);
    expect(w.text()).not.toContain('Sauter la série');
  });

  it('annule d\'un clic une série pas commencée', async () => {
    setup({ get_session: emptySession({ pending: [draw] }), dismiss: emptySession() });
    const w = await mountApp();
    await w.find('.card-actions .ghost').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('dismiss', { drawId: 7 });
  });

  it('demande confirmation avant d\'annuler une série commencée', async () => {
    const started = { ...draw, items: [{ ...draw.items[0], done: true }, { ...draw.items[0], exerciseId: 'pompes', name: 'Pompes' }] };
    setup({ get_session: emptySession({ pending: [started] }), dismiss: emptySession() });
    const w = await mountApp();
    await press('Backspace');
    expect(w.find('.card .banner.confirm').text()).toContain('ne seront pas comptés');
    await press('Escape');
    expect(w.find('.card .banner.confirm').exists()).toBe(false);
    await w.find('.card-actions .ghost').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('dismiss', expect.anything());
    await w.find('.card .banner.confirm .danger').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('dismiss', { drawId: 7 });
    expect(w.find('.card').exists()).toBe(false);
  });

  describe('jokers', () => {
    const second = { exerciseId: 'pompes', name: 'Pompes', amount: 8, unit: 'reps', halved: false, halveCost: 1, skipCost: 2 };
    const both = { ...draw, items: [...draw.items, second] };
    const jokerButtons = (w) => w.findAll('.item-actions .ghost');

    it('sont cachés sans joker', async () => {
      setup({ get_session: emptySession({ pending: [both] }) });
      const w = await mountApp();
      expect(jokerButtons(w)).toHaveLength(0);
      expect(w.text()).not.toContain('Sauter la série');
    });

    it('réduisent ou sautent un exercice selon les jokers disponibles', async () => {
      setup({ get_session: emptySession({ pending: [both], jokers: 1 }), halve: emptySession() });
      const w = await mountApp();
      const [half, skip] = jokerButtons(w);
      expect([half.text(), skip.text()]).toEqual(['Moitié H', 'Sauter J']);
      expect(half.attributes('disabled')).toBeUndefined();
      expect(skip.attributes('disabled')).toBeDefined();
      expect(w.find('.card-actions .ghost').text()).toBe('Sauter la série (4 jokers)');
      expect(w.find('.card-actions .ghost').attributes('disabled')).toBeDefined();
      await half.trigger('click');
      expect(invoke).toHaveBeenCalledWith('halve', { drawId: 7, exerciseId: 'squat' });
    });

    it('sautent toute la série avec assez de jokers', async () => {
      setup({ get_session: emptySession({ pending: [both], jokers: 4 }), skip_set: emptySession() });
      const w = await mountApp();
      await w.find('.card-actions .ghost').trigger('click');
      expect(invoke).toHaveBeenCalledWith('skip_set', { drawId: 7 });
    });

    it('ne proposent plus la moitié d\'un exercice déjà réduit', async () => {
      const halved = { ...draw, items: [{ ...draw.items[0], amount: 5, halved: true, halveCost: null, skipCost: 1 }] };
      setup({ get_session: emptySession({ pending: [halved], jokers: 1 }) });
      const w = await mountApp();
      expect(jokerButtons(w).map((b) => b.text())).toEqual(['Sauter J']);
      expect(w.find('.card .what').text()).toContain('moitié');
    });
  });

  describe('pause après un exercice fait', () => {
    const both = { ...draw, items: [...draw.items, { exerciseId: 'pompes', name: 'Pompes', amount: 8, unit: 'reps' }] };
    const next = { ...draw, id: 8 };
    const faits = (w) => w.findAll('.item-actions .primary');

    // Squats (premier) suivent la pause globale, Pompes (second) ont leur pause propre.
    async function completeOne(index = 1, pause = config.pause, own = 90) {
      vi.useFakeTimers();
      setup({
        get_config: { config: { ...config, pause, exercises: [...config.exercises, { id: 'pompes', name: 'Pompes', unit: 'reps', min: 8, max: 12, pause: own }] } },
        get_session: emptySession({ pending: [both, next] }),
        complete_exercise: emptySession({ pending: [both, next] }),
      });
      const w = await mountApp();
      await faits(w)[index].trigger('click');
      await flushPromises();
      return w;
    }

    it('prend la pause propre de l\'exercice et bloque tout exercice jusqu\'à sa fin', async () => {
      const w = await completeOne();
      expect(w.find('.pause-left').text()).toBe('1:30');
      expect(faits(w).every((b) => b.element.disabled)).toBe(true);
      await press('Enter');
      expect(invoke.mock.calls.filter(([cmd]) => cmd === 'complete_exercise')).toHaveLength(1);
      await vi.advanceTimersByTimeAsync(30_000);
      expect(w.find('.pause-left').text()).toBe('1:00');
      await vi.advanceTimersByTimeAsync(60_000);
      expect(w.find('.banner.pause').exists()).toBe(false);
      expect(faits(w).some((b) => b.element.disabled)).toBe(false);
    });

    it('prend la pause globale sans pause propre', async () => {
      const w = await completeOne(0);
      expect(w.find('.pause-left').text()).toBe('1:00');
    });

    it('se passe avec le bouton ou Échap', async () => {
      const w = await completeOne();
      await w.find('.banner.pause button').trigger('click');
      expect(w.find('.banner.pause').exists()).toBe(false);
      await faits(w)[1].trigger('click');
      await flushPromises();
      expect(w.find('.banner.pause').exists()).toBe(true);
      await press('Escape');
      expect(w.find('.banner.pause').exists()).toBe(false);
    });

    it('n\'en fait pas quand toutes les pauses valent 0', async () => {
      const w = await completeOne(1, 0, 0);
      expect(w.find('.banner.pause').exists()).toBe(false);
    });
  });

  it('recharge la séance si une action échoue', async () => {
    setup({ get_session: emptySession({ pending: [draw] }), complete_exercise: new Error('série inconnue') });
    const w = await mountApp();
    responses.get_session = emptySession();
    await w.find('.item-actions .primary').trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toContain('série inconnue');
    expect(w.find('.card').exists()).toBe(false);
  });

  it('verrouille le jeu pendant la séance', async () => {
    setup({ get_session: emptySession({ gameId: 'sekiro' }) });
    const w = await mountApp();
    const select = w.find('select');
    expect(select.element.disabled).toBe(true);
    expect(select.element.value).toBe('sekiro');
  });

  it('ne propose de terminer la séance qu\'avec des séries faites et aucune en attente', async () => {
    setup({ get_session: emptySession() });
    let w = await mountApp();
    expect(w.find('.session-actions .primary').element.disabled).toBe(true);
    await expectTip(w.find('.session-actions .primary'), 'Aucune série faite');
    setup({ get_session: emptySession({ sets: 1, pending: [draw] }) });
    w = await mountApp();
    expect(w.find('.session-actions .primary').element.disabled).toBe(true);
    await expectTip(w.find('.session-actions .primary'), 'Des séries sont encore à faire');
  });

  it('demande confirmation avant de terminer la séance, puis enchaîne sur une nouvelle', async () => {
    setup({ get_session: emptySession({ sets: 1 }), new_session: emptySession() });
    const w = await mountApp();
    await w.find('.session-actions .primary').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('new_session', undefined);
    expect(w.text()).toContain('Terminer la séance ?');
    await w.find('.confirm .ghost').trigger('click');
    expect(w.find('.confirm').exists()).toBe(false);
    await w.find('.session-actions .primary').trigger('click');
    await w.find('.confirm .primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('new_session', undefined);
    expect(w.find('.confirm').exists()).toBe(false);
    expect(w.find('.event').element.disabled).toBe(false);
  });

  it('désactive l\'annulation tant que la séance est vide', async () => {
    setup();
    const w = await mountApp();
    expect(w.find('.session-actions .danger').element.disabled).toBe(true);
  });

  it('demande confirmation avant d\'annuler la séance', async () => {
    setup({ get_session: emptySession({ gameId: 'er' }), cancel_session: emptySession() });
    const w = await mountApp();
    await w.find('.session-actions .danger').trigger('click');
    expect(w.text()).toContain('Annuler la séance ?');
    await w.find('.confirm .ghost').trigger('click');
    expect(w.find('.confirm').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('cancel_session', undefined);
    await w.find('.session-actions .danger').trigger('click');
    await w.find('.confirm .danger').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('cancel_session', undefined);
    expect(invoke).not.toHaveBeenCalledWith('new_session', undefined);
  });
});

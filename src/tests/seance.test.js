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

  it('valide, annule ou saute une série', async () => {
    setup({
      get_session: emptySession({ pending: [draw], jokers: 1 }),
      complete: emptySession(),
      dismiss: emptySession(),
    });
    const w = await mountApp();
    const buttons = w.findAll('.card-actions button').map((b) => b.text());
    expect(buttons).toEqual(['Fait Entrée', 'Annuler Retour arr.']);
    await w.find('.card-actions .primary').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('complete', { drawId: 7 });
    expect(w.find('.card').exists()).toBe(false);
  });

  describe('jokers', () => {
    const second = { exerciseId: 'pompes', name: 'Pompes', amount: 8, unit: 'reps', halved: false, halveCost: 1, skipCost: 2 };
    const both = { ...draw, items: [...draw.items, second] };
    const jokerButtons = (w) => w.findAll('.item-jokers button');

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

  describe('pause après une série validée', () => {
    const both = { ...draw, items: [...draw.items, { exerciseId: 'pompes', name: 'Pompes', amount: 8, unit: 'reps' }] };
    const next = { ...draw, id: 8 };

    // Squats suivent la pause globale, Pompes ont leur pause propre.
    async function completeFirst(pause = config.pause, own = 90) {
      vi.useFakeTimers();
      setup({
        get_config: { config: { ...config, pause, exercises: [...config.exercises, { id: 'pompes', name: 'Pompes', unit: 'reps', min: 8, max: 12, pause: own }] } },
        get_session: emptySession({ pending: [both, next] }),
        complete: emptySession({ pending: [next] }),
      });
      const w = await mountApp();
      await w.find('.card-actions .primary').trigger('click');
      await flushPromises();
      return w;
    }

    it('prend la plus longue pause des exercices et bloque la série suivante jusqu\'à sa fin', async () => {
      const w = await completeFirst();
      const fait = () => w.find('.card-actions .primary').element;
      expect(w.find('.pause-left').text()).toBe('1:30');
      expect(fait().disabled).toBe(true);
      await press('Enter');
      expect(invoke.mock.calls.filter(([cmd]) => cmd === 'complete')).toHaveLength(1);
      await vi.advanceTimersByTimeAsync(30_000);
      expect(w.find('.pause-left').text()).toBe('1:00');
      await vi.advanceTimersByTimeAsync(60_000);
      expect(w.find('.banner.pause').exists()).toBe(false);
      expect(fait().disabled).toBe(false);
    });

    it('se passe avec le bouton ou Échap', async () => {
      const w = await completeFirst();
      await w.find('.banner.pause button').trigger('click');
      expect(w.find('.banner.pause').exists()).toBe(false);
      responses.complete = emptySession({ pending: [next] });
      await w.find('.card-actions .primary').trigger('click');
      await flushPromises();
      expect(w.find('.banner.pause').exists()).toBe(true);
      await press('Escape');
      expect(w.find('.banner.pause').exists()).toBe(false);
    });

    it('n\'en fait pas quand toutes les pauses valent 0', async () => {
      const w = await completeFirst(0, 0);
      expect(w.find('.banner.pause').exists()).toBe(false);
    });
  });

  it('recharge la séance si une action échoue', async () => {
    setup({ get_session: emptySession({ pending: [draw] }), complete: new Error('série inconnue') });
    const w = await mountApp();
    responses.get_session = emptySession();
    await w.find('.card-actions .primary').trigger('click');
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

import { afterEach, beforeEach, expect, vi } from 'vitest';
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { createVfm } from 'vue-final-modal';
import App from '../App.vue';
import { i18n } from '../i18n.js';

export const config = {
  recovery: 60,
  steamDetection: false,
  steamNotify: true,
  tray: false,
  closeToTray: false,
  startHidden: false,
  sound: true,
  timerAutoDone: true,
  timerPrep: 3,
  updateCheck: true,
  updateAuto: true,
  ignoredGames: [],
  exercises: [{ id: 'squat', name: 'Squats', enabled: true, unit: 'reps', min: 10, max: 15 }],
  games: [
    {
      id: 'er',
      name: 'Elden Ring',
      events: [
        { id: 'death', name: 'Death', draws: 1, jokers: 0, maxPerSession: 15 },
        { id: 'boss', name: 'Boss defeated', draws: 1, jokers: 2 },
      ],
    },
    { id: 'sekiro', name: 'Sekiro', events: [{ id: 'bonfire', name: 'Bonfire', draws: 1, jokers: 0 }] },
  ],
};

// Session started, without any set.
export function emptySession(extra = {}) {
  return { startedAt: 1, pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, jokersSpent: 0, ...extra };
}

export const draw = { id: 7, eventName: 'Death', gameName: 'Elden Ring', jokers: 0, items: [{ exerciseId: 'squat', name: 'Squats', amount: 10, unit: 'reps', halved: false, halveCost: 1, skipCost: 2 }] };

// Responses of the simulated backend, adjustable per test.
export let responses;
// Mounted components, unmounted after each test to remove their keyboard listener.
let mounted = [];

export function setup(over = {}) {
  responses = {
    get_config: { config },
    get_session: emptySession(),
    get_history: [],
    count_active_days: 0,
    get_last_game: 'er',
    set_last_game: undefined,
    get_onboarding_seen: true,
    set_onboarding_seen: undefined,
    get_language: { setting: 'system', language: 'en' },
    set_language: ({ setting }) => ({ setting, language: setting === 'fr' ? 'fr' : 'en' }),
    set_native_texts: undefined,
    ...over,
  };
  invoke.mockImplementation(async (cmd, args) => {
    const r = responses[cmd];
    if (r instanceof Error) throw r;
    return typeof r === 'function' ? r(args) : r;
  });
}

// Backend refusal: error with its code and parameters.
export const rejection =
  (code, params = {}) =>
  () => {
    throw { code, params };
  };

// Real transitions: a modal is only removed at the end of its closing one.
export async function mountApp() {
  const wrapper = mount(App, { attachTo: document.body, global: { plugins: [i18n, createVfm()], stubs: { transition: false } } });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

// Settings tab, last of the main tabs.
export async function openSettings(w) {
  await w.findAll('.tabs button')[3].trigger('click');
}

// Program tab, third of the main tabs.
export async function openConfig() {
  const w = await mountApp();
  await w.findAll('.tabs button')[2].trigger('click');
  return w;
}

// Save and reset buttons of the bar shared by the Program and Settings tabs.
export const save = (w) => w.find('.cfg-actions .primary');
export const discard = (w) => w.find('.cfg-actions .ghost');
// Configuration sent by the last save, undefined without any save.
export const savedConfig = () => invoke.mock.calls.findLast(([cmd]) => cmd === 'save_config')?.[1].config;

// Without pending changes, the save and reset buttons stay shown but disabled.
export function expectUnchanged(w) {
  expect(w.findAll('.cfg-actions button').map((b) => b.element.disabled)).toEqual([true, true]);
  expect(w.find('.unsaved').exists()).toBe(false);
}

// Simulates a backend event, received by the last listener mounted.
export function emitEvent(name, payload) {
  listen.mock.calls.findLast(([n]) => n === name)[1]({ payload });
  return flushPromises();
}

export function press(code, init = {}) {
  window.dispatchEvent(new KeyboardEvent('keydown', { code, bubbles: true, cancelable: true, ...init }));
  return flushPromises();
}

beforeEach(() => {
  invoke.mockReset();
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

// Text of the shown tooltip, or undefined without a tooltip.
export const shownTip = () => document.querySelector('.v-popper__popper--shown .v-popper__inner')?.textContent.trim();

// jsdom lays nothing out: without this, a modal's focus trap finds no visible control.
export function layOutControls() {
  beforeEach(() => {
    const spy = vi.spyOn(HTMLElement.prototype, 'getClientRects').mockReturnValue([{}]);
    return () => spy.mockRestore();
  });
}

// Modal teleported to the body outside the app's wrapper, found by its content class.
// Open once its focus trap holds the focus: it activates on the next frame, then focuses after a timer,
// and closing the modal before would leave it active. Closed at the end of its closing transition.
async function dialogOpen(selector) {
  await vi.waitFor(() => expect(document.querySelector(selector)?.contains(document.activeElement)).toBe(true));
  await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve)));
}
const dialogClosed = (selector) => vi.waitFor(() => expect(document.querySelector(selector)).toBeNull());

// Modal creating or importing a game, or importing exercises.
export const formDialog = () => new DOMWrapper(document.querySelector('.form-dialog'));
export const formDialogOpen = () => dialogOpen('.form-dialog');
export const formDialogClosed = () => dialogClosed('.form-dialog');

// Edit dialog of an event or an exercise.
export const editDialog = () => new DOMWrapper(document.querySelector('.edit-dialog'));
export const editDialogOpen = () => dialogOpen('.edit-dialog');
export const editDialogClosed = () => dialogClosed('.edit-dialog');

// Confirmation modal (ConfirmDialog): removing a game, ending or cancelling the session, quitting.
export const confirmDialog = () => new DOMWrapper(document.querySelector('.confirm-modal-content'));
export const confirmDialogOpen = () => dialogOpen('.confirm-modal-content');
export const confirmDialogClosed = () => dialogClosed('.confirm-modal-content');

// Hovers a control's tooltip container (DisabledTip), waits for the expected text, then leaves it.
export async function expectTip(wrapper, text) {
  const tip = wrapper.element.closest('.tip');
  tip.dispatchEvent(new Event('mouseenter'));
  await vi.waitFor(() => expect(shownTip()).toBe(text));
  tip.dispatchEvent(new Event('mouseleave'));
  await vi.waitFor(() => expect(shownTip()).toBeUndefined());
}

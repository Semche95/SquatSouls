import { afterEach, beforeEach, expect, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import App from '../App.vue';
import { i18n } from '../i18n.js';

export const config = {
  pause: 60,
  steamDetection: false,
  steamNotify: true,
  tray: false,
  closeToTray: false,
  startHidden: false,
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
    get_language: { setting: 'system', language: 'fr' },
    set_language: ({ setting }) => ({ setting, language: setting === 'en' ? 'en' : 'fr' }),
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

export async function mountApp() {
  const wrapper = mount(App, { attachTo: document.body, global: { plugins: [i18n] } });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

// Settings tab, last of the main tabs.
export async function openSettings(w) {
  await w.findAll('.tabs button')[3].trigger('click');
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

// Hovers a control's tooltip container (DisabledTip), waits for the expected text, then leaves it.
export async function expectTip(wrapper, text) {
  const tip = wrapper.element.closest('.tip');
  tip.dispatchEvent(new Event('mouseenter'));
  await vi.waitFor(() => expect(shownTip()).toBe(text));
  tip.dispatchEvent(new Event('mouseleave'));
  await vi.waitFor(() => expect(shownTip()).toBeUndefined());
}

import { afterEach, beforeEach, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { invoke } from "@tauri-apps/api/core";
import App from "../App.vue";

export const config = {
  exercises: [{ id: "squat", name: "Squats", unit: "reps", min: 10, max: 15 }],
  games: [
    {
      id: "er",
      name: "Elden Ring",
      events: [
        { id: "mort", name: "Mort", draws: 1, maxPerSession: 15 },
        { id: "boss", name: "Boss vaincu", draws: 1, jokers: 2 },
      ],
    },
    { id: "ds", name: "Dark Souls", events: [{ id: "feu", name: "Feu de camp", draws: 1 }] },
  ],
};

export function emptySession(extra = {}) {
  return { pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, skipped: [], ...extra };
}

export const draw = { id: 7, eventName: "Mort", gameName: "Elden Ring", jokers: 0, items: [{ exerciseId: "squat", name: "Squats", amount: 10, unit: "reps" }] };

// Réponses du backend simulé, modifiables par test.
export let responses;
// Composants montés, démontés après chaque test pour retirer leur écouteur clavier.
let mounted = [];

export function setup(over = {}) {
  responses = {
    get_config: { config },
    get_session: emptySession(),
    get_history: [],
    get_last_game: "er",
    set_last_game: undefined,
    ...over,
  };
  invoke.mockImplementation(async (cmd, args) => {
    const r = responses[cmd];
    if (r instanceof Error) throw r;
    return typeof r === "function" ? r(args) : r;
  });
}

export async function mountApp() {
  const wrapper = mount(App, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

export function press(code, init = {}) {
  window.dispatchEvent(new KeyboardEvent("keydown", { code, bubbles: true, cancelable: true, ...init }));
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

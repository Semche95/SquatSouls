import { beforeEach, expect, vi } from 'vitest';
import { languageSetting, setLanguage } from '../i18n.js';

// The Tauri backend does not exist under Vitest: each test simulates its responses.
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(async () => () => {}) }));
vi.mock('@tauri-apps/api/app', () => ({ getVersion: vi.fn(async () => '0.1.0') }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn(async () => {}) }));
vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(async () => {}) }));

// English interface following the system by default: the tests check the English texts.
beforeEach(() => {
  languageSetting.value = 'system';
  return setLanguage('en');
});

// A vue-i18n warning (missing key, scope not found...) fails the test, which would only print it.
beforeEach(() => {
  const spy = vi.spyOn(console, 'warn');
  return () => {
    const warnings = spy.mock.calls.map(([message]) => String(message)).filter((m) => m.startsWith('[intlify]'));
    spy.mockRestore();
    expect(warnings).toEqual([]);
  };
});

// jsdom does not lay out pages, so it has no scrolling.
Element.prototype.scrollIntoView = () => {};

// Nor does it show anything: an observed element only comes into view through `intersect`.
const observers = new Set();
globalThis.IntersectionObserver = class {
  constructor(callback) {
    this.callback = callback;
    this.targets = new Set();
    observers.add(this);
  }
  observe(target) {
    this.targets.add(target);
  }
  disconnect() {
    this.targets.clear();
  }
};
beforeEach(() => observers.clear());

// Brings every observed element into view.
export function intersect() {
  for (const o of observers) {
    if (o.targets.size) o.callback([...o.targets].map((target) => ({ target, isIntersecting: true })));
  }
}

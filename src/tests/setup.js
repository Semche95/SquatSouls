import { beforeEach, vi } from 'vitest';
import { languageSetting, setLanguage } from '../i18n.js';

// The Tauri backend does not exist under Vitest: each test simulates its responses.
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(async () => () => {}) }));
vi.mock('@tauri-apps/api/app', () => ({ getVersion: vi.fn(async () => '0.1.0') }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn(async () => {}) }));
vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(async () => {}) }));

// French interface following the system by default: the tests check the French texts.
beforeEach(() => {
  languageSetting.value = 'system';
  return setLanguage('fr');
});

// jsdom does not lay out pages, so it has no scrolling.
Element.prototype.scrollIntoView = () => {};

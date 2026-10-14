import { beforeEach, describe, expect, it, vi } from 'vitest';
import { openUrl } from '@tauri-apps/plugin-opener';
import { setup, mountApp } from './app.js';
import { author, repository } from '../../package.json';

// Teleported to the body, outside the app's wrapper.
const modal = () => document.querySelector('.about-modal');

describe('about', () => {
  // jsdom lays nothing out: without this, the modal's focus trap finds no visible button.
  beforeEach(() => {
    const spy = vi.spyOn(HTMLButtonElement.prototype, 'getClientRects').mockReturnValue([{}]);
    return () => spy.mockRestore();
  });

  async function openAbout() {
    setup();
    const w = await mountApp();
    await w.find('.tabs .about').trigger('click');
    // Open once its focus trap holds the focus.
    await vi.waitFor(() => expect(modal()?.contains(document.activeElement)).toBe(true));
    return w;
  }

  it('opens from the menu without leaving the current tab', async () => {
    const w = await openAbout();
    expect(w.find('.tabs .active').text()).toBe('Jouer');
    expect(modal().textContent).toContain(`Par ${author}Version 0.1.0`);
  });

  it('opens the repository in the browser', async () => {
    await openAbout();
    modal().querySelector('a').click();
    expect(openUrl).toHaveBeenCalledWith(repository.url);
  });

  it('closes with its cross', async () => {
    await openAbout();
    modal().querySelector('.modal-close').click();
    await vi.waitFor(() => expect(modal()).toBeNull());
  });
});

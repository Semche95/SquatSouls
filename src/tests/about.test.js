import { describe, expect, it, vi } from 'vitest';
import { openUrl } from '@tauri-apps/plugin-opener';
import { setup, mountApp, layOutControls } from './app.js';
import { author, repository } from '../../package.json';

// Teleported to the body, outside the app's wrapper.
const modal = () => document.querySelector('.about-modal');

describe('about', () => {
  layOutControls();

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
    expect(w.find('.tabs .active').text()).toBe('Play');
    expect(modal().textContent).toContain(`v0.1.0 · by ${author}`);
  });

  it('states that the project is not affiliated with any game publisher', async () => {
    await openAbout();
    expect(modal().textContent).toContain('not affiliated with or endorsed by any game publisher');
  });

  it('shows the safety advice', async () => {
    await openAbout();
    expect(modal().textContent).toContain('Safety advice');
    expect(modal().querySelectorAll('li')).toHaveLength(5);
  });

  it('opens the repository in the browser', async () => {
    await openAbout();
    modal().querySelector('.about-source').click();
    expect(openUrl).toHaveBeenCalledWith(repository.url);
  });

  it('closes with its cross', async () => {
    await openAbout();
    modal().querySelector('.modal-close').click();
    await vi.waitFor(() => expect(modal()).toBeNull());
  });
});

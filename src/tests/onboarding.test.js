import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { setup, mountApp, layOutControls } from './app.js';

// Teleported to the body, outside the app's wrapper.
const screen = () => document.querySelector('.onboarding-screen');
const title = () => screen().querySelector('h2').textContent;
const button = (text) => [...screen().querySelectorAll('button')].find((b) => b.textContent === text);

describe('onboarding', () => {
  layOutControls();

  async function firstLaunch() {
    setup({ get_onboarding_seen: false });
    const w = await mountApp();
    await vi.waitFor(() => expect(screen()?.contains(document.activeElement)).toBe(true));
    return w;
  }

  // The previous step fades out before the next one shows.
  async function go(text, expected) {
    button(text).click();
    await vi.waitFor(() => expect(title()).toBe(expected));
  }

  async function toLastStep() {
    await go('Next', 'Craft your build');
    await go('Next', 'Take care of yourself');
    await go('Next', 'No need to git gud');
  }

  const closed = async () => {
    await vi.waitFor(() => expect(screen()).toBeNull());
    await flushPromises();
  };

  it('goes through the warnings one step at a time', async () => {
    await firstLaunch();
    expect(title()).toBe('Squat Souls');
    await go('Next', 'Craft your build');
    expect(screen().textContent).toContain('are only examples');
    await go('Next', 'Take care of yourself');
    await go('Back', 'Craft your build');
  });

  it('cannot be left before its last step at the first launch', async () => {
    await firstLaunch();
    expect(screen().querySelector('.modal-close')).toBeNull();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await flushPromises();
    expect(screen()).not.toBeNull();
  });

  it('is no longer shown once finished', async () => {
    await firstLaunch();
    await toLastStep();
    button('Let\'s go').click();
    await closed();
    expect(invoke).toHaveBeenCalledWith('set_onboarding_seen');
  });

  it('is not shown once seen', async () => {
    setup();
    await mountApp();
    expect(screen()).toBeNull();
  });

  it('opens the example exercises of the program', async () => {
    const w = await firstLaunch();
    await toLastStep();
    button('See the program').click();
    await closed();
    expect(w.find('.tabs .active').text()).toBe('Program');
    expect(w.find('.subtabs .active').text()).toBe('Exercises');
  });

});

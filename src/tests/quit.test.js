import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { setup, mountApp, press, emitEvent, layOutControls } from './app.js';

const ctrlQ = { key: 'q', ctrlKey: true };
// Teleported to the body, outside the app's wrapper.
const modal = () => document.querySelector('.confirm-modal');
const click = (selector) => {
  modal().querySelector(selector).click();
  return flushPromises();
};
// Removed at the end of its closing transition.
const closed = () => vi.waitFor(() => expect(modal()).toBeNull());
// Asked by the backend; open once its focus trap holds the focus.
async function ask() {
  await emitEvent('quit-requested');
  await vi.waitFor(() => expect(modal().contains(document.activeElement)).toBe(true));
}

describe('quit', () => {
  layOutControls();

  it('asks to quit with Ctrl+Q, whatever the tab and the layout', async () => {
    setup();
    const w = await mountApp();
    await w.findAll('.tabs button')[2].trigger('click');
    await press('KeyA', ctrlQ);
    expect(invoke).toHaveBeenCalledWith('request_quit');
  });

  it('uses Cmd+Q on macOS, not Ctrl+Q', async () => {
    vi.stubGlobal('navigator', { userAgent: 'Macintosh' });
    setup();
    await mountApp();
    await press('KeyQ', ctrlQ);
    expect(invoke).not.toHaveBeenCalledWith('request_quit');
    await press('KeyQ', { key: 'q', metaKey: true });
    expect(invoke).toHaveBeenCalledWith('request_quit');
    vi.unstubAllGlobals();
  });

  it('does not quit with Q alone', async () => {
    setup();
    await mountApp();
    await press('KeyA', { key: 'q' });
    expect(invoke).not.toHaveBeenCalledWith('request_quit');
  });

  it('confirms quitting during a session', async () => {
    setup();
    await mountApp();
    await ask();
    expect(modal().textContent).toContain('Quit Squat Souls?The current session stops');
    await click('button.danger');
    expect(invoke).toHaveBeenCalledWith('quit');
  });

  it('keeps the session with its button or Escape', async () => {
    setup();
    await mountApp();
    await ask();
    await click('button.ghost');
    await closed();
    await ask();
    modal().dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    await closed();
    expect(invoke).not.toHaveBeenCalledWith('quit');
  });

  it('quits with a second Ctrl+Q and silences the other shortcuts while open', async () => {
    setup({ draw: { capped: false } });
    await mountApp();
    await ask();
    await press('Digit1');
    expect(invoke).not.toHaveBeenCalledWith('draw', expect.anything());
    await press('KeyA', ctrlQ);
    expect(invoke).toHaveBeenCalledWith('quit');
  });
});

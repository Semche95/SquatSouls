import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { setup, mountApp, press, emitEvent, layOutControls, confirmDialog, confirmDialogOpen, confirmDialogClosed } from './app.js';

const ctrlQ = { key: 'q', ctrlKey: true };
const modal = () => confirmDialog().element;
const click = (selector) => {
  modal().querySelector(selector).click();
  return flushPromises();
};
// Asked by the backend.
async function ask() {
  await emitEvent('quit-requested');
  await confirmDialogOpen();
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

  it('confirms quitting during a session, which resumes at the next launch', async () => {
    setup();
    await mountApp();
    await ask();
    expect(modal().textContent).toContain(
      'Quit Squat Souls?The current session will resume where it left off the next time Squat Souls opens.',
    );
    await click('button.primary');
    expect(invoke).toHaveBeenCalledWith('quit');
  });

  it('stays with its button or Escape', async () => {
    setup();
    await mountApp();
    await ask();
    expect(modal().querySelector('button.ghost').textContent).toContain('Stay');
    await click('button.ghost');
    await confirmDialogClosed();
    await ask();
    modal().dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    await confirmDialogClosed();
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

import { describe, expect, it } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { setup, mountApp, emitEvent, emptySession, expectTip } from './app.js';

const banner = (w) => w.find('.banner.update');

describe('update', () => {
  it('shows nothing without a downloaded update', async () => {
    setup({ get_update: null });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('announces the update downloaded before the interface', async () => {
    setup({ get_update: '0.2.0', get_session: emptySession({ startedAt: null }) });
    const w = await mountApp();
    expect(banner(w).text()).toContain('La version 0.2.0 sera installée en quittant.');
  });

  it('announces the update once downloaded and restarts on request outside a session', async () => {
    setup({ get_update: null, get_session: emptySession({ startedAt: null }), restart_to_update: undefined });
    const w = await mountApp();
    await emitEvent('update-ready', '0.2.0');
    expect(banner(w).text()).toContain('0.2.0');
    await banner(w).find('button').trigger('click');
    expect(invoke).toHaveBeenCalledWith('restart_to_update', undefined);
  });

  it('does not restart during a session', async () => {
    setup({ get_update: '0.2.0' });
    const w = await mountApp();
    const button = banner(w).find('button');
    expect(button.element.disabled).toBe(true);
    await expectTip(button, 'Disponible une fois la séance terminée');
  });
});

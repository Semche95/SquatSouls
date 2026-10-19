import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { setup, mountApp, emitEvent, emptySession, idleSession, expectTip, openSettings, save, savedConfig } from './app.js';

const banner = (w) => w.find('.banner.update');
const downloading = (progress = null) => ({ version: '0.2.0', progress, stage: 'downloading' });
const ready = { version: '0.2.0', progress: 1, stage: 'ready' };

describe('update', () => {
  it('shows nothing without an update', async () => {
    setup({ get_update: null });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('announces the update found before the interface, while it downloads', async () => {
    setup({ get_update: downloading() });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Version 0.2.0 is available');
    expect(banner(w).text()).toContain('Downloading in the background…');
    expect(banner(w).find('.update-progress.unknown').exists()).toBe(true);
    expect(banner(w).find('button').exists()).toBe(false);
  });

  it('follows the download progress', async () => {
    setup({ get_update: null });
    const w = await mountApp();
    await emitEvent('update-changed', downloading(0.42));
    expect(banner(w).find('.update-progress > span').attributes('style')).toContain('width: 42%');
  });

  it('drops the banner when the check fails', async () => {
    setup({ get_update: downloading() });
    const w = await mountApp();
    await emitEvent('update-changed', null);
    expect(banner(w).exists()).toBe(false);
  });

  it('announces the downloaded update and restarts on request outside a session', async () => {
    setup({ get_update: null, get_session: idleSession(), restart_to_update: undefined });
    const w = await mountApp();
    await emitEvent('update-changed', ready);
    expect(banner(w).text()).toContain('Version 0.2.0 is ready');
    expect(banner(w).text()).toContain('It will be installed when you quit.');
    expect(banner(w).find('.update-progress').exists()).toBe(false);
    await banner(w).find('button').trigger('click');
    expect(invoke).toHaveBeenCalledWith('restart_to_update', undefined);
  });

  it('does not restart during a session', async () => {
    setup({ get_update: ready, get_session: emptySession() });
    const w = await mountApp();
    const button = banner(w).find('button');
    expect(button.element.disabled).toBe(true);
    await expectTip(button, 'Available once the session is over');
  });

  it('tells during a session that the update waits for quitting outside a session', async () => {
    setup({ get_update: ready, get_session: emptySession() });
    const w = await mountApp();
    expect(banner(w).text()).toContain('It installs when the app quits outside a session.');
    await emitEvent('update-changed', { version: '0.2.0', progress: null, stage: 'available' });
    expect(banner(w).text()).toContain('Ready to download. It installs when the app quits outside a session.');
  });

  it('downloads the update found on request when it is not automatic', async () => {
    setup({ get_update: { version: '0.2.0', progress: null, stage: 'available' }, download_update: undefined });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Version 0.2.0 is available');
    expect(banner(w).text()).toContain('Ready to download. It installs when the app quits.');
    expect(banner(w).find('.update-progress').exists()).toBe(false);
    await banner(w).find('button').trigger('click');
    expect(invoke).toHaveBeenCalledWith('download_update', undefined);
  });

  it('downloads the update again on request when the download fails', async () => {
    setup({ get_update: downloading(0.42), download_update: undefined });
    const w = await mountApp();
    await emitEvent('update-changed', { version: '0.2.0', progress: null, stage: 'failed' });
    expect(banner(w).text()).toContain('The download failed. Check the connection, then try again.');
    expect(banner(w).find('.update-progress').exists()).toBe(false);
    const button = banner(w).find('button');
    expect(button.text()).toBe('Try again');
    await button.trigger('click');
    expect(invoke).toHaveBeenCalledWith('download_update', undefined);
  });

  it('sends the packages that cannot install it to the release page', async () => {
    setup({ get_update: { version: '0.2.0', progress: null, stage: 'manual' } });
    const w = await mountApp();
    expect(banner(w).text()).toContain('Version 0.2.0 is available');
    expect(banner(w).text()).toContain('Download the new package to install it.');
    expect(banner(w).find('.update-progress').exists()).toBe(false);
    await banner(w).find('button').trigger('click');
    expect(openUrl).toHaveBeenCalledWith('https://github.com/Semche95/SquatSouls/releases/latest');
  });

  describe('settings', () => {
    const check = (w, label) => w.findAll('.check').find((c) => c.text() === label);

    it('are saved with the Settings tab', async () => {
      setup({ save_config: (args) => ({ config: args.config }) });
      const w = await mountApp();
      await openSettings(w);
      await check(w, 'Update without asking').find('input').setValue(false);
      await save(w).trigger('click');
      await flushPromises();
      expect(savedConfig()).toMatchObject({ updateCheck: true, updateAuto: false });
    });

    it('only offer to update without asking while updates are checked', async () => {
      setup();
      const w = await mountApp();
      await openSettings(w);
      await check(w, 'Check for updates').find('input').setValue(false);
      expect(check(w, 'Update without asking')).toBeUndefined();
    });
  });
});

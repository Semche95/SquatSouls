import { describe, expect, it } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { i18n } from '../i18n.js';
import { setup, mountApp } from './app.js';
import StartupFailure from '../components/StartupFailure.vue';

async function mountFailure(code) {
  const w = mount(StartupFailure, { props: { error: { code, params: {} } }, global: { plugins: [i18n] } });
  await flushPromises();
  return w;
}

const buttons = (w) => w.findAll('.startup-failure button');

describe('startup failure', () => {
  it('reports data the version could not prepare, a fix coming with an update', async () => {
    setup({ get_update: null, quit: undefined });
    const w = await mountFailure('database_not_prepared');
    expect(w.find('.startup-failure').text()).toContain('The data could not be opened');
    expect(w.find('.startup-failure').text()).toContain('a fix will come with an update');
    expect(buttons(w).map((b) => b.text())).toEqual(['Report the problem on GitHub', 'Get the latest version on GitHub', 'Quit']);
    await buttons(w)[0].trigger('click');
    expect(openUrl).toHaveBeenCalledWith('https://github.com/Semche95/SquatSouls/issues/new');
    await buttons(w)[1].trigger('click');
    expect(openUrl).toHaveBeenCalledWith('https://github.com/Semche95/SquatSouls/releases/latest');
    await buttons(w)[2].trigger('click');
    expect(invoke).toHaveBeenCalledWith('quit');
    w.unmount();
  });

  it('reports a damaged file', async () => {
    setup({ get_update: null });
    const w = await mountFailure('database_damaged');
    expect(w.find('.startup-failure').text()).toContain('The data file is damaged and cannot be read.');
    expect(buttons(w).map((b) => b.text())).toEqual(['Report the problem on GitHub', 'Quit']);
    w.unmount();
  });

  it.each([
    ['database_denied', 'Squat Souls cannot write its data.'],
    ['database_full', 'The disk is full.'],
  ])('lets the user fix %s, then try again', async (code, text) => {
    setup({ get_update: null, relaunch: undefined });
    const w = await mountFailure(code);
    expect(w.find('.startup-failure').text()).toContain(text);
    expect(buttons(w).map((b) => b.text())).toEqual(['Try again', 'Quit']);
    await buttons(w)[0].trigger('click');
    expect(invoke).toHaveBeenCalledWith('relaunch');
    w.unmount();
  });

  it('shows the update found under the reason, without the GitHub link to the latest version', async () => {
    setup({ get_update: { version: '0.2.0', progress: null, stage: 'downloading' } });
    const w = await mountFailure('database_not_prepared');
    expect(w.find('.startup-failure + .banner.update').text()).toContain('Version 0.2.0 is available');
    expect(buttons(w).map((b) => b.text())).toEqual(['Report the problem on GitHub', 'Quit']);
    w.unmount();
  });
});

describe('restored database', () => {
  const banner = (w) => w.find('.banner.closable');

  it('says nothing after a normal launch', async () => {
    setup({ was_restored: false });
    const w = await mountApp();
    expect(banner(w).exists()).toBe(false);
  });

  it('says the copy of the previous launch was restored, until closed', async () => {
    setup({ was_restored: true });
    const w = await mountApp();
    expect(banner(w).text()).toContain('The copy saved at the previous launch was restored');
    await banner(w).find('button').trigger('click');
    expect(banner(w).exists()).toBe(false);
  });
});

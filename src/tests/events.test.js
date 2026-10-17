import { describe, expect, it } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { setup, openConfig, expectTip, layOutControls, editDialog, editDialogClosed, save, savedConfig } from './app.js';
import { openTile, closeDialog, saveAndReopen } from './program.js';

describe('events', () => {
  layOutControls();

  it('opens an added event in its dialog with its name selected', async () => {
    setup();
    const w = await openConfig();
    await openTile(w.find('.cfg-events .tile-new'));
    const name = editDialog().find('input[aria-label="Name of the event"]').element;
    expect(name.value).toBe('New event');
    expect(document.activeElement).toBe(name);
    expect([name.selectionStart, name.selectionEnd]).toEqual([0, name.value.length]);
    await closeDialog();
    expect(w.findAll('.cfg-events .tile-name').map((n) => n.text())).toEqual(['Death', 'Boss defeated', 'New event']);
  });

  it('deletes an event from its dialog, waiting for save', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await openTile(w.findAll('.cfg-events .tile')[1]);
    await editDialog().find('.edit-foot .danger').trigger('click');
    await editDialogClosed();
    expect(w.findAll('.cfg-events .tile-name').map((n) => n.text())).toEqual(['Death']);
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().games[0].events.map((e) => e.id)).toEqual(['death']);
  });

  it('refuses an empty name: the dialog says so and save is disabled', async () => {
    setup();
    const w = await openConfig();
    await openTile(w.find('.cfg-events .tile'));
    const name = editDialog().find('input[aria-label="Name of the event"]');
    await name.setValue(' ');
    expect(name.attributes('aria-invalid')).toBe('true');
    expect(editDialog().find('.field-error').text()).toBe('Give the event a name');
    await closeDialog();
    expect(w.find('.cfg-events .tile-name').text()).toBe('(unnamed)');
    expect(save(w).element.disabled).toBe(true);
    await expectTip(save(w), 'Elden Ring › event name: it cannot be empty.');
  });

  it('clears the per-session limit to remove it, with the field or its - button', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openConfig();
    await openTile(w.find('.cfg-events .tile'));
    expect(w.find('.cfg-events .tile').text()).toContain('15 times max per session');
    await editDialog().find('#ev-limit').setValue('');
    expect(w.find('.cfg-events .tile').text()).not.toContain('max per session');
    // Below 1, - removes the limit, here of the second event.
    await saveAndReopen(w, () => w.findAll('.cfg-events .tile')[1]);
    expect(savedConfig().games[0].events[0]).not.toHaveProperty('maxPerSession');
    const less = () => editDialog().findAll('button[aria-label="Less"]')[1];
    await editDialog().findAll('button[aria-label="More"]')[1].trigger('click');
    expect(editDialog().find('#ev-limit').element.value).toBe('1');
    await less().trigger('click');
    expect(editDialog().find('#ev-limit').element.value).toBe('');
    expect(less().element.disabled).toBe(true);
    await expectTip(less(), 'Already empty');
  });
});

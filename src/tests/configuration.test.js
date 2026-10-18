import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { rejection, responses, setup, openConfig, expectUnchanged, expectTip, layOutControls, editDialog, save, discard, savedConfig } from './app.js';
import { openExercises, openTile, closeDialog, shownGame, rename } from './program.js';

describe('configuration', () => {
  layOutControls();

  it('edits the configuration with fields and applies it on save', async () => {
    setup();
    const w = await openConfig();
    expectUnchanged(w);
    await expectTip(save(w), 'No pending changes.');
    await openTile(w.find('.cfg-events .tile'));
    expect(document.activeElement).toBe(editDialog().find('.edit-foot .primary').element);
    await editDialog().find('input[aria-label="Name of the event"]').setValue('Brutal death');
    await closeDialog();
    expect(w.find('.cfg-events .tile-name').text()).toBe('Brutal death');
    vi.useFakeTimers();
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().games[0].events[0]).toMatchObject({ id: 'death', name: 'Brutal death' });
    expect(w.find('.banner.ok').text()).toBe('Configuration saved.');
    await vi.runAllTimersAsync();
    expect(w.find('.banner.ok').exists()).toBe(false);
    await w.findAll('.tabs button')[0].trigger('click');
    expect(w.find('.event').text()).toContain('Brutal death');
  });

  it('resets unsaved changes', async () => {
    setup();
    const w = await openConfig();
    await rename(w, 'Other');
    await discard(w).trigger('click');
    expect(shownGame(w)).toBe('Elden Ring');
    expectUnchanged(w);
  });

  it('shows the save error', async () => {
    setup({ save_config: rejection('exercise_range', { name: 'Squats', min: 20, max: 15, limit: 9999 }) });
    const w = await openConfig();
    await rename(w, 'Other');
    await save(w).trigger('click');
    await flushPromises();
    expect(w.find('.banner.error').text()).toBe('Invalid range for "Squats" (20 to 15): from 1 to 9999, the minimum not above the maximum.');
  });

  it('disables saving while a field is badly filled and gives the reason', async () => {
    setup();
    const w = await openExercises();
    const min = editDialog().find('input[aria-label="Minimum"]');
    await min.setValue('-2');
    expect(save(w).attributes('disabled')).toBeDefined();
    await expectTip(save(w), 'Squats › minimum: a whole number is needed, 1 or more.');
    await min.setValue('2');
    expect(save(w).attributes('disabled')).toBeUndefined();
  });
});

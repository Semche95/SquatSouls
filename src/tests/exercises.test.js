import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { emptySession, idleSession, responses, setup, openConfig, shownTip, expectTip, layOutControls, editDialog, save, savedConfig } from './app.js';
import { openExercises, openTile, closeDialog, saveAndReopen } from './program.js';

describe('exercises', () => {
  layOutControls();
  // Choice of the timer in the open exercise dialog.
  const timer = (text) => editDialog().findAll('[role="radio"]').find((b) => b.text() === text);

  it('adds an exercise with an id derived from the name, its dialog open with the name selected', async () => {
    setup();
    const w = await openConfig();
    await w.findAll('.subtabs button')[1].trigger('click');
    expect(w.find('#cfg-game').exists()).toBe(false);
    for (let i = 0; i < 2; i++) {
      await openTile(w.find('.cfg-exercises .tile-new'));
      const name = editDialog().find('input[aria-label="Name of the exercise"]').element;
      expect(document.activeElement).toBe(name);
      expect([name.selectionStart, name.selectionEnd]).toEqual([0, name.value.length]);
      await closeDialog();
    }
    expect(w.findAll('.cfg-exercises .tile-name').map((n) => n.text())).toEqual(['Squats', 'New exercise', 'New exercise']);
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().exercises.map((e) => e.id)).toEqual(['squat', 'new-exercise', 'new-exercise-2']);
  });

  it('removes the last exercise', async () => {
    setup();
    const w = await openExercises({ dialog: false });
    await openTile(w.find('.tile-slot .tile'));
    await editDialog().find('.edit-foot .danger').trigger('click');
    await flushPromises();
    expect(w.findAll('.cfg-exercises .tile-name')).toHaveLength(0);
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().exercises).toEqual([]);
  });

  it('tells why the session ended once no exercise is included, encouraging only if exercises were done', async () => {
    setup({ get_session: emptySession(), save_config: (args) => ({ config: args.config }) });
    const w = await openExercises({ dialog: false });
    await w.find('input[aria-label="Include this exercise"]').setValue(false);
    // Without any exercise done, the backend cancels the session, telling why.
    responses.get_session = idleSession({ ended: { saved: false, cause: 'noExercise' } });
    await save(w).trigger('click');
    await flushPromises();
    await w.findAll('.tabs button')[0].trigger('click');
    expect(w.find('.banner.info').text()).toBe('The session ended: no exercise is included anymore.');
    expect(w.find('.farewell').exists()).toBe(false);
  });

  it('shows an exercise\'s range on its tile and sets it in its dialog', async () => {
    setup();
    const w = await openExercises({ dialog: false });
    expect(w.find('.tile-slot .tile-foot').text()).toBe('10–15 reps');
    await openTile(w.find('.tile-slot .tile'));
    // The minimum never goes above the maximum.
    await editDialog().find('input[aria-label="Maximum"]').setValue('11');
    const more = editDialog().findAll('button[aria-label="More"]')[0];
    await more.trigger('click');
    expect(w.find('.tile-slot .tile-foot').text()).toBe('11–11 reps');
    expect(more.element.disabled).toBe(true);
    await expectTip(more, 'Already at 11');
    await closeDialog();
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().exercises[0]).toMatchObject({ min: 11, max: 11 });
  });

  it('disables an exercise from its tile or its dialog to keep it out of the draw without removing it', async () => {
    setup();
    const w = await openExercises({ dialog: false });
    const tile = w.find('.tile-slot');
    const title = () => w.find('.cfg-exercises .group-head').text();
    expect(title()).toContain('Exercises · 1 included out of 1');
    expect(title()).toContain('Switch an exercise off to stop it being asked for, without deleting it.');
    expect(tile.classes()).not.toContain('cfg-off');
    await w.find('input[aria-label="Include this exercise"]').setValue(false);
    expect(tile.classes()).toContain('cfg-off');
    expect(title()).toContain('Exercises · 0 included out of 1');
    await openTile(tile.find('.tile'));
    const included = editDialog().find('.edit-aside input');
    expect(included.element.checked).toBe(false);
    await included.setValue(true);
    expect(tile.classes()).not.toContain('cfg-off');
    await included.setValue(false);
    await closeDialog();
    responses.save_config = (args) => ({ config: args.config });
    await save(w).trigger('click');
    await flushPromises();
    expect(savedConfig().exercises).toEqual([expect.objectContaining({ id: 'squat', enabled: false })]);
  });

  it('sets the global recovery and an exercise\'s own recovery', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const own = () => editDialog().find('#ex-recovery');
    await w.find('#cfg-recovery').setValue(45);
    expect(own().attributes('placeholder')).toBe('45');
    await own().setValue('90');
    expect(w.find('.tile-slot .tile').text()).toContain('recovery 90 s');
    const tile = () => w.find('.tile-slot .tile');
    await saveAndReopen(w, tile);
    expect(savedConfig()).toMatchObject({ recovery: 45, exercises: [{ id: 'squat', recovery: 90 }] });
    await own().setValue('');
    await saveAndReopen(w, tile);
    expect(savedConfig().exercises[0]).not.toHaveProperty('recovery');
    // Empty, + starts from the global recovery.
    await editDialog().findAll('button[aria-label="More"]').at(-1).trigger('click');
    expect(own().element.value).toBe('46');
  });

  it('times an exercise in seconds or minutes, or no longer', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    expect(editDialog().findAll('[role="radio"]').map((b) => b.text())).toEqual(['Not timed', 'Seconds', 'Minutes']);
    expect(timer('Not timed').attributes('aria-checked')).toBe('true');
    await timer('Minutes').trigger('click');
    expect(w.find('.timer-icon').attributes('aria-label')).toBe('Timed in minutes');
    const tile = () => w.find('.tile-slot .tile');
    await saveAndReopen(w, tile);
    expect(savedConfig().exercises[0].timed).toBe('minutes');
    await timer('Not timed').trigger('click');
    expect(w.find('.timer-icon').exists()).toBe(false);
    await saveAndReopen(w, tile);
    expect(savedConfig().exercises[0]).not.toHaveProperty('timed');
  });

  it('repeats a timed exercise for the other side, explained, until it is no longer timed', async () => {
    setup({ save_config: (args) => ({ config: args.config }) });
    const w = await openExercises();
    const eachSide = () => editDialog().find('#ex-each-side');
    expect(eachSide().exists()).toBe(false);
    await timer('Seconds').trigger('click');
    expect(editDialog().text()).toContain('Repeat for the other sideThe duration is for one side: the timer runs again for the other side.');
    await eachSide().setValue(true);
    expect(w.find('.timer-icon').attributes('aria-label')).toBe('Timed in seconds, on each side');
    const tile = () => w.find('.tile-slot .tile');
    await saveAndReopen(w, tile);
    expect(savedConfig().exercises[0]).toMatchObject({ timed: 'seconds', eachSide: true });
    await timer('Not timed').trigger('click');
    expect(eachSide().exists()).toBe(false);
    await saveAndReopen(w, tile);
    expect(savedConfig().exercises[0].eachSide).toBe(false);
  });

  it('reports an invalid format while typing', async () => {
    setup();
    const w = await openExercises();
    const min = editDialog().find('input[aria-label="Minimum"]');
    await min.trigger('focus');
    await min.setValue('abc');
    await vi.waitFor(() => expect(shownTip()).toBe('a whole number is needed, 1 or more'));
    await min.trigger('blur');
    await vi.waitFor(() => expect(shownTip()).toBeUndefined());
    await min.trigger('mouseenter');
    await vi.waitFor(() => expect(shownTip()).toBe('a whole number is needed, 1 or more'));
    await min.trigger('mouseleave');
    await min.trigger('focus');
    expect(min.attributes('aria-invalid')).toBe('true');
    expect(w.find('.banner.error').exists()).toBe(false);
    await min.setValue('12');
    expect(min.attributes('aria-invalid')).toBe('false');
    await vi.waitFor(() => expect(shownTip()).toBeUndefined());
  });

  it('brings a cleared required field back to 0 when leaving it', async () => {
    setup();
    const w = await openExercises();
    const min = editDialog().find('input[aria-label="Minimum"]');
    await min.setValue('');
    await min.trigger('change');
    expect(min.element.value).toBe('0');
  });
});

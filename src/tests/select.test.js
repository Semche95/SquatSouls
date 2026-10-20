import { describe, expect, it } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { nextTick, ref } from 'vue';
import { vSelect } from '../ui/select.js';

// A select drawn by Tom Select, its options and state given by the test.
const mountSelect = async (options, { value = '', disabled = false } = {}) => {
  const state = { options: ref(options), value: ref(value), disabled: ref(disabled) };
  const w = mount(
    {
      directives: { select: vSelect },
      setup: () => state,
      template: `<select v-select v-model="value" :disabled="disabled">
        <option v-for="o in options" :key="o.value" :value="o.value" :disabled="o.disabled">{{ o.text }}</option>
      </select>`,
    },
    { attachTo: document.body },
  );
  await nextTick();
  return { w, ...state, ts: w.element.tomselect };
};
const games = [
  { value: '', text: 'Choose a game', disabled: true },
  { value: 'er', text: 'Elden Ring' },
  { value: 'sekiro', text: 'Sekiro' },
];
const labels = (ts) => Object.values(ts.options).sort((a, b) => a.$order - b.$order).map((o) => o.text);

describe('select', () => {
  it('gives the option chosen in the list to the model without reordering the options', async () => {
    const { w, value, ts } = await mountSelect(games);
    ts.setValue('er');
    await flushPromises();
    expect(value.value).toBe('er');
    expect(w.findAll('option').map((o) => o.text())).toEqual(['Choose a game', 'Elden Ring', 'Sekiro']);
    w.unmount();
  });

  it('shows the model and the options changed by the app', async () => {
    const { w, options, value, disabled, ts } = await mountSelect(games);
    value.value = 'sekiro';
    options.value = [...games, { value: 'ds', text: 'Dark Souls' }];
    disabled.value = true;
    await flushPromises();
    expect(ts.getValue()).toBe('sekiro');
    expect(labels(ts)).toEqual(['Choose a game', 'Elden Ring', 'Sekiro', 'Dark Souls']);
    expect(ts.isDisabled).toBe(true);
    w.unmount();
  });

  it('shows the empty option like the others, whether it can be chosen or not', async () => {
    const { w, value, ts } = await mountSelect([{ value: '', text: 'Copy nothing' }, ...games.slice(1)], { value: 'er' });
    ts.setValue('');
    await flushPromises();
    expect(value.value).toBe('');
    expect(ts.control.textContent).toBe('Copy nothing');
    w.unmount();
    const locked = await mountSelect(games);
    expect(locked.ts.control.textContent).toBe('Choose a game');
    expect(locked.ts.options[''].disabled).toBe(true);
    locked.w.unmount();
  });

  it('opens on a click but not on the focus alone, without a search field', async () => {
    const { w, ts } = await mountSelect(games, { value: 'er' });
    expect(ts.control.querySelector('input')).toBeNull();
    ts.focus();
    await new Promise((resolve) => setTimeout(resolve));
    expect(ts.isOpen).toBe(false);
    ts.control.click();
    expect(ts.isOpen).toBe(true);
    ts.control.click();
    expect(ts.isOpen).toBe(false);
    w.unmount();
  });

  it('keeps its value on Backspace and Delete', async () => {
    const { w, value, ts } = await mountSelect(games, { value: 'er' });
    ts.focus();
    for (const keyCode of [8, 46]) ts.control.dispatchEvent(new KeyboardEvent('keydown', { keyCode, bubbles: true }));
    await flushPromises();
    expect(value.value).toBe('er');
    w.unmount();
  });
});

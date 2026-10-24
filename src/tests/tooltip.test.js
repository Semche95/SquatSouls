import { describe, expect, it } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { nextTick, ref } from 'vue';
import { vTooltip } from '../ui/tooltip.js';

describe('tooltip', () => {
  // floating-vue resizes the tooltip once its text changed, which threw when it was gone by then (patched): Vitest
  // fails the run on that unhandled rejection.
  it('goes away with its element right after its text changes', async () => {
    const state = { tip: ref('Choose a game'), shown: ref(true) };
    const w = mount({ directives: { tooltip: vTooltip }, setup: () => state, template: '<button v-if="shown" v-tooltip="{ content: tip }">Start</button>' }, { attachTo: document.body });
    await nextTick();
    // The element goes away right after, as after an answer of the backend.
    state.tip.value = 'No exercise included';
    await Promise.resolve();
    state.shown.value = false;
    await flushPromises();
    await new Promise((resolve) => {
      setTimeout(resolve);
    });
    expect(w.find('button').exists()).toBe(false);
    w.unmount();
  });
});

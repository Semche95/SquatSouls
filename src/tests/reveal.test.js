import { afterEach, describe, expect, it, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { nextTick, ref } from 'vue';
import { reveal, vReveal } from '../reveal.js';

// A block revealed when it appears and an accordion revealed when it opens, their state given by the test.
const mountBlocks = () => {
  const state = { shown: ref(false), open: ref(false) };
  const w = mount({
    directives: { reveal: vReveal },
    setup: () => state,
    template: `<div>
      <p v-if="shown" v-reveal class="shown">Saved.</p>
      <div v-reveal="open" class="accordion">Squats</div>
    </div>`,
  });
  return { w, ...state };
};
const scrolled = () => Element.prototype.scrollIntoView.mock.contexts.map((el) => el.className);

describe('reveal', () => {
  afterEach(() => vi.restoreAllMocks());

  it('scrolls a block into view as soon as it is shown', async () => {
    vi.spyOn(Element.prototype, 'scrollIntoView');
    const { shown } = mountBlocks();
    expect(scrolled()).toEqual([]);
    shown.value = true;
    await nextTick();
    expect(scrolled()).toEqual(['shown']);
    expect(Element.prototype.scrollIntoView).toHaveBeenCalledWith({ block: 'nearest' });
  });

  it('scrolls an accordion into view each time it opens, not when it closes', async () => {
    vi.spyOn(Element.prototype, 'scrollIntoView');
    const { open } = mountBlocks();
    for (const value of [true, false, true]) {
      open.value = value;
      await nextTick();
    }
    expect(scrolled()).toEqual(['accordion', 'accordion']);
  });

  it('scrolls a block further so the save bar does not cover it, its top staying in view', () => {
    vi.spyOn(Element.prototype, 'scrollIntoView');
    const scrollBy = vi.spyOn(window, 'scrollBy').mockImplementation(() => {});
    const w = mount({ template: '<main class="config"><div class="cfg-event">Death</div><div class="cfg-actions"></div></main>' });
    const box = (el, top, bottom) => vi.spyOn(el, 'getBoundingClientRect').mockReturnValue({ top, bottom });
    const [block, bar] = [w.find('.cfg-event').element, w.find('.cfg-actions').element];
    box(bar, 700, 780);
    for (const [top, bottom] of [[500, 690], [600, 760], [50, 900]]) {
      box(block, top, bottom);
      reveal(block);
    }
    expect(scrollBy.mock.calls).toEqual([[0, 60], [0, 50]]);
  });
});

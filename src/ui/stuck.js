// Marks a bar kept at the bottom of the window (position: sticky; bottom: 0) with the class `stuck` while it sits
// there over the content, rather than in its place under it. Stuck, its bottom edge touches the window's: the window
// shrunk by one pixel no longer holds it entirely.
export const vStuck = {
  mounted(el) {
    el.stuckObserver = new IntersectionObserver(
      ([entry]) => el.classList.toggle('stuck', entry.intersectionRatio < 1),
      { rootMargin: '0px 0px -1px 0px', threshold: 1 },
    );
    el.stuckObserver.observe(el);
  },
  beforeUnmount: (el) => el.stuckObserver.disconnect(),
};

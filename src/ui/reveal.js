// Scrolls an element just enough to show it entirely in the window, or as much of it as fits.
// A tab's action bar (save, end of the session) stays at the bottom of the window: the element must clear it too,
// its top staying in view.
export function reveal(el) {
  el.scrollIntoView({ block: 'nearest' });
  const bar = el.closest('.panel')?.querySelector('.bottom-bar');
  if (!bar || bar.contains(el)) return;
  const box = el.getBoundingClientRect();
  const hidden = Math.min(box.bottom - bar.getBoundingClientRect().top, box.top);
  if (hidden > 0) window.scrollBy(0, hidden);
}

// Reveals an element shown in place of another, such as a confirmation, as soon as it appears.
// Given a value, reveals it instead each time that value changes to a truthy one, such as an accordion opening.
export const vReveal = {
  mounted: (el, { value }) => value === undefined && reveal(el),
  updated: (el, { value, oldValue }) => value && value !== oldValue && reveal(el),
};

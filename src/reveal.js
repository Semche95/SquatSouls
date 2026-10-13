// Scrolls an element shown in place of a button into view, so a confirmation is never cut off.
export const vReveal = {
  mounted: (el) => el.scrollIntoView({ block: 'nearest' }),
};

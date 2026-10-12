import { options, vTooltip } from 'floating-vue';

// floating-vue takes a numeric 0 hide delay for a missing one and hides after a NaN timeout: the string keeps it at 0.
options.themes.tooltip.delay.hide = '0';

export { vTooltip };

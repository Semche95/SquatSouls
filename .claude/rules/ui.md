---
paths:
  - "src/**/*.vue"
  - "src/style/**"
---

# Interface

- Every checkbox is a switch, never a native checkbox: `style/controls.css` styles every `input type="checkbox"` as one, for the whole app.
- Never mark a state or an emphasis with a thicker or colored left border (accent bar, side stripe): use the background, the text color or an existing control instead.
- A disabled button must look disabled: check that `style/base.css` gives it a `:disabled` style (reduced opacity, no pointer cursor nor hover effect), rather than relying on the `disabled` attribute alone. The reason it is disabled is shown in a tooltip by wrapping it in `DisabledTip` (`components/DisabledTip.vue`), never with a `title`.

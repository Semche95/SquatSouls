import { nextTick } from 'vue';
import TomSelect from 'tom-select';

// Tom Select made to behave like a native select.
class Select extends TomSelect {
  // It appends the chosen <option> to the end of the select, which would reorder the options Vue renders:
  // selecting it in place is enough, every value having its option.
  updateOriginalInput({ silent } = {}) {
    this.input.value = this.getValue();
    if (this.isSetup && !silent) this.trigger('change', this.getValue());
  }

  // A click on the field opens or closes the list; the focus alone (Tab, click on the label) does not open it.
  onClick() {
    if (this.isOpen) this.close();
    else {
      this.focus();
      this.open();
    }
  }

  // Backspace and Delete would empty the select.
  deleteSelection() {
    return false;
  }
}

// What Tom Select copies from the select: it is synced again when it changes (options, language, value, disabled).
const snapshot = (el) => JSON.stringify([el.value, el.disabled, [...el.options].map((o) => [o.value, o.text, o.disabled])]);

// Draws a native select with Tom Select, the select Vue renders staying the source of truth. No search field, and
// the empty option ("Choose a game", "Copy nothing") shown like the others rather than as a placeholder.
const settings = { controlInput: null, openOnFocus: false, allowEmptyOption: true };

// With `v-select="{ search, none }"`, a field at the top of the list filters it, `search` its placeholder and `none`
// what it says when no option matches. The options keep their order while filtered, without highlighting the text typed,
// like the list of the games in a wide window.
const searchSettings = ({ none }) => ({
  ...settings,
  controlInput: '<input type="text" autocomplete="off" size="1" />',
  plugins: ['dropdown_input'],
  sortField: '$order',
  highlight: false,
  render: {
    no_results: () => {
      const p = document.createElement('div');
      p.className = 'no-results';
      p.textContent = none;
      return p;
    },
  },
});

// Texts of the search field, following the language.
function searchTexts(el, binding) {
  const ts = el.tomselect;
  if (!ts || !binding.value) return;
  ts.control_input.placeholder = binding.value.search;
  ts.control_input.setAttribute('aria-label', binding.value.search);
  ts.settings.render.no_results = searchSettings(binding.value).render.no_results;
}

export const vSelect = {
  // After the next tick, once v-model has selected the model's option.
  mounted: (el, binding) =>
    nextTick(() => {
      if (!el.isConnected) return;
      new Select(el, binding.value ? searchSettings(binding.value) : settings);
      searchTexts(el, binding);
      el.snapshot = snapshot(el);
    }),
  updated: (el, binding) =>
    nextTick(() => {
      searchTexts(el, binding);
      const ts = el.tomselect;
      if (!ts || el.snapshot === snapshot(el)) return;
      el.snapshot = snapshot(el);
      // Clearing the items empties the select: its value is put back for sync() to read it.
      const { value } = el;
      ts.clear(true);
      ts.clearOptions();
      el.value = value;
      ts.sync();
    }),
  beforeUnmount: (el) => el.tomselect?.destroy(),
};

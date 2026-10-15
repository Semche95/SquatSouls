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

export const vSelect = {
  // After the next tick, once v-model has selected the model's option.
  mounted: (el) =>
    nextTick(() => {
      if (!el.isConnected) return;
      new Select(el, settings);
      el.snapshot = snapshot(el);
    }),
  updated: (el) =>
    nextTick(() => {
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

import { ref } from 'vue';
import { defineShared } from './shared.js';

// Item of a draft list (event or exercise) open in its edit dialog, `list` giving that list. The tab that holds the
// list provides it to its dialog, which edits the item in place: the changes wait for Save. The item stays set while
// the dialog fades out; one just added is fresh, opened with its name selected.
export const [provideEditDialog, useEditDialog] = defineShared((list) => {
  const item = ref(null);
  const open = ref(false);
  const fresh = ref(false);

  function edit(it, added = false) {
    item.value = it;
    fresh.value = added;
    open.value = true;
  }

  // The removal waits for Save, like the other changes to the list.
  function remove() {
    const items = list();
    items.splice(items.indexOf(item.value), 1);
    open.value = false;
  }

  return { item, open, fresh, edit, remove };
});

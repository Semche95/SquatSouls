<script setup>
import { VueFinalModal } from 'vue-final-modal';
import { MAX_NAME } from '../configDraft.js';
import { t } from '../i18n.js';
import { vReveal } from '../reveal.js';
import DisabledTip from './DisabledTip.vue';

// Edit dialog of an event or an exercise: its name under the kind of item, its settings in rows (EditRow), then its
// removal and Close. `item` belongs to the draft and is edited in place: the changes wait for Save.
// A fresh item opens with its name selected, the others on Close.
defineProps({
  item: { type: Object, required: true },
  title: { type: String, required: true },
  nameLabel: { type: String, required: true },
  nameNeeded: { type: String, required: true },
  remove: { type: String, required: true },
  removeTip: { type: String, default: '' },
  fresh: Boolean,
});
defineEmits(['remove']);
const open = defineModel({ type: Boolean, required: true });
</script>

<template>
  <VueFinalModal
    v-model="open"
    class="confirm-modal"
    content-class="confirm-modal-content edit-dialog"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
    :focus-trap="{ initialFocus: fresh ? '.edit-name' : '.edit-foot .primary' }"
  >
    <div class="edit-head">
      <h2 class="setting-title">{{ title }}</h2>
      <div class="edit-name-row">
        <input v-model="item.name" class="edit-name" :maxlength="MAX_NAME" :aria-label="nameLabel" :aria-invalid="!item.name.trim()" />
        <slot name="aside" />
      </div>
      <p v-if="!item.name.trim()" v-reveal class="field-error" role="alert">{{ nameNeeded }}</p>
    </div>
    <slot />
    <div class="edit-foot">
      <DisabledTip :tip="removeTip">
        <button class="ghost danger" :disabled="!!removeTip" @click="$emit('remove')">{{ remove }}</button>
      </DisabledTip>
      <button class="primary" @click="open = false">{{ t('common.close') }}</button>
    </div>
  </VueFinalModal>
</template>

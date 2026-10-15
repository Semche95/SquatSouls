<script setup>
import { VueFinalModal } from 'vue-final-modal';
import { t } from '../i18n.js';

// Confirmation over the window: `keep`, Escape or a click outside closes it, `confirm` closes it and emits `confirm`.
// Red for a loss, gold (`validate`) for an action that loses nothing.
defineProps({
  title: { type: String, default: '' },
  text: { type: String, required: true },
  confirm: { type: String, required: true },
  keep: { type: String, required: true },
  validate: Boolean,
});
const emit = defineEmits(['confirm']);
const open = defineModel({ type: Boolean, required: true });

function confirmed() {
  open.value = false;
  emit('confirm');
}
</script>

<template>
  <VueFinalModal
    v-model="open"
    class="confirm-modal"
    :content-class="['confirm-modal-content', { validate }]"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
  >
    <h3 v-if="title">{{ title }}</h3>
    <p>{{ text }}</p>
    <div class="card-actions">
      <button class="ghost" @click="open = false">{{ keep }} <kbd>{{ t('keys.escape') }}</kbd></button>
      <button :class="validate ? 'primary' : 'danger'" @click="confirmed">{{ confirm }}</button>
    </div>
  </VueFinalModal>
</template>

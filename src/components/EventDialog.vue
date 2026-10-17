<script setup>
import { t } from '../i18n.js';
import CountStepper from './CountStepper.vue';
import EditDialog from './EditDialog.vue';
import EditRow from './EditRow.vue';

// Edit dialog of an event of the draft: exercises required, limit per session, jokers earned.
defineProps({ ev: { type: Object, required: true }, fresh: Boolean });
defineEmits(['remove']);
const open = defineModel({ type: Boolean, required: true });
</script>

<template>
  <EditDialog
    v-model="open"
    :item="ev"
    :title="t('config.event.title')"
    :name-label="t('config.event.name')"
    :name-needed="t('config.event.nameNeeded')"
    :remove="t('config.event.remove')"
    :fresh="fresh"
    @remove="$emit('remove')"
  >
    <EditRow :label="t('config.event.draws')" :hint="t('config.event.drawsHint')" for-id="ev-draws">
      <CountStepper id="ev-draws" v-model="ev.draws" :min="1" />
    </EditRow>
    <EditRow :label="t('config.event.limit')" :hint="t('config.event.limitHint')" for-id="ev-limit">
      <CountStepper id="ev-limit" v-model="ev.maxPerSession" optional :min="1" :placeholder="t('config.event.noLimit')" />
    </EditRow>
    <EditRow :label="t('config.event.jokers')" :hint="t('config.event.jokersHint')" for-id="ev-jokers">
      <CountStepper id="ev-jokers" v-model="ev.jokers" />
    </EditRow>
  </EditDialog>
</template>

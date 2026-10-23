<script setup>
import { VueFinalModal } from 'vue-final-modal';
import { BOUNDS } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { useConfig } from '../composables/useConfig.js';
import CountStepper from './CountStepper.vue';
import EditRow from './EditRow.vue';

// Settings shared by every exercise, in a dialog opened from the Exercises sub-tab: the changes wait for Save.
const open = defineModel({ type: Boolean });
const { draft } = useConfig();
</script>

<template>
  <VueFinalModal
    v-model="open"
    class="confirm-modal"
    content-class="confirm-modal-content edit-dialog"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
    :focus-trap="{ initialFocus: '.edit-foot .primary' }"
  >
    <div class="edit-head">
      <h2 class="setting-title">{{ t('config.exercises.settings') }}</h2>
    </div>
    <EditRow :label="t('config.exercises.recovery')" :hint="t('config.exercises.recoveryHint')" for-id="cfg-recovery">
      <CountStepper id="cfg-recovery" v-model="draft.recovery" :bounds="BOUNDS.seconds" :unit="t('fields.secondsUnit')" />
    </EditRow>
    <EditRow :label="t('config.exercises.repDuration')" :hint="t('config.exercises.repDurationHint')" for-id="cfg-rep-duration">
      <CountStepper id="cfg-rep-duration" v-model="draft.repDuration" :bounds="BOUNDS.repDuration" :unit="t('fields.secondsUnit')" />
    </EditRow>
    <div class="edit-foot">
      <button class="primary" @click="open = false">{{ t('common.close') }}</button>
    </div>
  </VueFinalModal>
</template>

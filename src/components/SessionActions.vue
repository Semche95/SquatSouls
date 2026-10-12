<script setup>
import { computed, ref } from 'vue';
import { t } from '../i18n.js';
import { useSession } from '../composables/useSession.js';
import DisabledTip from './DisabledTip.vue';

const { session, startSession, endSession, cancelSession } = useSession();

// "" or the session action awaiting confirmation: "end" or "cancel".
const confirmSession = ref('');
// Ending only makes sense with at least one set done and none pending.
const canEnd = computed(() => session.value.sets > 0 && !session.value.pending.length);
// Nothing to cancel until the session has started.
const started = computed(() => !!session.value.startedAt);

function confirmed(action) {
  confirmSession.value = '';
  action();
}
</script>

<template>
  <div v-if="!confirmSession" class="session-actions">
    <button v-if="!started" class="primary" @click="startSession">{{ t('session.start') }}</button>
    <DisabledTip v-else :tip="canEnd ? '' : t(session.pending.length ? 'session.pendingSets' : 'session.noSets')">
      <button class="primary" :disabled="!canEnd" @click="confirmSession = 'end'">{{ t('session.end') }}</button>
    </DisabledTip>
    <DisabledTip :tip="started ? '' : t('session.nothingToCancel')">
      <button class="ghost danger" :disabled="!started" @click="confirmSession = 'cancel'">{{ t('session.cancel') }}</button>
    </DisabledTip>
  </div>
  <div v-else-if="confirmSession === 'end'" class="banner confirm validate session-actions">
    <p>{{ t('session.confirmEnd') }}</p>
    <div class="card-actions">
      <button class="primary" @click="confirmed(endSession)">{{ t('session.endConfirmed') }}</button>
      <button class="ghost" @click="confirmSession = ''">{{ t('common.cancel') }}</button>
    </div>
  </div>
  <div v-else class="banner confirm session-actions">
    <p>{{ t('session.confirmCancel') }}</p>
    <div class="card-actions">
      <button class="danger" @click="confirmed(cancelSession)">{{ t('session.cancel') }}</button>
      <button class="ghost" @click="confirmSession = ''">{{ t('session.keep') }}</button>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue';
import { t } from '../i18n.js';
import { useSession } from '../composables/useSession.js';
import DisabledTip from './DisabledTip.vue';

const { session, newSession, cancelSession } = useSession();

// "" or the session action awaiting confirmation: "end" or "cancel".
const confirmSession = ref('');
// Ending only makes sense with at least one set done and none pending.
const canEnd = computed(() => session.value.sets > 0 && !session.value.pending.length);
// Nothing to cancel until an event has been triggered.
const canCancel = computed(() => !!session.value.gameId);

function confirmed(action) {
  confirmSession.value = '';
  action();
}
</script>

<template>
  <div v-if="!confirmSession" class="session-actions">
    <DisabledTip :tip="canEnd ? '' : t(session.pending.length ? 'session.pendingSets' : 'session.noSets')">
      <button class="primary" :disabled="!canEnd" @click="confirmSession = 'end'">{{ t('session.end') }}</button>
    </DisabledTip>
    <DisabledTip :tip="canCancel ? '' : t('session.nothingToCancel')">
      <button class="ghost danger" :disabled="!canCancel" @click="confirmSession = 'cancel'">{{ t('session.cancel') }}</button>
    </DisabledTip>
  </div>
  <div v-else-if="confirmSession === 'end'" class="banner confirm validate session-actions">
    <p>{{ t('session.confirmEnd') }}</p>
    <div class="card-actions">
      <button class="primary" @click="confirmed(newSession)">{{ t('session.endConfirmed') }}</button>
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

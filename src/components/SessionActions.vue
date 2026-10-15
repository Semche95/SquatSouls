<script setup>
import { computed, ref } from 'vue';
import { rt, t, tm } from '../i18n.js';
import { useSession } from '../composables/useSession.js';
import ConfirmDialog from './ConfirmDialog.vue';
import DisabledTip from './DisabledTip.vue';

const { session, farewell, startSession, endSession, cancelSession } = useSession();

// Session action awaiting confirmation.
const confirmEnd = ref(false);
const confirmCancel = ref(false);
// Ending only makes sense with at least one set done and none pending.
const canEnd = computed(() => session.value.sets > 0 && !session.value.pending.length);
// Nothing to cancel until the session has started.
const started = computed(() => !!session.value.startedAt);
const farewellText = computed(() => (farewell.value < 0 ? '' : rt(tm('session.farewells')[farewell.value])));
</script>

<template>
  <div class="session-actions">
    <p v-if="farewellText" class="farewell">{{ farewellText }}</p>
    <button v-if="!started" class="primary" @click="startSession">{{ t('session.start') }}</button>
    <DisabledTip v-else :tip="canEnd ? '' : t(session.pending.length ? 'session.pendingSets' : 'session.noSets')">
      <button class="primary" :disabled="!canEnd" @click="confirmEnd = true">{{ t('session.end') }}</button>
    </DisabledTip>
    <DisabledTip :tip="started ? '' : t('session.nothingToCancel')">
      <button class="ghost danger" :disabled="!started" @click="confirmCancel = true">{{ t('session.cancel') }}</button>
    </DisabledTip>
  </div>
  <ConfirmDialog
    v-model="confirmEnd"
    :text="t('session.confirmEnd')"
    :confirm="t('session.endConfirmed')"
    :keep="t('common.cancel')"
    validate
    @confirm="endSession"
  />
  <ConfirmDialog
    v-model="confirmCancel"
    :text="t('session.confirmCancel')"
    :confirm="t('session.cancel')"
    :keep="t('session.keep')"
    @confirm="cancelSession"
  />
</template>

<script setup>
import { computed, ref } from 'vue';
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
    <DisabledTip :tip="canEnd ? '' : session.pending.length ? 'Des séries sont encore à faire' : 'Aucune série faite'">
      <button class="primary" :disabled="!canEnd" @click="confirmSession = 'end'">Terminer la séance</button>
    </DisabledTip>
    <DisabledTip :tip="canCancel ? '' : 'Rien à annuler'">
      <button class="ghost danger" :disabled="!canCancel" @click="confirmSession = 'cancel'">Annuler la séance</button>
    </DisabledTip>
  </div>
  <div v-else-if="confirmSession === 'end'" class="banner confirm validate session-actions">
    <p>
      Terminer la séance ? Une nouvelle commence aussitôt : compteurs (dont la limite de morts) et
      jokers repartent de zéro.
      L'historique est conservé.
    </p>
    <div class="card-actions">
      <button class="primary" @click="confirmed(newSession)">Terminer</button>
      <button class="ghost" @click="confirmSession = ''">Annuler</button>
    </div>
  </div>
  <div v-else class="banner confirm session-actions">
    <p>
      Annuler la séance ? Les compteurs (dont la limite de morts), les séries à faire et les
      séries faites seront effacés, et ses séries retirées de l'historique.
    </p>
    <div class="card-actions">
      <button class="danger" @click="confirmed(cancelSession)">Annuler la séance</button>
      <button class="ghost" @click="confirmSession = ''">Garder la séance</button>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue';
import { formatDay, formatDuration, formatTime, plural } from '../format.js';
import { useApi } from '../composables/useApi.js';
import { useSession } from '../composables/useSession.js';
import { useKeydown } from '../composables/useKeydown.js';

const { call } = useApi();
const { session } = useSession();

const history = ref([]);
// Historique complet, affiché en grand par-dessus le Bilan (null quand il est fermé).
const fullHistory = ref(null);
const jokersSpent = computed(() => session.value.skipped.reduce((sum, k) => sum + k.jokers, 0));

async function openFullHistory() {
  try {
    fullHistory.value = await call('get_full_history');
  } catch {
    /* erreur déjà affichée */
  }
}

useKeydown((e) => {
  if (fullHistory.value && e.code === 'Escape') fullHistory.value = null;
});

onMounted(async () => {
  try {
    history.value = await call('get_history', { days: 7 });
  } catch {
    history.value = [];
  }
});
</script>

<template>
  <main class="panel bilan">
    <section class="group current">
      <header class="group-head">
        <h2 class="group-title">Séance actuelle</h2>
        <span class="muted">en cours depuis {{ formatTime(session.startedAt) }}</span>
      </header>
      <p v-if="!session.sets" class="empty">Aucune série faite pendant cette séance.</p>
      <div v-else class="session-detail">
        <div class="session-totals">
          <h2>Totaux</h2>
          <ul class="totals">
            <li v-for="t in session.totals" :key="t.exercise + t.unit">
              <span class="total-amount">{{ t.amount }} <small>{{ t.unit }}</small></span>
              <strong>{{ t.exercise }}</strong>
              <span class="muted">{{ plural(t.sets, 'série') }}</span>
            </li>
          </ul>
        </div>
        <div>
          <h2>Séries faites</h2>
          <ul class="log">
            <li v-for="(e, i) in session.log" :key="i">
              <span class="muted">{{ formatTime(e.at) }}</span>
              <span>{{ e.amount }} {{ e.unit }} · {{ e.exercise }}</span>
              <span class="muted">{{ e.event }}</span>
            </li>
          </ul>
        </div>
      </div>
      <div v-if="session.jokersEarned || session.skipped.length">
        <h2>Jokers</h2>
        <p class="muted">
          {{ session.jokersEarned }} gagné{{ session.jokersEarned > 1 ? 's' : '' }} ·
          {{ jokersSpent }} utilisé{{ jokersSpent > 1 ? 's' : '' }}
        </p>
        <ul v-if="session.skipped.length" class="log">
          <li v-for="(k, i) in session.skipped" :key="i">
            <span class="muted">{{ formatTime(k.at) }}</span>
            <span>{{ k.items.map((it) => `${it.amount} ${it.unit} · ${it.name}`).join(', ') }}</span>
            <span class="muted">{{ k.event }} ({{ k.halved ? 'moitié' : 'sauté' }})</span>
          </li>
        </ul>
      </div>
    </section>

    <section class="group" :class="{ full: fullHistory }">
      <header class="group-head">
        <h2 class="group-title">{{ fullHistory ? 'Historique complet' : 'Historique' }}</h2>
        <span v-if="fullHistory" class="muted">{{ plural(fullHistory.length, 'jour') }} d'activité</span>
        <span v-else class="muted">7 derniers jours, par séance</span>
        <button v-if="fullHistory" class="ghost" @click="fullHistory = null">Fermer</button>
        <button v-else class="ghost" @click="openFullHistory">Tout voir</button>
      </header>
      <p v-if="fullHistory && !fullHistory.length" class="muted">Aucune série enregistrée pour l'instant.</p>
      <ul class="days">
        <li v-for="d in fullHistory ?? history" :key="d.date" :class="{ off: !d.sets }">
          <span class="day">{{ formatDay(d.date) }}</span>
          <span v-if="!d.sets" class="muted">repos</span>
          <span v-else class="day-body">
            <span v-for="se in d.sessions" :key="se.start + se.game" class="day-session">
              <span class="day-games">{{ formatTime(se.start) }} · {{ se.game }} · {{ formatDuration(se.duration) }}</span>
              <span class="day-totals">
                <span v-for="t in se.totals" :key="t.exercise + t.unit" class="chip">
                  {{ t.exercise }} {{ t.amount }} {{ t.unit }}
                </span>
              </span>
            </span>
          </span>
        </li>
      </ul>
    </section>
  </main>
</template>

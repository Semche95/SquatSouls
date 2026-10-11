<script setup>
import { computed } from 'vue';
import { plural } from '../format.js';
import { useSession } from '../composables/useSession.js';
import DisabledTip from './DisabledTip.vue';

// current : série en tête de file, la seule pilotable au clavier.
const props = defineProps({ draw: Object, current: Boolean });

const { session, pauseLeft, confirmDismiss, dismiss, dismissNow, remaining, canAfford, setSkipCost, complete, halve, skipExercise, skipSet } = useSession();
// Exercice visé par les raccourcis : le premier restant de la série en cours.
const keyed = computed(() => props.current && remaining(props.draw)[0]);
</script>

<template>
  <article class="card" :class="{ current }">
    <header class="card-head">
      <span>{{ draw.eventName }}</span>
      <span class="muted">{{ draw.gameName }}</span>
    </header>
    <p v-if="draw.jokers" class="reward">+{{ plural(draw.jokers, 'joker') }}</p>
    <ul class="items">
      <li v-for="it in draw.items" :key="it.exerciseId" :class="{ done: it.done }">
        <span class="amount">{{ it.amount }}</span>
        <span class="what">
          <strong>{{ it.name }}</strong>
          <span class="muted">{{ it.unit }}<template v-if="it.halved"> · moitié</template><template v-if="it.done"> · fait</template></span>
        </span>
        <span v-if="!it.done" class="item-actions">
          <DisabledTip :tip="pauseLeft ? 'En pause' : ''">
            <button class="primary" :disabled="!!pauseLeft" @click="complete(draw, it)">
              Fait <kbd v-if="it === keyed" class="dark">Entrée</kbd>
            </button>
          </DisabledTip>
          <template v-if="session.jokers">
            <DisabledTip v-if="it.halveCost" :tip="plural(it.halveCost, 'joker')">
              <button class="ghost" :disabled="!canAfford(it.halveCost)" @click="halve(draw, it)">
                Moitié <kbd v-if="it === keyed">H</kbd>
              </button>
            </DisabledTip>
            <DisabledTip :tip="plural(it.skipCost, 'joker')">
              <button class="ghost" :disabled="!canAfford(it.skipCost)" @click="skipExercise(draw, it)">
                Sauter <kbd v-if="it === keyed">J</kbd>
              </button>
            </DisabledTip>
          </template>
        </span>
      </li>
    </ul>
    <footer v-if="confirmDismiss === draw.id" class="banner confirm">
      <p>Annuler la série ? Ses exercices déjà faits ne seront pas comptés et seront retirés de l'historique.</p>
      <div class="card-actions">
        <button class="danger" @click="dismissNow(draw)">Annuler la série</button>
        <button class="ghost" @click="confirmDismiss = 0">Garder la série <kbd>Échap</kbd></button>
      </div>
    </footer>
    <footer v-else class="card-actions">
      <button v-if="session.jokers && remaining(draw).length > 1" class="ghost" :disabled="!canAfford(setSkipCost(draw))" @click="skipSet(draw)">
        Sauter la série ({{ plural(setSkipCost(draw), 'joker') }})
      </button>
      <button class="ghost" @click="dismiss(draw)">
        Annuler <kbd v-if="current">Retour arr.</kbd>
      </button>
    </footer>
  </article>
</template>

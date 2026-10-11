<script setup>
import { plural } from '../format.js';
import { useSession } from '../composables/useSession.js';

// current : série en tête de file, la seule pilotable au clavier.
defineProps({ draw: Object, current: Boolean });

const { session, pauseLeft, act, canAfford, setSkipCost, halve, skipExercise, skipSet } = useSession();
</script>

<template>
  <article class="card" :class="{ current }">
    <header class="card-head">
      <span>{{ draw.eventName }}</span>
      <span class="muted">{{ draw.gameName }}</span>
    </header>
    <p v-if="draw.jokers" class="reward">+{{ plural(draw.jokers, 'joker') }}</p>
    <ul class="items">
      <li v-for="(it, i) in draw.items" :key="it.exerciseId">
        <span class="amount">{{ it.amount }}</span>
        <span class="what">
          <strong>{{ it.name }}</strong>
          <span class="muted">{{ it.unit }}<template v-if="it.halved"> · moitié</template></span>
        </span>
        <span v-if="session.jokers" class="item-jokers">
          <button v-if="it.halveCost" class="ghost" :disabled="!canAfford(it.halveCost)" :title="plural(it.halveCost, 'joker')" @click="halve(draw, it)">
            Moitié <kbd v-if="current && !i">H</kbd>
          </button>
          <button class="ghost" :disabled="!canAfford(it.skipCost)" :title="plural(it.skipCost, 'joker')" @click="skipExercise(draw, it)">
            Sauter <kbd v-if="current && !i">J</kbd>
          </button>
        </span>
      </li>
    </ul>
    <footer class="card-actions">
      <button class="primary" :disabled="!!pauseLeft" :title="pauseLeft ? 'En pause' : ''" @click="act('complete', draw)">
        Fait <kbd v-if="current" class="dark">Entrée</kbd>
      </button>
      <button v-if="session.jokers && draw.items.length > 1" class="ghost" :disabled="!canAfford(setSkipCost(draw))" @click="skipSet(draw)">
        Sauter la série ({{ plural(setSkipCost(draw), 'joker') }})
      </button>
      <button class="ghost" @click="act('dismiss', draw)">
        Annuler <kbd v-if="current">Retour arr.</kbd>
      </button>
    </footer>
  </article>
</template>

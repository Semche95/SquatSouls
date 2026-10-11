<script setup>
import { plural } from "../format.js";
import { useSession } from "../composables/useSession.js";

// current : série en tête de file, la seule pilotable au clavier.
defineProps({ draw: Object, current: Boolean });

const { session, act } = useSession();
</script>

<template>
  <article class="card" :class="{ current }">
    <header class="card-head">
      <span>{{ draw.eventName }}</span>
      <span class="muted">{{ draw.gameName }}</span>
    </header>
    <p v-if="draw.jokers" class="reward">+{{ plural(draw.jokers, "joker") }}</p>
    <ul class="items">
      <li v-for="it in draw.items" :key="it.exerciseId">
        <span class="amount">{{ it.amount }}</span>
        <span class="what">
          <strong>{{ it.name }}</strong>
          <span class="muted">{{ it.unit }}</span>
        </span>
      </li>
    </ul>
    <footer class="card-actions">
      <button class="primary" @click="act('complete', draw)">
        Fait <kbd v-if="current" class="dark">Entrée</kbd>
      </button>
      <button v-if="session.jokers" class="ghost" @click="act('use_joker', draw)">
        Joker <kbd v-if="current">J</kbd>
      </button>
      <button class="ghost" @click="act('dismiss', draw)">
        Annuler <kbd v-if="current">Retour arr.</kbd>
      </button>
    </footer>
  </article>
</template>

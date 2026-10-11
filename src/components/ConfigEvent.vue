<script setup>
import { plural } from '../format.js';
import CountInput from './CountInput.vue';

// ev belongs to the draft: it is edited in place.
defineProps({ ev: Object });
defineEmits(['remove']);
</script>

<template>
  <details class="cfg-event">
    <summary>
      <span class="cfg-event-head">
        <strong>{{ ev.name || '(sans nom)' }}</strong>
        <span class="muted">
          {{ plural(ev.draws, 'exercice') }} par série<template v-if="ev.maxPerSession"> · {{ ev.maxPerSession }} fois max par séance</template>
        </span>
        <span v-if="ev.jokers" class="reward-tag">+{{ plural(ev.jokers, 'joker') }}</span>
      </span>
    </summary>
    <div class="field">
      <span>Nom</span>
      <input v-model="ev.name" aria-label="Nom de l'événement" />
      <button class="ghost danger" @click="$emit('remove')">Supprimer</button>
    </div>
    <div class="cfg-params">
      <label>
        <span>Exercices tirés</span>
        <CountInput v-model="ev.draws" />
      </label>
      <label>
        <span>Limite par séance</span>
        <CountInput v-model="ev.maxPerSession" optional placeholder="aucune" />
      </label>
      <label>
        <span>Jokers gagnés</span>
        <CountInput v-model="ev.jokers" />
      </label>
    </div>
  </details>
</template>

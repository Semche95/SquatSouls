<script setup>
import { plural } from '../format.js';
import { useConfig } from '../composables/useConfig.js';

// ev appartient au brouillon : on l'édite en place.
defineProps({ ev: Object });
defineEmits(['remove']);

const { draft } = useConfig();

function setMaxPerSession(ev, value) {
  if (value === '') delete ev.maxPerSession;
  else ev.maxPerSession = Number(value);
}
</script>

<template>
  <details class="cfg-event">
    <summary>
      <span class="cfg-event-head">
        <strong>{{ ev.name || '(sans nom)' }}</strong>
        <span class="muted">
          tire {{ ev.draws }}<template v-if="ev.maxPerSession"> · {{ ev.maxPerSession }} par séance</template>
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
        <input v-model.number="ev.draws" type="number" @change="ev.draws = Number(ev.draws) || 0" min="1" :max="draft.exercises.length" />
      </label>
      <label>
        <span>Limite par séance</span>
        <input
          :value="ev.maxPerSession ?? ''"
          type="number"
          min="1"
          placeholder="aucune"
          @input="setMaxPerSession(ev, $event.target.value)"
        />
      </label>
      <label>
        <span>Jokers gagnés</span>
        <input v-model.number="ev.jokers" type="number" @change="ev.jokers = Number(ev.jokers) || 0" min="0" />
      </label>
    </div>
  </details>
</template>

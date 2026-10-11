<script setup>
import { newId } from '../configDraft.js';
import { useConfig } from '../composables/useConfig.js';

const { draft } = useConfig();

function addExercise() {
  const list = draft.value.exercises;
  list.push({ id: newId('Nouvel exercice', list), name: 'Nouvel exercice', unit: 'rép.', min: 5, max: 10 });
}
</script>

<template>
  <section class="group cfg-exercises">
    <h2 class="group-title">Exercices</h2>
    <table class="cfg-table">
      <thead>
        <tr><th>Nom</th><th>Unité</th><th>Min</th><th>Max</th><th></th></tr>
      </thead>
      <tbody>
        <tr v-for="(ex, i) in draft.exercises" :key="ex.id">
          <td><input v-model="ex.name" aria-label="Nom de l'exercice" /></td>
          <td class="cfg-unit"><input v-model="ex.unit" aria-label="Unité" placeholder="rép., s…" /></td>
          <td><input v-model.number="ex.min" type="number" @change="ex.min = Number(ex.min) || 0" min="0" aria-label="Minimum" /></td>
          <td><input v-model.number="ex.max" type="number" @change="ex.max = Number(ex.max) || 0" min="1" aria-label="Maximum" /></td>
          <td>
            <button
              class="ghost icon"
              :disabled="draft.exercises.length < 2"
              title="Supprimer"
              @click="draft.exercises.splice(i, 1)"
            >✕</button>
          </td>
        </tr>
      </tbody>
    </table>
    <button class="ghost" @click="addExercise">Ajouter un exercice</button>
  </section>
</template>

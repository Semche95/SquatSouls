<script setup>
import { ref } from 'vue';
import { newId } from '../configDraft.js';
import { plural } from '../format.js';
import { mergeExercises, parseExercises, shareExercises } from '../share.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import CountInput from './CountInput.vue';
import ShareImport from './ShareImport.vue';

const { config, draft, notify, copy } = useConfig();
const { saveNow } = useSession();
const importing = ref(false);

// L'import validé s'enregistre aussitôt ; le brouillon le reçoit sans perdre ses autres modifications.
async function importShared(imported) {
  const { exercises, added, updated } = mergeExercises(config.value.exercises, imported);
  if (!(await saveNow({ ...config.value, exercises }))) return;
  draft.value.exercises = mergeExercises(draft.value.exercises, imported).exercises;
  importing.value = false;
  notify(`Exercices importés : ${plural(added, 'ajouté')}, ${updated} mis à jour.`);
}

function addExercise() {
  const list = draft.value.exercises;
  list.push({ id: newId('Nouvel exercice', list), name: 'Nouvel exercice', enabled: true, unit: 'rép.', min: 5, max: 10 });
}
</script>

<template>
  <section class="group cfg-exercises">
    <header class="group-head">
      <h2 class="group-title">Exercices</h2>
      <button class="ghost" @click="copy(shareExercises(config.exercises))">Partager</button>
      <button class="ghost" :disabled="importing" @click="importing = true">Importer</button>
    </header>
    <ShareImport v-if="importing" :parse="parseExercises" @import="importShared" @cancel="importing = false" />
    <label class="field cfg-pause">
      <span>Pause après chaque série (s)</span>
      <CountInput v-model="draft.pause" />
    </label>
    <table class="cfg-table">
      <thead>
        <tr><th></th><th>Nom</th><th>Unité</th><th>Min</th><th>Max</th><th>Pause (s)</th><th></th></tr>
      </thead>
      <tbody>
        <tr v-for="(ex, i) in draft.exercises" :key="ex.id" :class="{ 'cfg-off': !ex.enabled }">
          <td><input v-model="ex.enabled" type="checkbox" aria-label="Tirer cet exercice" /></td>
          <td class="cfg-name"><input v-model="ex.name" aria-label="Nom de l'exercice" /></td>
          <td class="cfg-unit"><input v-model="ex.unit" aria-label="Unité" placeholder="rép., s…" /></td>
          <td><CountInput v-model="ex.min" aria-label="Minimum" /></td>
          <td><CountInput v-model="ex.max" aria-label="Maximum" /></td>
          <td><CountInput v-model="ex.pause" optional :placeholder="String(draft.pause)" aria-label="Pause" /></td>
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

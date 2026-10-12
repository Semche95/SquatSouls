<script setup>
import { ref } from 'vue';
import { newId } from '../configDraft.js';
import { t } from '../i18n.js';
import { mergeExercises, parseExercises, shareExercises } from '../share.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import CountInput from './CountInput.vue';
import ShareImport from './ShareImport.vue';

const { config, draft, notify, copy } = useConfig();
const { saveNow } = useSession();
const importing = ref(false);

// The confirmed import is saved right away; the draft receives it without losing its other changes.
async function importShared(imported) {
  const { exercises, added, updated } = mergeExercises(config.value.exercises, imported);
  if (!(await saveNow({ ...config.value, exercises }))) return;
  draft.value.exercises = mergeExercises(draft.value.exercises, imported).exercises;
  importing.value = false;
  notify(t('config.exercises.imported', { added: t('config.exercises.added', added), updated }));
}

function addExercise() {
  const list = draft.value.exercises;
  const name = t('config.exercises.new');
  list.push({ id: newId(name, list), name, enabled: true, unit: t('config.exercises.defaultUnit'), min: 5, max: 10 });
}
</script>

<template>
  <section class="group cfg-exercises">
    <header class="group-head">
      <h2 class="group-title">{{ t('config.tabs.exercises') }}</h2>
      <button class="ghost" @click="copy(shareExercises(config.exercises))">{{ t('common.share') }}</button>
      <button class="ghost" :disabled="importing" @click="importing = true">{{ t('common.import') }}</button>
    </header>
    <ShareImport v-if="importing" :parse="parseExercises" @import="importShared" @cancel="importing = false" />
    <label class="field cfg-pause">
      <span>{{ t('config.exercises.pause') }}</span>
      <CountInput v-model="draft.pause" />
    </label>
    <table class="cfg-table">
      <thead>
        <tr>
          <th></th>
          <th>{{ t('common.name') }}</th>
          <th>{{ t('config.exercises.unit') }}</th>
          <th>{{ t('config.exercises.min') }}</th>
          <th>{{ t('config.exercises.max') }}</th>
          <th>{{ t('config.exercises.pauseColumn') }}</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(ex, i) in draft.exercises" :key="ex.id" :class="{ 'cfg-off': !ex.enabled }">
          <td><input v-model="ex.enabled" type="checkbox" :aria-label="t('config.exercises.enabled')" /></td>
          <td class="cfg-name"><input v-model="ex.name" :aria-label="t('config.exercises.name')" /></td>
          <td class="cfg-unit"><input v-model="ex.unit" :aria-label="t('config.exercises.unit')" :placeholder="t('config.exercises.unitPlaceholder')" /></td>
          <td><CountInput v-model="ex.min" :aria-label="t('config.exercises.minimum')" /></td>
          <td><CountInput v-model="ex.max" :aria-label="t('config.exercises.maximum')" /></td>
          <td><CountInput v-model="ex.pause" optional :placeholder="String(draft.pause)" :aria-label="t('config.exercises.pauseLabel')" /></td>
          <td>
            <button
              class="ghost icon"
              :disabled="draft.exercises.length < 2"
              :title="t('common.remove')"
              @click="draft.exercises.splice(i, 1)"
            >✕</button>
          </td>
        </tr>
      </tbody>
    </table>
    <button class="ghost" @click="addExercise">{{ t('config.exercises.add') }}</button>
  </section>
</template>

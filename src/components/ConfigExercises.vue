<script setup>
import { ref } from 'vue';
import { VueFinalModal } from 'vue-final-modal';
import { newId } from '../configDraft.js';
import { t } from '../i18n.js';
import { mergeExercises, parseExercises, shareExercises } from '../share.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import ConfigExercise from './ConfigExercise.vue';
import CountInput from './CountInput.vue';
import ShareImport from './ShareImport.vue';

const { config, draft, notify, copy } = useConfig();
const { saveNow } = useSession();
const importing = ref(false);
// Exercise just added, opened to type its name.
const freshId = ref(null);

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
  const id = newId(name, list);
  list.push({ id, name, enabled: true, unit: t('config.exercises.defaultUnit'), min: 5, max: 10 });
  freshId.value = id;
}
</script>

<template>
  <section class="group cfg-exercises">
    <header class="group-head">
      <h2 class="group-title">{{ t('config.tabs.exercises') }}</h2>
      <button class="ghost" @click="copy(shareExercises(config.exercises))">{{ t('common.share') }}</button>
      <button class="ghost" @click="importing = true">{{ t('common.import') }}</button>
    </header>
    <VueFinalModal
      v-model="importing"
      class="confirm-modal"
      content-class="confirm-modal-content form-dialog"
      overlay-transition="vfm-fade"
      content-transition="vfm-fade"
      :focus-trap="{ initialFocus: '.form-dialog textarea' }"
    >
      <button class="modal-close" :aria-label="t('common.close')" @click="importing = false">×</button>
      <h3>{{ t('config.exercises.import') }}</h3>
      <ShareImport :parse="parseExercises" @import="importShared" @cancel="importing = false" />
    </VueFinalModal>
    <label class="field cfg-recovery">
      <span>{{ t('config.exercises.recovery') }}</span>
      <CountInput v-model="draft.recovery" />
    </label>
    <div>
      <ConfigExercise
        v-for="(ex, i) in draft.exercises"
        :key="ex.id"
        :ex="ex"
        :recovery="draft.recovery"
        :removable="draft.exercises.length > 1"
        :fresh="ex.id === freshId"
        @remove="draft.exercises.splice(i, 1)"
      />
    </div>
    <button class="ghost" @click="addExercise">{{ t('config.exercises.add') }}</button>
  </section>
</template>

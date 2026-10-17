<script setup>
import { ref } from 'vue';
import { VueFinalModal } from 'vue-final-modal';
import { newId } from '../configDraft.js';
import { t } from '../i18n.js';
import { mergeExercises, parseExercises, shareExercises } from '../share.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useEditDialog } from '../composables/useEditDialog.js';
import ConfigExercise from './ConfigExercise.vue';
import CountStepper from './CountStepper.vue';
import EditRow from './EditRow.vue';
import ExerciseDialog from './ExerciseDialog.vue';
import ShareImport from './ShareImport.vue';

const { config, draft, notify, copy } = useConfig();
const { saveNow } = useSession();
const importing = ref(false);
// Exercise in its edit dialog.
const { item: editing, open: editOpen, fresh, edit: editExercise, remove: removeExercise } = useEditDialog(() => draft.value.exercises);

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
  editExercise(list.at(-1), true);
}
</script>

<template>
  <section class="group cfg-exercises">
    <header class="group-head">
      <h2 class="group-title">{{ t('config.tabs.exercises') }}</h2>
      <button class="ghost" @click="copy(shareExercises(config.exercises))">{{ t('common.share') }}</button>
      <button @click="importing = true">{{ t('common.import') }}</button>
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
    <div class="setting-card">
      <EditRow :label="t('config.exercises.recovery')" :hint="t('config.exercises.recoveryHint')" for-id="cfg-recovery">
        <CountStepper id="cfg-recovery" v-model="draft.recovery" :unit="t('fields.secondsUnit')" />
      </EditRow>
    </div>
    <div class="tiles">
      <ConfigExercise v-for="ex in draft.exercises" :key="ex.id" :ex="ex" @edit="editExercise(ex)" />
      <button class="tile tile-new" @click="addExercise">{{ t('config.exercises.add') }}</button>
    </div>
    <ExerciseDialog
      v-if="editing"
      v-model="editOpen"
      :ex="editing"
      :recovery="draft.recovery"
      :removable="draft.exercises.length > 1"
      :fresh="fresh"
      @remove="removeExercise"
    />
  </section>
</template>

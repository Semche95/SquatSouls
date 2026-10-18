<script setup>
import { computed, ref, shallowRef, watch } from 'vue';
import { VueFinalModal } from 'vue-final-modal';
import { BOUNDS, clone, newId } from '../configDraft.js';
import { t } from '../i18n.js';
import { parseExercises, shareExercises } from '../share.js';
import { exerciseChanges, mergeExercises } from '../importChanges.js';
import { exerciseSummary } from '../itemSummary.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useEditDialog } from '../composables/useEditDialog.js';
import ConfigExercise from './ConfigExercise.vue';
import CountStepper from './CountStepper.vue';
import DisabledTip from './DisabledTip.vue';
import EditRow from './EditRow.vue';
import ExerciseDialog from './ExerciseDialog.vue';
import ShareImport from './ShareImport.vue';

const { config, draft, notify, copy, shareBlocked } = useConfig();
const { saveNow } = useSession();
const importing = ref(false);
// Saved exercises as the import opened: saving it does not change the preview while the modal fades out.
const savedExercises = shallowRef([]);
watch(importing, (on) => on && (savedExercises.value = config.value.exercises));
// Exercise in its edit dialog.
const { item: editing, open: editOpen, fresh, edit: editExercise, remove: removeExercise } = useEditDialog(() => draft.value.exercises);
// Exercises asked for in Play, out of all those defined.
const included = computed(() => draft.value.exercises.filter((ex) => ex.enabled).length);

// The confirmed import is saved right away, Program having no pending change; the draft receives it.
async function importShared(imported, chosen) {
  const { exercises, added, updated } = mergeExercises(config.value.exercises, imported, chosen);
  if (!(await saveNow({ ...config.value, exercises }))) return;
  draft.value.exercises = clone(exercises);
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
      <div class="cfg-head-text">
        <h2 class="setting-title">{{ t('config.exercises.count', { n: included, total: draft.exercises.length }, included) }}</h2>
        <p class="hint">{{ t('config.exercises.enabledHint') }}</p>
      </div>
      <DisabledTip :tip="shareBlocked">
        <button class="ghost" :disabled="!!shareBlocked" @click="copy(shareExercises(config.exercises))">{{ t('common.share') }}</button>
      </DisabledTip>
      <DisabledTip :tip="shareBlocked">
        <button :disabled="!!shareBlocked" @click="importing = true">{{ t('common.import') }}</button>
      </DisabledTip>
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
      <ShareImport
        :parse="parseExercises"
        :changes="(imported) => exerciseChanges(savedExercises, imported)"
        :summary="exerciseSummary"
        @import="importShared"
        @cancel="importing = false"
      />
    </VueFinalModal>
    <div class="setting-card">
      <EditRow :label="t('config.exercises.recovery')" :hint="t('config.exercises.recoveryHint')" for-id="cfg-recovery">
        <CountStepper id="cfg-recovery" v-model="draft.recovery" :bounds="BOUNDS.seconds" :unit="t('fields.secondsUnit')" />
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
      :fresh="fresh"
      @remove="removeExercise"
    />
  </section>
</template>

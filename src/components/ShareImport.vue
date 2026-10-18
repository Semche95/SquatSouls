<script setup>
import { computed, onMounted, ref, watch } from 'vue';
import { t } from '../i18n.js';
import { vReveal } from '../reveal.js';
import { appliedCount, defaultChoice, differ } from '../importChanges.js';
import { useConfig } from '../composables/useConfig.js';
import DisabledTip from './DisabledTip.vue';
import ImportPreview from './ImportPreview.vue';

// `parse` reads the pasted text and throws an error if it is not importable; `changes` lists what its
// content changes (`importChanges.js`), `summary` sums up an item, `blocked` tells why the import cannot
// go ahead ('' when it can). Shown in a modal: Escape cancels. Emits the content and the chosen changes.
const props = defineProps({
  parse: { type: Function, required: true },
  changes: { type: Function, required: true },
  summary: { type: Function, required: true },
  blocked: { type: Function, default: () => '' },
});
const emit = defineEmits(['import', 'cancel']);
const { shareBlocked } = useConfig();

const text = ref('');
const input = ref(null);
onMounted(() => input.value.focus());

// Content of the pasted text, or the error that prevents importing it.
const result = computed(() => {
  if (!text.value.trim()) return null;
  try {
    return props.parse(text.value);
  } catch (e) {
    return e;
  }
});
const invalid = computed(() => result.value instanceof Error);
const importable = computed(() => !!result.value && !invalid.value);

// Kept as long as its content is the same, so editing what does not change it (the name of a game imported
// as new) leaves the switches as they were.
const list = computed((previous) => {
  const next = importable.value ? props.changes(result.value) : [];
  return previous && !differ(previous, next) ? previous : next;
});
const chosen = ref(new Set());
watch(list, (changes) => (chosen.value = defaultChoice(changes)));

// Why the import cannot go ahead, '' when it can.
const tip = computed(() => {
  if (invalid.value) return result.value.message;
  if (shareBlocked.value) return shareBlocked.value;
  if (!importable.value) return t('share.pasteFirst');
  if (!list.value.length) return t('share.nothing');
  return props.blocked(result.value) || (appliedCount(list.value, chosen.value) ? '' : t('share.chooseOne'));
});
</script>

<template>
  <form class="cfg-import" @submit.prevent="tip || emit('import', result, chosen)">
    <textarea ref="input" v-model="text" rows="5" :aria-label="t('share.text')" :placeholder="t('share.placeholder')" :aria-invalid="invalid"></textarea>
    <p v-if="invalid" v-reveal class="banner error" role="alert">{{ result.message }}</p>
    <template v-else-if="importable">
      <slot :result="result" />
      <ImportPreview v-model:chosen="chosen" :changes="list" :summary="summary" />
    </template>
    <div class="row">
      <DisabledTip :tip="tip">
        <button class="primary" :disabled="!!tip">{{ t('common.import') }}</button>
      </DisabledTip>
      <button type="button" class="ghost" @click="emit('cancel')">{{ t('common.cancel') }} <kbd>{{ t('keys.escape') }}</kbd></button>
    </div>
  </form>
</template>

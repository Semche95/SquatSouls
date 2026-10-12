<script setup>
import { computed, onMounted, ref } from 'vue';
import { t } from '../i18n.js';

// `parse` reads the pasted text and throws an error if it is not importable.
const props = defineProps({ parse: { type: Function, required: true } });
const emit = defineEmits(['import', 'cancel']);

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
</script>

<template>
  <form class="cfg-import" @submit.prevent="emit('import', result)">
    <textarea ref="input" v-model="text" rows="5" :aria-label="t('share.text')" :placeholder="t('share.placeholder')" :aria-invalid="invalid"></textarea>
    <p v-if="invalid" class="banner error" role="alert">{{ result.message }}</p>
    <slot v-else-if="importable" :result="result" />
    <div class="row">
      <button class="primary" :disabled="!importable">{{ t('common.import') }}</button>
      <button type="button" class="ghost" @click="emit('cancel')">{{ t('common.cancel') }}</button>
    </div>
  </form>
</template>

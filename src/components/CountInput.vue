<script setup>
import { computed, ref, watch } from 'vue';
import { vTooltip } from '../tooltip.js';
import { countError, parseCount } from '../configDraft.js';

// Integer field whose format and `bounds` are checked while typing (error in a tooltip).
// Cleared, it becomes 0 when leaving the field, or disappears if it is optional.
const props = defineProps({ modelValue: [Number, String], optional: Boolean, bounds: { type: Object, required: true } });
const emit = defineEmits(['update:modelValue']);

const text = ref(String(props.modelValue ?? ''));
const value = (t) => {
  const v = parseCount(t);
  return v === '' && props.optional ? undefined : v;
};
const error = computed(() => countError(value(text.value), props.bounds));
// The error shows right away while typing; outside the field, only on hover.
const focused = ref(false);

// Value changed elsewhere (reset, import): the field takes it.
watch(
  () => props.modelValue,
  (v) => {
    if (v !== value(text.value)) text.value = String(v ?? '');
  },
);

function onInput(e) {
  text.value = e.target.value;
  emit('update:modelValue', value(text.value));
}

function onChange() {
  if (!props.optional && text.value.trim() === '') emit('update:modelValue', 0);
}
</script>

<template>
  <input
    v-tooltip="{ content: error, disabled: !error, shown: !!error && focused, triggers: ['hover'], placement: 'bottom', popperClass: 'error' }"
    class="count-input"
    :value="text"
    inputmode="numeric"
    :aria-invalid="!!error"
    @input="onInput"
    @change="onChange"
    @focus="focused = true"
    @blur="focused = false"
  />
</template>

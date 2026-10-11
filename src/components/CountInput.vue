<script setup>
import { computed, ref, watch } from 'vue';
import { vTooltip } from 'floating-vue';
import { countError, parseCount } from '../configDraft.js';

// Champ de nombre entier dont le format est vérifié pendant la saisie (erreur en infobulle).
// Vidé, il vaut 0 en quittant le champ, ou disparaît s'il est facultatif.
const props = defineProps({ modelValue: [Number, String], optional: Boolean });
const emit = defineEmits(['update:modelValue']);

const text = ref(String(props.modelValue ?? ''));
const value = (t) => {
  const v = parseCount(t);
  return v === '' && props.optional ? undefined : v;
};
const error = computed(() => countError(value(text.value)));
// L'erreur s'affiche aussitôt pendant la saisie ; hors du champ, seulement au survol.
const focused = ref(false);

// Valeur changée ailleurs (réinitialisation, import) : le champ la reprend.
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

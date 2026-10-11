<script setup>
import { computed, onMounted, ref } from 'vue';

// `parse` lit le texte collé et lève une erreur s'il n'est pas importable.
const props = defineProps({ parse: { type: Function, required: true } });
const emit = defineEmits(['import', 'cancel']);

const text = ref('');
const input = ref(null);
onMounted(() => input.value.focus());

// Contenu du texte collé, ou l'erreur qui empêche de l'importer.
const result = computed(() => {
  if (!text.value.trim()) return null;
  try {
    return props.parse(text.value);
  } catch (e) {
    return e;
  }
});
const importable = computed(() => !!result.value && !(result.value instanceof Error));
</script>

<template>
  <form class="cfg-import" @submit.prevent="emit('import', result)">
    <textarea ref="input" v-model="text" rows="5" aria-label="Texte partagé" placeholder="Coller ici le texte partagé"></textarea>
    <p v-if="result instanceof Error" class="muted">{{ result.message }}</p>
    <slot v-else-if="importable" :result="result" />
    <div class="row">
      <button class="primary" :disabled="!importable">Importer</button>
      <button type="button" class="ghost" @click="emit('cancel')">Annuler</button>
    </div>
  </form>
</template>

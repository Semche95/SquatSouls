<script setup>
// Row of an edit dialog or a Program card: the setting and its hint on the left, its control on the right.
defineProps({ label: String, hint: String, forId: String });
</script>

<template>
  <div class="setting-row check">
    <div>
      <label :for="forId">{{ label }}</label>
      <p v-if="hint" class="hint">{{ hint }}</p>
    </div>
    <slot />
  </div>
</template>

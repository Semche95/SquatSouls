<script setup>
// Games listed in their frame to choose one, filtered by name: each button shows the name, then what the slot adds.
import { computed, ref } from 'vue';
import { byName, nameMatches } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';

const props = defineProps({
  games: { type: Array, required: true },
  label: { type: String, required: true },
  active: { type: Object, default: null },
});
defineEmits(['pick']);
const filter = defineModel('filter', { type: String, default: '' });
const shown = computed(() => byName(props.games).filter((g) => nameMatches(g.name, filter.value)));

const input = ref(null);
defineExpose({ focus: () => input.value.focus() });
</script>

<template>
  <div class="cfg-game-list">
    <input ref="input" v-model="filter" class="cfg-game-filter" :placeholder="t('config.games.filter')" :aria-label="t('config.games.filter')" />
    <nav :aria-label="label">
      <button v-for="g in shown" :key="g.id" type="button" :class="{ active: g === active }" :aria-current="g === active" @click="$emit('pick', g)">
        <span class="cfg-game-name">{{ g.name }}</span>
        <slot :game="g" />
      </button>
      <p v-if="!shown.length" class="cfg-game-none">{{ t('config.games.noMatch') }}</p>
    </nav>
  </div>
</template>

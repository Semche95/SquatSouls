<script setup>
import { computed } from 'vue';
import { t } from '../i18n/i18n.js';
import CountInput from './CountInput.vue';
import DisabledTip from './DisabledTip.vue';

// Number field between its - and + buttons, the unit inside the frame. Its attributes (id, aria-label, placeholder)
// go to the field. - stops at the lowest of `bounds`, or empties an optional field; + stops at the highest
// and, on an empty field, starts from `base`.
defineOptions({ inheritAttrs: false });
const props = defineProps({ unit: String, optional: Boolean, bounds: { type: Object, required: true }, base: { type: Number, default: 0 } });
const model = defineModel({ type: [Number, String] });

// An invalid field counts as empty, or as 0 when it is required.
const value = computed(() => (Number.isInteger(model.value) ? model.value : props.optional ? null : 0));
const lowest = computed(() => (props.optional ? value.value === null : value.value <= props.bounds.min));
const highest = computed(() => value.value !== null && value.value >= props.bounds.max);
// Why - or + is disabled: the field is already at its bound, or past it while badly filled.
const lowTip = computed(() => {
  if (value.value === null) return t('fields.alreadyEmpty');
  return t(value.value < props.bounds.min ? 'fields.belowBound' : 'fields.atBound', { n: props.bounds.min });
});
const highTip = computed(() => t(value.value > props.bounds.max ? 'fields.aboveBound' : 'fields.atBound', { n: props.bounds.max }));

function down() {
  model.value = props.optional && value.value <= props.bounds.min ? undefined : Math.min(props.bounds.max, value.value - 1);
}
function up() {
  model.value = Math.max(props.bounds.min, (value.value ?? props.base) + 1);
}
</script>

<template>
  <span class="stepper">
    <DisabledTip :tip="lowest ? lowTip : ''">
      <button type="button" :aria-label="t('fields.decrease')" :disabled="lowest" @click="down">−</button>
    </DisabledTip>
    <CountInput v-bind="$attrs" v-model="model" :optional="optional" :bounds="bounds" :class="{ alone: !unit }" />
    <span v-if="unit" class="unit">{{ unit }}</span>
    <DisabledTip :tip="highest ? highTip : ''">
      <button type="button" :aria-label="t('fields.increase')" :disabled="highest" @click="up">+</button>
    </DisabledTip>
  </span>
</template>

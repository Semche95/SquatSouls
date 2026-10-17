<script setup>
import { computed } from 'vue';
import { t } from '../i18n.js';
import CountInput from './CountInput.vue';
import DisabledTip from './DisabledTip.vue';

// Number field between its - and + buttons, the unit inside the frame. Its attributes (id, aria-label, placeholder)
// go to the field. - stops at `min`, or empties an optional field; + on an empty field starts from `base`.
defineOptions({ inheritAttrs: false });
const props = defineProps({ unit: String, optional: Boolean, min: { type: Number, default: 0 }, base: { type: Number, default: 0 } });
const model = defineModel({ type: [Number, String] });

// An invalid field counts as empty, or as 0 when it is required.
const value = computed(() => (Number.isInteger(model.value) ? model.value : props.optional ? null : 0));
const lowest = computed(() => (props.optional ? value.value === null : value.value <= props.min));
const lowTip = computed(() => (value.value === null ? t('fields.alreadyEmpty') : t('fields.atMin', { n: props.min })));

function down() {
  model.value = props.optional && value.value <= props.min ? undefined : value.value - 1;
}
function up() {
  model.value = Math.max(props.min, (value.value ?? props.base) + 1);
}
</script>

<template>
  <span class="stepper">
    <DisabledTip :tip="lowest ? lowTip : ''">
      <button type="button" :aria-label="t('fields.decrease')" :disabled="lowest" @click="down">−</button>
    </DisabledTip>
    <CountInput v-bind="$attrs" v-model="model" :optional="optional" :class="{ alone: !unit }" />
    <span v-if="unit" class="unit">{{ unit }}</span>
    <button type="button" :aria-label="t('fields.increase')" @click="up">+</button>
  </span>
</template>

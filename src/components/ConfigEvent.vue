<script setup>
import { MAX_NAME } from '../configDraft.js';
import { t } from '../i18n.js';
import CountInput from './CountInput.vue';

// ev belongs to the draft: it is edited in place.
defineProps({ ev: Object });
defineEmits(['remove']);
</script>

<template>
  <details class="cfg-event">
    <summary>
      <span class="cfg-event-head">
        <strong>{{ ev.name || t('common.unnamed') }}</strong>
        <span class="muted">
          {{ t('config.event.perSet', ev.draws) }}<template v-if="ev.maxPerSession"> · {{ t('config.event.maxPerSession', ev.maxPerSession) }}</template>
        </span>
        <span v-if="ev.jokers" class="reward-tag">{{ t('common.reward', ev.jokers) }}</span>
      </span>
    </summary>
    <div class="field">
      <span>{{ t('common.name') }}</span>
      <input v-model="ev.name" :maxlength="MAX_NAME" :aria-label="t('config.event.name')" />
      <button class="ghost danger" @click="$emit('remove')">{{ t('common.remove') }}</button>
    </div>
    <div class="cfg-params">
      <label>
        <span>{{ t('config.event.draws') }}</span>
        <CountInput v-model="ev.draws" />
      </label>
      <label>
        <span>{{ t('config.event.limit') }}</span>
        <CountInput v-model="ev.maxPerSession" optional :placeholder="t('config.event.noLimit')" />
      </label>
      <label>
        <span>{{ t('config.event.jokers') }}</span>
        <CountInput v-model="ev.jokers" />
      </label>
    </div>
  </details>
</template>

<script setup>
import { t } from '../i18n.js';
import ProgramTile from './ProgramTile.vue';

// Tile of an event: its name, then its exercises per set, its limit and its jokers. A click opens EventDialog.
defineProps({ ev: Object });
</script>

<template>
  <ProgramTile>
    <strong class="tile-name">{{ ev.name.trim() || t('common.unnamed') }}</strong>
    <i18n-t keypath="config.event.perSet" :plural="+ev.draws || 0" tag="span" class="tile-foot" scope="global">
      <template #n><span class="tile-count">{{ ev.draws }}</span></template>
    </i18n-t>
    <span v-if="ev.maxPerSession" class="tile-meta">{{ t('config.event.maxPerSession', ev.maxPerSession) }}</span>
    <span v-if="ev.jokers" class="chip">{{ t('common.reward', ev.jokers) }}</span>
  </ProgramTile>
</template>

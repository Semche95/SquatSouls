<script setup>
import { t } from '../i18n/i18n.js';
import { useSession } from '../composables/useSession.js';

// Set waiting behind the current one, in a compact row: it can only be dismissed, never started.
defineProps({ draw: Object });

const { dismiss } = useSession();
</script>

<template>
  <article class="card queued">
    <header class="queued-event">
      {{ draw.eventName }}
      <span v-if="draw.jokers" class="reward-tag">{{ t('common.reward', draw.jokers) }}</span>
    </header>
    <ul class="queued-items">
      <li v-for="it in draw.items" :key="it.exerciseId">
        <span class="queued-amount">{{ it.amount }}</span> <span>{{ it.name }} <span class="muted">{{ it.unit }}</span></span>
      </li>
    </ul>
    <button class="ghost" @click="dismiss(draw)">{{ t('common.cancel') }}</button>
  </article>
</template>

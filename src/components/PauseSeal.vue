<script setup>
import { formatCountdown } from '../format.js';
import { t } from '../i18n.js';
import { useSession } from '../composables/useSession.js';
import { vTooltip } from '../tooltip.js';

// Pause countdown set astride the top border of its container, which must be positioned: the ring empties clockwise.
const { pauseLeft, pauseShare, skipPause } = useSession();
</script>

<template>
  <Transition name="seal">
    <button v-if="pauseLeft" v-tooltip="{ content: t('play.skipPause'), placement: 'top' }" class="pause-seal" :aria-label="t('play.skipPause')" @click="skipPause">
      <svg viewBox="0 0 100 100" aria-hidden="true">
        <circle class="seal-track" cx="50" cy="50" r="46" />
        <circle class="seal-ring" cx="50" cy="50" r="46" pathLength="1" :stroke-dashoffset="pauseShare - 1" />
      </svg>
      <span class="seal-label">{{ t('play.pause') }}</span>
      <strong class="pause-left">{{ formatCountdown(pauseLeft) }}</strong>
      <kbd>{{ t('keys.escape') }}</kbd>
    </button>
  </Transition>
</template>

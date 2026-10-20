<script setup>
import { computed } from 'vue';
import { formatCountdown } from '../i18n/format.js';
import { t } from '../i18n/i18n.js';
import { SIDES, useSession } from '../composables/useSession.js';
import { vTooltip } from '../ui/tooltip.js';

// Countdown set astride the top border of its container, which must be positioned: the ring empties clockwise.
// It shows the timer of a timed exercise (click or Space holds or resumes it, Escape stops it), otherwise the recovery
// (click or Escape skips it); both never run together. Its key is the one of the click.
const { recoveryLeft, recoveryShare, skipRecovery, timing, timerLeft, timerShare, timerHeld, toggleTimer } = useSession();

const seal = computed(() => {
  if (timing.value) {
    const { ready, side } = timing.value;
    const running = side ? t('timer.side', { side, sides: SIDES }) : t('draw.timer');
    const label = timerHeld.value ? t('timer.paused') : !ready ? running : side > 1 ? t('timer.otherSide') : t('timer.ready');
    const tip = t(timerHeld.value ? 'timer.resume' : 'timer.hold', { toggle: t('keys.space'), stop: t('keys.escape') });
    return { left: timerLeft.value, share: timerShare.value, label, tip, key: t('keys.space'), action: toggleTimer };
  }
  if (recoveryLeft.value) return { left: recoveryLeft.value, share: recoveryShare.value, label: t('play.recovery'), tip: t('play.skipRecovery'), key: t('keys.escape'), action: skipRecovery };
  return null;
});
</script>

<template>
  <Transition name="seal">
    <button
      v-if="seal"
      v-tooltip="{ content: seal.tip, placement: 'top' }"
      class="countdown-seal"
      :class="{ timer: timing, held: timerHeld }"
      :aria-label="seal.tip"
      @click="seal.action"
    >
      <svg viewBox="0 0 100 100" aria-hidden="true">
        <circle class="seal-track" cx="50" cy="50" r="46" />
        <circle class="seal-ring" cx="50" cy="50" r="46" pathLength="1" :stroke-dashoffset="seal.share - 1" />
      </svg>
      <span class="seal-label">{{ seal.label }}</span>
      <strong class="countdown-left">{{ formatCountdown(seal.left) }}</strong>
      <kbd>{{ seal.key }}</kbd>
    </button>
  </Transition>
</template>

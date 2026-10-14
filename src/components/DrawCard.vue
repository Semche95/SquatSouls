<script setup>
import { computed } from 'vue';
import { t } from '../i18n.js';
import { useSession } from '../composables/useSession.js';
import { vReveal } from '../reveal.js';
import DisabledTip from './DisabledTip.vue';
import CountdownSeal from './CountdownSeal.vue';

// current: set at the head of the queue, the only one to do; the next ones can only be dismissed.
const props = defineProps({ draw: Object, current: Boolean });

const { session, recoveryLeft, timing, timedUnit, startTimer, confirmDismiss, dismiss, dismissNow, remaining, canAfford, setSkipCost, complete, halve, skipExercise, skipSet } = useSession();
// First remaining exercise of the current set: the only one to mark done or time, also targeted by shortcuts.
const keyed = computed(() => remaining(props.draw)[0]);
// Why Done and the timer are blocked: recovery or timer running, empty otherwise.
const busy = computed(() => (recoveryLeft.value ? t('draw.recovering') : timing.value ? t('draw.timing') : ''));
</script>

<template>
  <article class="card" :class="{ current }">
    <CountdownSeal v-if="current" />
    <header class="card-head">
      <span>{{ draw.eventName }}</span>
    </header>
    <p v-if="draw.jokers" class="reward">{{ t('common.reward', draw.jokers) }}</p>
    <ul class="items">
      <li v-for="it in draw.items" :key="it.exerciseId" :class="{ done: it.done }">
        <span class="amount">{{ it.amount }}</span>
        <span class="what">
          <strong>{{ it.name }}</strong>
          <span class="muted">{{ it.unit }}<template v-if="it.halved"> · {{ t('common.halved') }}</template><template v-if="it.done"> · {{ t('draw.done') }}</template></span>
        </span>
        <span v-if="current && !it.done" class="item-actions">
          <template v-if="it === keyed">
            <DisabledTip :tip="busy">
              <button class="primary" :disabled="!!busy" @click="complete(draw, it)">
                {{ t('draw.complete') }} <kbd class="dark">{{ t('keys.enter') }}</kbd>
              </button>
            </DisabledTip>
            <DisabledTip v-if="timedUnit(it)" :tip="busy">
              <button class="ghost" :disabled="!!busy" @click="startTimer(draw, it)">{{ t('draw.timer') }} <kbd>T</kbd></button>
            </DisabledTip>
          </template>
          <template v-if="session.jokers">
            <DisabledTip v-if="it.halveCost" :tip="t('common.joker', it.halveCost)">
              <button class="ghost" :disabled="!canAfford(it.halveCost)" @click="halve(draw, it)">
                {{ t('draw.halve') }} <kbd v-if="it === keyed">H</kbd>
              </button>
            </DisabledTip>
            <DisabledTip :tip="t('common.joker', it.skipCost)">
              <button class="ghost" :disabled="!canAfford(it.skipCost)" @click="skipExercise(draw, it)">
                {{ t('draw.skip') }} <kbd v-if="it === keyed">J</kbd>
              </button>
            </DisabledTip>
          </template>
        </span>
      </li>
    </ul>
    <footer v-if="confirmDismiss === draw.id" v-reveal class="banner confirm">
      <p>{{ t('draw.confirmDismiss') }}</p>
      <div class="card-actions">
        <button class="danger" @click="dismissNow(draw)">{{ t('draw.dismissSet') }}</button>
        <button class="ghost" @click="confirmDismiss = 0">{{ t('draw.keepSet') }} <kbd>{{ t('keys.escape') }}</kbd></button>
      </div>
    </footer>
    <footer v-else class="card-actions">
      <DisabledTip v-if="current && session.jokers && remaining(draw).length > 1" :tip="canAfford(setSkipCost(draw)) ? '' : t('errors.not_enough_jokers')">
        <button class="ghost" :disabled="!canAfford(setSkipCost(draw))" @click="skipSet(draw)">
          {{ t('draw.skipSet', setSkipCost(draw)) }}
        </button>
      </DisabledTip>
      <button class="ghost" @click="dismiss(draw)">
        {{ t('common.cancel') }} <kbd v-if="current">{{ t('keys.backspace') }}</kbd>
      </button>
    </footer>
  </article>
</template>

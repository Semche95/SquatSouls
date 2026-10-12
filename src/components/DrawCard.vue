<script setup>
import { computed } from 'vue';
import { t } from '../i18n.js';
import { useSession } from '../composables/useSession.js';
import DisabledTip from './DisabledTip.vue';

// current: set at the head of the queue, the only one to do; the next ones can only be dismissed.
const props = defineProps({ draw: Object, current: Boolean });

const { session, pauseLeft, confirmDismiss, dismiss, dismissNow, remaining, canAfford, setSkipCost, complete, halve, skipExercise, skipSet } = useSession();
// Exercise targeted by shortcuts: the first remaining one of the current set.
const keyed = computed(() => remaining(props.draw)[0]);
</script>

<template>
  <article class="card" :class="{ current }">
    <header class="card-head">
      <span>{{ draw.eventName }}</span>
      <span class="muted">{{ draw.gameName }}</span>
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
          <DisabledTip :tip="pauseLeft ? t('draw.paused') : ''">
            <button class="primary" :disabled="!!pauseLeft" @click="complete(draw, it)">
              {{ t('draw.complete') }} <kbd v-if="it === keyed" class="dark">{{ t('keys.enter') }}</kbd>
            </button>
          </DisabledTip>
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
    <footer v-if="confirmDismiss === draw.id" class="banner confirm">
      <p>{{ t('draw.confirmDismiss') }}</p>
      <div class="card-actions">
        <button class="danger" @click="dismissNow(draw)">{{ t('draw.dismissSet') }}</button>
        <button class="ghost" @click="confirmDismiss = 0">{{ t('draw.keepSet') }} <kbd>{{ t('keys.escape') }}</kbd></button>
      </div>
    </footer>
    <footer v-else class="card-actions">
      <button v-if="current && session.jokers && remaining(draw).length > 1" class="ghost" :disabled="!canAfford(setSkipCost(draw))" @click="skipSet(draw)">
        {{ t('draw.skipSet', setSkipCost(draw)) }}
      </button>
      <button class="ghost" @click="dismiss(draw)">
        {{ t('common.cancel') }} <kbd v-if="current">{{ t('keys.backspace') }}</kbd>
      </button>
    </footer>
  </article>
</template>

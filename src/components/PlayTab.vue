<script setup>
import { computed } from 'vue';
import { t } from '../i18n/i18n.js';
import { byName } from '../config/configDraft.js';
import { vReveal } from '../ui/reveal.js';
import { vSelect } from '../ui/select.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useKeydown } from '../composables/useKeydown.js';
import { useSteam } from '../composables/useSteam.js';
import DrawCard from './DrawCard.vue';
import CountdownSeal from './CountdownSeal.vue';
import SessionActions from './SessionActions.vue';
import DisabledTip from './DisabledTip.vue';

const { config, tab, cfgTab, gameId, intensity, noGame, noExercise, game } = useConfig();
const { session, notice, skipRecovery, stopTimer, startTimer, toggleTimer, timing, trigger, confirmStop, dismiss, remaining, complete, halve, skipExercise } = useSession();
const { detected, selectDetected, addGame, ignoreDetected } = useSteam();

const INTENSITIES = ['easy', 'normal', 'hard'];

// The game is locked as soon as the session starts, by its button or its first event, until the next session.
const gameLocked = computed(() => !!session.value.startedAt);
// Detected Steam game: an unknown game is offered for adding, unless it is ignored; the configured games
// matching it, for selection, unless one of them already is or the session's game is locked.
const steamOffer = computed(() => {
  const d = detected.value;
  if (!d?.name) return false;
  if (!d.games.length) return !d.ignored;
  return !gameLocked.value && !d.games.some((g) => g.id === gameId.value);
});
// Games and exercises are created in their sub-tab of Program.
function showProgram(sub) {
  tab.value = 'program';
  cfgTab.value = sub;
}
// Current set highlighted, then the queue of the next ones.
const stage = computed(() => [session.value.pending.slice(0, 1), session.value.pending.slice(1)]);

function triggerCount(eventId) {
  const t = session.value.triggers.find((t) => t.gameId === game.value?.id && t.eventId === eventId);
  return t?.count ?? 0;
}

useKeydown((e) => {
  if (e.repeat) return;
  if (e.target.closest?.('input, textarea, .ts-wrapper')) return;
  // e.code ignores the layout: 1-9 work on AZERTY without Shift.
  const digit = e.code.match(/^(?:Digit|Numpad)([1-9])$/);
  if (digit) {
    const ev = game.value?.events[Number(digit[1]) - 1];
    if (ev) {
      e.preventDefault();
      trigger(ev);
    }
    return;
  }
  if (e.code === 'Escape') {
    skipRecovery();
    stopTimer();
    confirmStop.value = 0;
    return;
  }
  // Space holds or resumes the timer, it never marks done: a press just after the timer would do so by mistake.
  if (e.code === 'Space') {
    e.preventDefault();
    if (timing.value) toggleTimer();
    return;
  }
  const validate = e.code === 'Enter' || e.code === 'NumpadEnter';
  const skip = e.code === 'KeyJ';
  const half = e.code === 'KeyH';
  const time = e.code === 'KeyT';
  if (!validate && !skip && !half && !time && e.code !== 'Backspace') return;
  // Always blocked: otherwise they would activate the focused button (e.g. "Cancel the session").
  e.preventDefault();
  const first = session.value.pending[0];
  if (!first) return;
  // Done, the timer and jokers apply to the first remaining exercise of the set.
  const [item] = remaining(first);
  if (validate) complete(first, item);
  else if (time) startTimer(first, item);
  else if (skip) skipExercise(first, item);
  else if (half) halve(first, item);
  else dismiss(first);
});
</script>

<template>
  <main class="panel play">
    <div class="play-head">
      <div class="field">
        <label for="play-game">{{ t('play.game') }}</label>
        <DisabledTip class="grow" :tip="gameLocked ? t('play.gameLocked') : noGame ? t('play.noGame', { program: t('app.tabs.program') }) : ''">
          <select v-select id="play-game" v-model="gameId" :disabled="gameLocked || noGame">
            <option value="" disabled>{{ t('play.chooseGame') }}</option>
            <option v-for="g in byName(config?.games ?? [])" :key="g.id" :value="g.id" :disabled="!g.events.length">
              {{ g.events.length ? g.name : t('play.noEvent', { name: g.name }) }}
            </option>
          </select>
        </DisabledTip>
      </div>
      <div class="field">
        <span id="play-intensity">{{ t('play.intensity') }}</span>
        <div class="segmented" role="radiogroup" aria-labelledby="play-intensity">
          <DisabledTip v-for="level in INTENSITIES" :key="level" :tip="t(`play.intensityTips.${level}`)">
            <button
              role="radio"
              :aria-checked="intensity === level"
              :class="{ active: intensity === level }"
              @click="intensity = level"
            >
              {{ t(`play.intensities.${level}`) }}
            </button>
          </DisabledTip>
        </div>
      </div>
    </div>

    <p v-if="steamOffer" class="banner steam">
      <i18n-t keypath="play.detected" tag="span" scope="global">
        <template #name><strong>{{ detected.name }}</strong></template>
      </i18n-t>
      <span class="row">
        <template v-if="detected.games.length">
          <DisabledTip v-for="g in detected.games" :key="g.id" :tip="g.events.length ? '' : t('play.noEvent', { name: g.name })">
            <button class="primary" :disabled="!g.events.length" @click="selectDetected(g)">{{ g.name }}</button>
          </DisabledTip>
        </template>
        <template v-else>
          <button class="primary" @click="addGame(detected)">{{ t('common.add') }}</button>
          <button class="ghost" @click="ignoreDetected">{{ t('play.ignore') }}</button>
        </template>
      </span>
    </p>

    <div v-if="game" class="events">
      <DisabledTip v-for="(ev, i) in game.events" :key="ev.id" :tip="noExercise ? t('session.noExercise') : ''">
        <button class="event" :disabled="noExercise" @click="trigger(ev)">
          <kbd v-if="i < 9">{{ i + 1 }}</kbd>
          <span class="event-name">
            {{ ev.name }}
            <span v-if="ev.jokers" class="reward-tag">{{ t('common.reward', ev.jokers) }}</span>
          </span>
          <span v-if="triggerCount(ev.id)" class="count">
            <template v-if="ev.maxPerSession">{{ Math.min(triggerCount(ev.id), ev.maxPerSession) }} / {{ ev.maxPerSession }}</template>
            <template v-else>×{{ triggerCount(ev.id) }}</template>
          </span>
          <span v-if="ev.maxPerSession" class="edge-bar gauge">
            <span :style="{ width: `${Math.min(100, (100 * triggerCount(ev.id)) / ev.maxPerSession)}%` }"></span>
          </span>
        </button>
      </DisabledTip>
    </div>
    <i18n-t v-else-if="noGame" keypath="play.noGame" tag="p" class="empty" scope="global">
      <template #program><button class="link" @click="showProgram('games')">{{ t('app.tabs.program') }}</button></template>
    </i18n-t>
    <i18n-t v-if="noExercise" keypath="play.noExercise" tag="p" class="empty" scope="global">
      <template #program><button class="link" @click="showProgram('exercises')">{{ t('app.tabs.program') }}</button></template>
    </i18n-t>

    <p v-if="notice" v-reveal class="banner info">{{ notice }}</p>
    <p v-if="session.ended?.cause" v-reveal class="banner info">{{ t(`session.endCause.${session.ended.cause}`) }}</p>

    <section class="pending">
      <h2>
        {{ t('play.todo') }}
        <DisabledTip :tip="t('play.jokersTip')">
          <span class="jokers" :class="{ none: !session.jokers }">{{ t('play.jokers', session.jokers) }}</span>
        </DisabledTip>
      </h2>
      <p v-if="!session.pending.length" class="empty">
        {{ t('play.empty') }}
        <CountdownSeal />
      </p>
      <div v-if="session.pending.length" class="stage">
        <div v-for="(col, c) in stage" v-show="col.length" :key="c" :class="c === 0 ? 'stage-current' : 'queue'">
          <DrawCard v-for="d in col" :key="d.id" :draw="d" :current="c === 0" />
        </div>
      </div>
    </section>

    <SessionActions />
  </main>
</template>

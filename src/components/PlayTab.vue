<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { t } from '../i18n/i18n.js';
import { byName } from '../config/configDraft.js';
import { vReveal } from '../ui/reveal.js';
import { vSelect } from '../ui/select.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useKeydown } from '../composables/useKeydown.js';
import { useSteam } from '../composables/useSteam.js';
import { useSessionLimit } from '../composables/useSessionLimit.js';
import DrawCard from './DrawCard.vue';
import QueuedDraw from './QueuedDraw.vue';
import CountdownSeal from './CountdownSeal.vue';
import SessionActions from './SessionActions.vue';
import LimitField from './LimitField.vue';
import LimitGauge from './LimitGauge.vue';
import DisabledTip from './DisabledTip.vue';

const { config, tab, cfgTab, gameId, intensity, noGame, noExercise, game } = useConfig();
const { session, skipRecovery, stopTimer, startTimer, toggleTimer, timing, trigger, confirmStop, dismiss, remaining, addedDraw, addedTrigger, cappedTrigger, complete, halve, skipExercise } = useSession();
const { detected, selectDetected, addGame, ignoreDetected } = useSteam();
const { reached: limitReached } = useSessionLimit();

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
// Current set highlighted, then the queue of the next ones, oldest first: past a few, the rest is folded.
// Folding is display only, never saved: it starts folded again on coming back to Play.
const QUEUE_SHOWN = 3;
const queue = computed(() => session.value.pending.slice(1));
const queueOpen = ref(false);
const folded = computed(() => Math.max(0, queue.value.length - QUEUE_SHOWN));
const shownQueue = computed(() => (queueOpen.value ? queue.value : queue.value.slice(0, QUEUE_SHOWN)));
// A set just added to the folded part lights up the button that shows it.
const addedFolded = computed(() => !queueOpen.value && queue.value.slice(QUEUE_SHOWN).some((d) => d.id === addedDraw.value));
// Once nothing is left to fold, the next long queue starts folded.
watch(folded, (n) => {
  if (!n) queueOpen.value = false;
});

function triggerCount(eventId) {
  const counted = session.value.triggers.find((tr) => tr.gameId === game.value?.id && tr.eventId === eventId);
  return counted?.count ?? 0;
}
const atLimit = (ev) => ev.maxPerSession && triggerCount(ev.id) >= ev.maxPerSession;
// Why the event asks for nothing more, '' while it gives sets.
function limitTip(ev) {
  if (atLimit(ev)) return t('play.limitTip');
  return limitReached.value ? t('play.sessionLimitTip') : '';
}
// Counter of the event just selected: recreated at each selection, by its key, to replay its animation.
const bumped = (eventId) => addedTrigger.value?.eventId === eventId;
const capped = (eventId) => cappedTrigger.value?.eventId === eventId;
function countKey(eventId) {
  if (bumped(eventId)) return addedTrigger.value.drawId;
  return capped(eventId) ? `capped-${cappedTrigger.value.n}` : 0;
}
// Leaving Play ends the animations: coming back does not replay them.
onBeforeUnmount(() => {
  addedDraw.value = 0;
  addedTrigger.value = null;
  cappedTrigger.value = null;
});

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
          <select id="play-game" v-model="gameId" v-select :disabled="gameLocked || noGame">
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
      <LimitField />
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
      <DisabledTip
        v-for="(ev, i) in game.events"
        :key="ev.id"
        :tip="noExercise ? t('session.noExercise') : limitTip(ev)"
      >
        <button class="event" :disabled="noExercise" @click="trigger(ev)">
          <span class="event-head">
            <kbd v-if="i < 9">{{ i + 1 }}</kbd>
            <span
              v-if="triggerCount(ev.id)"
              :key="countKey(ev.id)"
              class="count"
              :class="{ added: bumped(ev.id), full: atLimit(ev) || limitReached, refused: capped(ev.id) }"
              @animationend="addedTrigger = null"
            >
              <template v-if="ev.maxPerSession">{{ Math.min(triggerCount(ev.id), ev.maxPerSession) }} / {{ ev.maxPerSession }}</template>
              <template v-else>×{{ triggerCount(ev.id) }}</template>
            </span>
          </span>
          <span class="event-name">
            {{ ev.name }}
            <span v-if="ev.jokers" class="reward-tag">{{ t('common.reward', ev.jokers) }}</span>
          </span>
          <span v-if="ev.maxPerSession" class="edge-bar gauge">
            <span :style="{ width: `${Math.min(100, (100 * triggerCount(ev.id)) / ev.maxPerSession)}%` }"></span>
          </span>
          <span
            v-if="capped(ev.id)"
            :key="cappedTrigger.n"
            class="limit-reached"
            aria-hidden="true"
            @animationend="cappedTrigger = null"
          >
            {{ t('play.limitReached') }}
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

    <p class="sr-only" aria-live="polite">
      <template v-if="cappedTrigger?.session">{{ t('play.sessionCapped', { event: cappedTrigger.eventName }) }}</template>
      <template v-else-if="cappedTrigger">{{ t('play.capped', { event: cappedTrigger.eventName, max: cappedTrigger.max }) }}</template>
    </p>
    <p v-if="session.ended?.cause" v-reveal class="banner info">{{ t(`session.endCause.${session.ended.cause}`) }}</p>

    <section class="pending">
      <h2>
        {{ t('play.todo') }}
        <DisabledTip :tip="t('play.jokersTip')">
          <span class="jokers" :class="{ none: !session.jokers }">{{ t('play.jokers', session.jokers) }}</span>
        </DisabledTip>
        <LimitGauge />
      </h2>
      <p v-if="!session.pending.length" class="empty">
        {{ t('play.empty') }}
        <CountdownSeal />
      </p>
      <div v-if="session.pending.length" class="stage">
        <div class="stage-current">
          <DrawCard
            :key="session.pending[0].id"
            :draw="session.pending[0]"
            :class="{ added: session.pending[0].id === addedDraw }"
            @animationend.self="addedDraw = 0"
          />
        </div>
        <div v-if="queue.length" class="queue">
          <QueuedDraw v-for="d in shownQueue" :key="d.id" :draw="d" :class="{ added: d.id === addedDraw }" @animationend.self="addedDraw = 0" />
          <button
            v-if="folded"
            class="ghost queue-more"
            :class="{ added: addedFolded }"
            :aria-expanded="queueOpen"
            @click="queueOpen = !queueOpen"
            @animationend.self="addedDraw = 0"
          >
            {{ queueOpen ? t('play.foldQueued') : t('play.moreQueued', folded) }}
          </button>
        </div>
      </div>
    </section>

    <SessionActions />
  </main>
</template>

<script setup>
import { computed } from 'vue';
import { formatCountdown, plural } from '../format.js';
import { byName } from '../configDraft.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useKeydown } from '../composables/useKeydown.js';
import { useSteam } from '../composables/useSteam.js';
import DrawCard from './DrawCard.vue';
import SessionActions from './SessionActions.vue';
import DisabledTip from './DisabledTip.vue';

const { config, gameId, game } = useConfig();
const { session, notice, pauseLeft, skipPause, trigger, confirmDismiss, dismiss, remaining, complete, halve, skipExercise } = useSession();
const { detected, selectDetected, addGame, ignoreDetected } = useSteam();

// The game is locked from the session's first event until the next session.
const gameLocked = computed(() => !!session.value.gameId);
// Detected Steam game: an unknown game is offered for adding, unless it is ignored; the configured games
// matching it, for selection, unless one of them already is or the session's game is locked.
const steamOffer = computed(() => {
  const d = detected.value;
  if (!d?.name) return false;
  if (!d.games.length) return !d.ignored;
  return !gameLocked.value && !d.games.some((g) => g.id === gameId.value);
});
// Current set highlighted, then the queue of the next ones.
const stage = computed(() => [session.value.pending.slice(0, 1), session.value.pending.slice(1)]);

function triggerCount(eventId) {
  const t = session.value.triggers.find((t) => t.gameId === game.value?.id && t.eventId === eventId);
  return t?.count ?? 0;
}

useKeydown((e) => {
  if (e.repeat) return;
  if (e.target.closest?.('input, textarea, select')) return;
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
  // Always blocked: otherwise they would activate the focused button (e.g. "Annuler la séance").
  if (e.code === 'Escape') {
    skipPause();
    confirmDismiss.value = 0;
    return;
  }
  const validate = e.code === 'Enter' || e.code === 'NumpadEnter' || e.code === 'Space';
  const skip = e.code === 'KeyJ';
  const half = e.code === 'KeyH';
  if (!validate && !skip && !half && e.code !== 'Backspace') return;
  e.preventDefault();
  const first = session.value.pending[0];
  if (!first) return;
  // Done and jokers apply to the first remaining exercise of the set.
  const [item] = remaining(first);
  if (validate) complete(first, item);
  else if (skip) skipExercise(first, item);
  else if (half) halve(first, item);
  else dismiss(first);
});
</script>

<template>
  <main class="panel play">
    <label class="field">
      <span>Jeu</span>
      <DisabledTip :tip="gameLocked ? 'Le jeu ne peut plus changer pendant la séance' : ''">
        <select v-model="gameId" :disabled="gameLocked">
          <option value="" disabled>Choisir un jeu</option>
          <option v-for="g in byName(config?.games ?? [])" :key="g.id" :value="g.id">{{ g.name }}</option>
        </select>
      </DisabledTip>
    </label>

    <p v-if="steamOffer" class="banner steam">
      <span><strong>{{ detected.name }}</strong> détecté.</span>
      <span class="row">
        <template v-if="detected.games.length">
          <button v-for="g in detected.games" :key="g.id" class="primary" @click="selectDetected(g)">{{ g.name }}</button>
        </template>
        <template v-else>
          <button class="primary" @click="addGame(detected)">Ajouter</button>
          <button class="ghost" @click="ignoreDetected">Ignorer</button>
        </template>
      </span>
    </p>

    <div v-if="game" class="events">
      <button v-for="(ev, i) in game.events" :key="ev.id" class="event" @click="trigger(ev)">
        <kbd v-if="i < 9">{{ i + 1 }}</kbd>
        <span class="event-name">
          {{ ev.name }}
          <span v-if="ev.jokers" class="reward-tag">+{{ plural(ev.jokers, 'joker') }}</span>
        </span>
        <span v-if="triggerCount(ev.id)" class="count">
          <template v-if="ev.maxPerSession">{{ Math.min(triggerCount(ev.id), ev.maxPerSession) }} / {{ ev.maxPerSession }}</template>
          <template v-else>×{{ triggerCount(ev.id) }}</template>
        </span>
        <span v-if="ev.maxPerSession" class="gauge">
          <span :style="{ width: `${Math.min(100, (100 * triggerCount(ev.id)) / ev.maxPerSession)}%` }"></span>
        </span>
      </button>
    </div>

    <p v-if="notice" class="banner info">{{ notice }}</p>

    <p v-if="pauseLeft" class="banner pause">
      Pause <strong class="pause-left">{{ formatCountdown(pauseLeft) }}</strong>
      <button class="ghost" @click="skipPause">Passer <kbd>Échap</kbd></button>
    </p>

    <section class="pending">
      <h2>
        À faire
        <span class="jokers" :class="{ none: !session.jokers }" title="J : sauter la plus ancienne série">
          {{ session.jokers ? plural(session.jokers, 'joker') : 'aucun joker' }}
        </span>
      </h2>
      <p v-if="!session.pending.length" class="empty">
        Rien en attente. Clique sur un événement (ou tape son numéro) quand il arrive en jeu.
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

<script setup>
import { computed } from 'vue';
import { formatCountdown, plural } from '../format.js';
import { byName } from '../configDraft.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { useKeydown } from '../composables/useKeydown.js';
import DrawCard from './DrawCard.vue';
import SessionActions from './SessionActions.vue';

const { config, gameId, game } = useConfig();
const { session, notice, pauseLeft, skipPause, trigger, act, halve, skipExercise } = useSession();

// Le jeu est figé dès le premier événement de la séance, jusqu'à la suivante.
const gameLocked = computed(() => !!session.value.gameId);
// Série en cours mise en avant, puis la file des suivantes.
const stage = computed(() => [session.value.pending.slice(0, 1), session.value.pending.slice(1)]);

function triggerCount(eventId) {
  const t = session.value.triggers.find((t) => t.gameId === game.value?.id && t.eventId === eventId);
  return t?.count ?? 0;
}

useKeydown((e) => {
  if (e.repeat) return;
  if (e.target.closest?.('input, textarea, select')) return;
  // e.code ignore la disposition : 1-9 marchent sur AZERTY sans Maj.
  const digit = e.code.match(/^(?:Digit|Numpad)([1-9])$/);
  if (digit) {
    const ev = game.value?.events[Number(digit[1]) - 1];
    if (ev) {
      e.preventDefault();
      trigger(ev);
    }
    return;
  }
  // Toujours bloquées : sinon elles activeraient le bouton qui a le focus (ex. « Annuler la séance »).
  if (e.code === 'Escape') {
    skipPause();
    return;
  }
  const validate = e.code === 'Enter' || e.code === 'NumpadEnter' || e.code === 'Space';
  const skip = e.code === 'KeyJ';
  const half = e.code === 'KeyH';
  if (!validate && !skip && !half && e.code !== 'Backspace') return;
  e.preventDefault();
  const first = session.value.pending[0];
  if (!first) return;
  // Les jokers portent sur le premier exercice de la série.
  if (validate) act('complete', first);
  else if (skip) skipExercise(first, first.items[0]);
  else if (half) halve(first, first.items[0]);
  else act('dismiss', first);
});
</script>

<template>
  <main class="panel play">
    <label class="field">
      <span>Jeu</span>
      <select v-model="gameId" :disabled="gameLocked" :title="gameLocked ? 'Le jeu ne peut plus changer pendant la séance' : ''">
        <option value="" disabled>Choisir un jeu</option>
        <option v-for="g in byName(config?.games ?? [])" :key="g.id" :value="g.id">{{ g.name }}</option>
      </select>
    </label>

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

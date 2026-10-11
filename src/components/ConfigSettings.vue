<script setup>
import { computed } from 'vue';
import { byName } from '../configDraft.js';
import { useConfig } from '../composables/useConfig.js';
import { useSteam } from '../composables/useSteam.js';

const { config, draft } = useConfig();
const { detected, storeBlocked, addGame, unignore } = useSteam();

// Status of the saved detection, to see it works even without a banner in Jouer:
// text and color of its badge.
const status = computed(() => {
  const d = detected.value;
  if (!d) return { text: 'Aucun jeu Steam détecté.', kind: 'idle' };
  if (d.name) return { text: `Jeu détecté : ${d.name}${d.ignored ? ' (ignoré)' : ''}.`, kind: 'found' };
  return storeBlocked.value
    ? { text: 'Jeu Steam détecté, mais le Steam Store limite les requêtes : les jeux inconnus ne seront plus proposés pendant 24 h au plus.', kind: 'blocked' }
    : { text: 'Jeu Steam détecté, nom pas encore connu.', kind: 'idle' };
});
</script>

<template>
  <div class="cfg-settings">
    <section class="group setting">
      <h2 class="group-title">Zone de notification</h2>
      <div>
        <label class="check">
          Afficher l'icône dans la zone de notification
          <input v-model="draft.tray" type="checkbox" />
        </label>
        <p class="hint">Garde l'appli accessible depuis la barre système.</p>
      </div>
      <div v-if="draft.tray" class="nested">
        <label class="check">
          Fermer la fenêtre la masque au lieu de quitter
          <input v-model="draft.closeToTray" type="checkbox" />
        </label>
        <label class="check">
          Démarrer masqué
          <input v-model="draft.startHidden" type="checkbox" />
        </label>
      </div>
    </section>
    <section class="group setting">
      <h2 class="group-title">Steam</h2>
      <div>
        <label class="check">
          Détecter le jeu Steam en cours
          <input v-model="draft.steamDetection" type="checkbox" />
        </label>
        <p class="hint">Propose le jeu lancé dans l'onglet Jouer.</p>
      </div>
      <div v-if="draft.steamDetection" class="nested">
        <label class="check">
          Notifier quand un jeu est lancé
          <input v-model="draft.steamNotify" type="checkbox" />
        </label>
      </div>
      <p v-if="config.steamDetection" class="status" :class="status.kind">{{ status.text }}</p>
      <template v-if="config.ignoredGames.length">
        <h3 class="group-title sub">Jeux ignorés</h3>
        <ul class="ignored">
          <li v-for="g in byName(config.ignoredGames)" :key="g.appId">
            {{ g.name }}
            <span class="row">
              <button class="ghost" @click="addGame(g)">Ajouter</button>
              <button class="ghost" @click="unignore(g.appId)">Retirer</button>
            </span>
          </li>
        </ul>
      </template>
    </section>
  </div>
</template>

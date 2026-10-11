<script setup>
import { computed } from 'vue';
import { byName } from '../configDraft.js';
import { useConfig } from '../composables/useConfig.js';
import { useSteam } from '../composables/useSteam.js';

const { config, draft } = useConfig();
const { detected, storeBlocked, addGame, unignore } = useSteam();

// État de la détection enregistrée, pour voir qu'elle marche même sans bandeau dans Jouer.
const status = computed(() => {
  const d = detected.value;
  if (!d) return 'Aucun jeu Steam détecté.';
  if (d.name) return `Jeu détecté : ${d.name}${d.ignored ? ' (ignoré)' : ''}.`;
  return storeBlocked.value
    ? 'Jeu Steam détecté, mais le Steam Store limite les requêtes : les jeux inconnus ne seront plus proposés avant le prochain lancement.'
    : 'Jeu Steam détecté, nom pas encore connu.';
});
</script>

<template>
  <section class="group cfg-settings">
    <h2 class="group-title">Détection des jeux Steam</h2>
    <div class="setting">
      <label class="check">
        <input v-model="draft.steamDetection" type="checkbox" />
        Détecter le jeu Steam en cours
      </label>
      <p v-if="config.steamDetection" class="muted small">{{ status }}</p>
    </div>
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
</template>

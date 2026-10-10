<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

const tab = ref("jouer");
const config = ref(null);
const configJson = ref("");
const configPath = ref("");
const configError = ref("");
const configMsg = ref("");
const confirmReset = ref(false);
const confirmNewSession = ref(false);

const session = ref({ pending: [], sets: 0, totals: [], log: [], triggers: [] });
const history = ref([]);
const notice = ref("");
const error = ref("");

const gameId = ref(readLocal("gameId"));

function readLocal(key) {
  try {
    return localStorage.getItem(key) ?? "";
  } catch {
    return "";
  }
}
function writeLocal(key, value) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* stockage indisponible : on oublie simplement le dernier jeu */
  }
}

const game = computed(() => {
  const games = config.value?.games ?? [];
  return games.find((g) => g.id === gameId.value) ?? games[0] ?? null;
});

watch(game, (g) => {
  if (g && g.id !== gameId.value) gameId.value = g.id;
});
watch(gameId, (id) => writeLocal("gameId", id));

function triggerCount(eventId) {
  const t = session.value.triggers.find((t) => t.gameId === game.value?.id && t.eventId === eventId);
  return t?.count ?? 0;
}

function formatAmount(item) {
  return `${item.amount} ${item.unit}`;
}

function formatDay(iso) {
  const [y, m, d] = iso.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString("fr-FR", { weekday: "short", day: "numeric", month: "short" });
}

function formatTime(seconds) {
  return new Date(seconds * 1000).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
}

function applyConfigView(view) {
  config.value = view.config;
  configJson.value = view.json;
  configPath.value = view.path;
  configError.value = view.error ?? "";
}

async function call(command, args) {
  error.value = "";
  try {
    return await invoke(command, args);
  } catch (e) {
    error.value = String(e);
    throw e;
  }
}

async function trigger(ev) {
  notice.value = "";
  try {
    const draw = await call("draw", { gameId: game.value.id, eventId: ev.id });
    if (draw.capped) {
      notice.value = `${draw.eventName} : limite de ${draw.maxPerSession} atteinte pour la séance, pas de série.`;
    }
    session.value = await call("get_session");
  } catch {
    /* erreur déjà affichée */
  }
}

async function complete(draw) {
  try {
    session.value = await call("complete", { drawId: draw.id });
  } catch {
    session.value = await invoke("get_session");
  }
}

async function dismiss(draw) {
  try {
    session.value = await call("dismiss", { drawId: draw.id });
  } catch {
    session.value = await invoke("get_session");
  }
}

async function newSession() {
  if (!confirmNewSession.value) {
    confirmNewSession.value = true;
    return;
  }
  confirmNewSession.value = false;
  notice.value = "";
  session.value = await call("new_session");
}

async function loadHistory() {
  try {
    history.value = await call("get_history", { days: 7 });
  } catch {
    history.value = [];
  }
}

async function saveConfig() {
  configMsg.value = "";
  try {
    applyConfigView(await invoke("save_config", { json: configJson.value }));
    configMsg.value = "Configuration enregistrée.";
    session.value = await invoke("get_session");
  } catch (e) {
    configError.value = String(e);
  }
}

async function resetConfig() {
  if (!confirmReset.value) {
    confirmReset.value = true;
    return;
  }
  confirmReset.value = false;
  configMsg.value = "";
  try {
    applyConfigView(await invoke("reset_config"));
    configMsg.value = "Configuration par défaut restaurée.";
    session.value = await invoke("get_session");
  } catch (e) {
    configError.value = String(e);
  }
}

watch(tab, (t) => {
  confirmReset.value = false;
  confirmNewSession.value = false;
  if (t === "seance") loadHistory();
});

function onKey(e) {
  if (tab.value !== "jouer" || e.repeat) return;
  if (e.target.closest?.("input, textarea, select")) return;
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
  const first = session.value.pending[0];
  if (!first) return;
  if (e.code === "Enter" || e.code === "NumpadEnter" || e.code === "Space") {
    e.preventDefault();
    complete(first);
  } else if (e.code === "Backspace") {
    e.preventDefault();
    dismiss(first);
  }
}

onMounted(async () => {
  window.addEventListener("keydown", onKey);
  try {
    applyConfigView(await call("get_config"));
    session.value = await call("get_session");
  } catch {
    /* erreur déjà affichée */
  }
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <div class="app">
    <header class="top">
      <h1>Souls Fit</h1>
      <nav class="tabs">
        <button :class="{ active: tab === 'jouer' }" @click="tab = 'jouer'">Jouer</button>
        <button :class="{ active: tab === 'seance' }" @click="tab = 'seance'">
          Séance <span v-if="session.sets" class="badge">{{ session.sets }}</span>
        </button>
        <button :class="{ active: tab === 'config' }" @click="tab = 'config'">Configuration</button>
      </nav>
    </header>

    <p v-if="error" class="banner error">{{ error }}</p>
    <p v-if="configError && tab !== 'config'" class="banner error">
      {{ configError }}
      <button class="link" @click="tab = 'config'">Voir la configuration</button>
    </p>

    <!-- JOUER -->
    <main v-if="tab === 'jouer'" class="panel">
      <label class="field">
        <span>Jeu</span>
        <select v-model="gameId">
          <option v-for="g in config?.games ?? []" :key="g.id" :value="g.id">{{ g.name }}</option>
        </select>
      </label>

      <div v-if="game" class="events">
        <button v-for="(ev, i) in game.events" :key="ev.id" class="event" @click="trigger(ev)">
          <kbd v-if="i < 9">{{ i + 1 }}</kbd>
          <span class="event-name">{{ ev.name }}</span>
          <span v-if="triggerCount(ev.id)" class="count">
            <template v-if="ev.maxPerSession">{{ Math.min(triggerCount(ev.id), ev.maxPerSession) }} / {{ ev.maxPerSession }}</template>
            <template v-else>×{{ triggerCount(ev.id) }}</template>
          </span>
        </button>
      </div>

      <p v-if="notice" class="banner info">{{ notice }}</p>

      <section class="pending">
        <h2>À faire</h2>
        <p v-if="!session.pending.length" class="empty">
          Rien en attente. Clique sur un événement (ou tape son numéro) quand il arrive en jeu.
        </p>
        <article v-for="(d, i) in session.pending" :key="d.id" class="card" :class="{ current: i === 0 }">
          <header class="card-head">
            <span>{{ d.eventName }}</span>
            <span class="muted">{{ d.gameName }}</span>
          </header>
          <ul class="items">
            <li v-for="it in d.items" :key="it.exerciseId">
              <span class="amount">{{ it.amount }}</span>
              <span class="what">
                <strong>{{ it.name }}</strong>
                <span class="muted">{{ it.unit }}</span>
              </span>
            </li>
          </ul>
          <footer class="card-actions">
            <button class="primary" @click="complete(d)">
              Fait <kbd v-if="i === 0" class="dark">Entrée</kbd>
            </button>
            <button class="ghost" @click="dismiss(d)">
              Annuler <kbd v-if="i === 0">Retour arr.</kbd>
            </button>
          </footer>
        </article>
      </section>
    </main>

    <!-- SÉANCE -->
    <main v-else-if="tab === 'seance'" class="panel">
      <section>
        <div class="section-head">
          <h2>Séance en cours</h2>
          <span class="muted">depuis {{ formatTime(session.startedAt) }}</span>
        </div>
        <p v-if="!session.sets" class="empty">Aucune série faite pour l'instant.</p>
        <table v-else class="totals">
          <thead>
            <tr><th>Exercice</th><th>Séries</th><th>Total</th></tr>
          </thead>
          <tbody>
            <tr v-for="t in session.totals" :key="t.exercise + t.unit">
              <td>{{ t.exercise }}</td>
              <td>{{ t.sets }}</td>
              <td>{{ t.amount }} {{ t.unit }}</td>
            </tr>
          </tbody>
        </table>
        <button class="ghost wide" :class="{ danger: confirmNewSession }" @click="newSession">
          {{ confirmNewSession ? "Confirmer : remettre la séance à zéro" : "Nouvelle séance" }}
        </button>
      </section>

      <section v-if="session.log.length">
        <h2>Séries faites</h2>
        <ul class="log">
          <li v-for="(e, i) in session.log" :key="i">
            <span class="muted">{{ formatTime(e.at) }}</span>
            <span>{{ e.amount }} {{ e.unit }} · {{ e.exercise }}</span>
            <span class="muted">{{ e.event }}</span>
          </li>
        </ul>
      </section>

      <section>
        <h2>7 derniers jours</h2>
        <ul class="days">
          <li v-for="d in history" :key="d.date" :class="{ off: !d.sets }">
            <span class="day">{{ formatDay(d.date) }}</span>
            <span v-if="!d.sets" class="muted">repos</span>
            <span v-else class="day-totals">
              <span v-for="t in d.totals" :key="t.exercise + t.unit" class="chip">
                {{ t.exercise }} {{ t.amount }} {{ t.unit }}
              </span>
            </span>
          </li>
        </ul>
      </section>
    </main>

    <!-- CONFIGURATION -->
    <main v-else class="panel config">
      <p class="muted small">
        Fichier : <code>{{ configPath }}</code><br />
        Chaque événement tire <code>draws</code> exercices différents dans sa liste, avec un nombre entre
        <code>min</code> et <code>max</code> inclus. <code>maxPerSession</code> (facultatif) coupe l'événement
        au-delà d'un certain nombre de déclenchements dans la séance.
      </p>
      <p v-if="configError" class="banner error">{{ configError }}</p>
      <p v-if="configMsg" class="banner ok">{{ configMsg }}</p>
      <textarea v-model="configJson" spellcheck="false" @input="configMsg = ''"></textarea>
      <div class="row">
        <button class="primary" @click="saveConfig">Enregistrer</button>
        <button class="ghost" :class="{ danger: confirmReset }" @click="resetConfig">
          {{ confirmReset ? "Confirmer la réinitialisation" : "Revenir à la config par défaut" }}
        </button>
      </div>
    </main>
  </div>
</template>

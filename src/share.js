import { deflateSync, inflateSync, strFromU8, strToU8 } from 'fflate';
import { newId, sameName } from './configDraft.js';

// Texte partagé entre utilisateurs : un jeu (nom et événements) ou la liste des exercices, sans id,
// en JSON compressé (deflate) puis encodé en base64.
const MARK = 'squatsouls';
const GAME = 'jeu';
const EXERCISES = 'exercices';

const isText = (v) => typeof v === 'string' && v.trim() !== '';
const isCount = (v, min) => Number.isInteger(v) && v >= min;

function share(kind, data) {
  return btoa(String.fromCharCode(...deflateSync(strToU8(JSON.stringify({ [MARK]: kind, ...data })))));
}

// JSON contenu dans un texte partagé ; lève une erreur si le texte n'en est pas un.
export function unpack(text) {
  return strFromU8(inflateSync(Uint8Array.from(atob(text), (c) => c.charCodeAt(0))));
}

// Renvoie le contenu du texte collé s'il est du type attendu et que `valid` l'accepte, sinon lève `error`.
function parse(text, kind, valid, error) {
  let data = null;
  try {
    data = JSON.parse(unpack(text));
  } catch {
    // Texte quelconque : refusé ci-dessous.
  }
  if (data?.[MARK] !== kind || !valid(data)) throw new Error(error);
  return data;
}

export function shareGame(game) {
  const events = game.events.map(({ name, draws, maxPerSession, jokers }) => ({ name, draws, maxPerSession, jokers }));
  return share(GAME, { name: game.name, events });
}

export function shareExercises(exercises) {
  return share(EXERCISES, { exercises: exercises.map(({ name, unit, min, max }) => ({ name, unit, min, max })) });
}

const validEvent = (ev) =>
  isText(ev?.name) && isCount(ev.draws, 1) && (ev.maxPerSession === undefined || isCount(ev.maxPerSession, 1)) && (ev.jokers === undefined || isCount(ev.jokers, 0));

// Jeu partagé prêt à importer ; `draws` est ramené au nombre d'exercices définis (`clamped` le signale).
export function parseGame(text, exerciseCount) {
  const data = parse(
    text,
    GAME,
    (d) => isText(d.name) && Array.isArray(d.events) && d.events.length > 0 && d.events.every(validEvent),
    'Ce texte n\'est pas un jeu partagé valide.',
  );
  const events = [];
  for (const { name, draws, maxPerSession, jokers = 0 } of data.events) {
    const ev = { id: newId(name, events), name: name.trim(), draws: Math.min(draws, exerciseCount), jokers };
    if (maxPerSession !== undefined) ev.maxPerSession = maxPerSession;
    events.push(ev);
  }
  return { name: data.name.trim(), events, clamped: data.events.some((ev) => ev.draws > exerciseCount) };
}

const validExercise = (ex) => isText(ex?.name) && typeof ex.unit === 'string' && isCount(ex.min, 0) && isCount(ex.max, 1) && ex.min <= ex.max;

export function parseExercises(text) {
  return parse(
    text,
    EXERCISES,
    (d) => Array.isArray(d.exercises) && d.exercises.length > 0 && d.exercises.every(validExercise),
    'Ce texte n\'est pas une liste d\'exercices partagée valide.',
  ).exercises;
}

// Le jeu importé remplace celui qui porte le même nom (en gardant son id), sinon il s'ajoute.
export function importGame(games, { name, events }) {
  const existing = games.find((g) => sameName(g.name, name));
  const game = { id: existing?.id ?? newId(name, games), name, events };
  return existing ? games.map((g) => (g === existing ? game : g)) : [...games, game];
}

// Les exercices importés mettent à jour ceux qui portent le même nom et ajoutent les autres.
export function mergeExercises(exercises, imported) {
  const merged = exercises.map((ex) => ({ ...ex }));
  let added = 0;
  let updated = 0;
  for (const { name, unit, min, max } of imported) {
    const ex = merged.find((e) => sameName(e.name, name));
    if (!ex) {
      merged.push({ id: newId(name, merged), name: name.trim(), unit, min, max });
      added++;
    } else if (ex.unit !== unit || ex.min !== min || ex.max !== max) {
      Object.assign(ex, { unit, min, max });
      updated++;
    }
  }
  return { exercises: merged, added, updated };
}

import { deflateSync, inflateSync, strFromU8, strToU8 } from 'fflate';
import { isCount, newId, sameName } from './configDraft.js';

// Texte partagé entre utilisateurs : un jeu (nom et événements) ou la liste des exercices, sans id,
// en JSON compressé (deflate) puis encodé en base64, précédé d'un court identifiant qui donne le type de
// contenu (`squatsouls/game/sekiro:`, `squatsouls/exercises:`). La version du format est dans le JSON compressé.
const MARK = 'squatsouls';
const VERSION = 1;
const GAME = 'game';
const EXERCISES = 'exercises';

// Lecture du contenu selon la version du format qui l'a écrit. Changer le format, c'est passer à une
// nouvelle version et lui ajouter un lecteur, en gardant ceux des précédentes pour importer leurs textes.
const READERS = {
  1: (data) => data,
};

const isText = (v) => typeof v === 'string' && v.trim() !== '';

function share(kind, data, id) {
  const payload = JSON.stringify({ version: VERSION, ...data });
  return `${MARK}/${id ? `${kind}/${id}` : kind}:` + btoa(String.fromCharCode(...deflateSync(strToU8(payload))));
}

// Version, type et contenu d'un texte partagé (vide si ce n'en est pas un, contenu absent si la version
// est inconnue) ; lève une erreur si le contenu est illisible. L'identifiant précède le dernier `:`,
// absent du base64.
export function unpack(text) {
  const at = text.lastIndexOf(':');
  const [mark, kind] = text.slice(0, at).trim().split('/');
  if (at < 0 || mark !== MARK) return {};
  const { version, ...data } = JSON.parse(strFromU8(inflateSync(Uint8Array.from(atob(text.slice(at + 1)), (c) => c.charCodeAt(0)))));
  return { version, kind, data: READERS[version]?.(data) };
}

// Messages d'import par type : texte invalide, ou texte de ce type collé dans l'import de l'autre type.
const ERRORS = {
  [GAME]: {
    invalid: 'Ce texte n\'est pas un jeu partagé valide.',
    misplaced: 'Ce texte partage un jeu : l\'importer depuis l\'onglet Jeux, avec Importer un jeu.',
  },
  [EXERCISES]: {
    invalid: 'Ce texte n\'est pas une liste d\'exercices partagée valide.',
    misplaced: 'Ce texte partage des exercices : l\'importer depuis l\'onglet Exercices, avec Importer.',
  },
};

const NEWER = 'Ce texte vient d\'une version plus récente de SquatSouls : la mettre à jour pour l\'importer.';

// Renvoie le contenu du texte collé s'il est du type attendu et que `valid` l'accepte, sinon lève une erreur.
function parse(text, kind, valid) {
  let read = {};
  try {
    read = unpack(text);
  } catch {
    // Contenu illisible : refusé ci-dessous.
  }
  if (read.kind !== kind && Object.hasOwn(ERRORS, read.kind)) throw new Error(ERRORS[read.kind].misplaced);
  if (read.kind === kind && !Object.hasOwn(READERS, read.version)) throw new Error(NEWER);
  if (read.kind !== kind || !read.data || !valid(read.data)) throw new Error(ERRORS[kind].invalid);
  return read.data;
}

export function shareGame(game) {
  const events = game.events.map(({ name, draws, maxPerSession, jokers }) => ({ name, draws, maxPerSession, jokers }));
  return share(GAME, { name: game.name, events }, newId(game.name, []));
}

// Exercice sans pause propre : la clé est absente, il suit la pause globale.
const withPause = (ex, pause) => (pause === undefined ? ex : { ...ex, pause });

// La pause globale n'est pas partagée : chacun garde la sienne.
export function shareExercises(exercises) {
  return share(EXERCISES, { exercises: exercises.map(({ name, unit, min, max, pause }) => withPause({ name, unit, min, max }, pause)) });
}

const validEvent = (ev) =>
  isText(ev?.name) && isCount(ev.draws, 1) && (ev.maxPerSession === undefined || isCount(ev.maxPerSession, 1)) && (ev.jokers === undefined || isCount(ev.jokers, 0));

// Jeu partagé prêt à importer ; `draws` est ramené au nombre d'exercices définis (`clamped` le signale).
export function parseGame(text, exerciseCount) {
  const data = parse(
    text,
    GAME,
    (d) => isText(d.name) && Array.isArray(d.events) && d.events.length > 0 && d.events.every(validEvent),
  );
  const events = [];
  for (const { name, draws, maxPerSession, jokers = 0 } of data.events) {
    const ev = { id: newId(name, events), name: name.trim(), draws: Math.min(draws, exerciseCount), jokers };
    if (maxPerSession !== undefined) ev.maxPerSession = maxPerSession;
    events.push(ev);
  }
  return { name: data.name.trim(), events, clamped: data.events.some((ev) => ev.draws > exerciseCount) };
}

const validExercise = (ex) =>
  isText(ex?.name) && typeof ex.unit === 'string' && isCount(ex.min, 0) && isCount(ex.max, 1) && ex.min <= ex.max && (ex.pause === undefined || isCount(ex.pause, 0));

export function parseExercises(text) {
  return parse(
    text,
    EXERCISES,
    (d) => Array.isArray(d.exercises) && d.exercises.length > 0 && d.exercises.every(validExercise),
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
  for (const { name, unit, min, max, pause } of imported) {
    const i = merged.findIndex((e) => sameName(e.name, name));
    const ex = merged[i];
    if (!ex) {
      merged.push(withPause({ id: newId(name, merged), name: name.trim(), unit, min, max }, pause));
      added++;
    } else if (ex.unit !== unit || ex.min !== min || ex.max !== max || ex.pause !== pause) {
      merged[i] = withPause({ id: ex.id, name: ex.name, unit, min, max }, pause);
      updated++;
    }
  }
  return { exercises: merged, added, updated };
}

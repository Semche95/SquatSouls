export function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

// Les id restent internes : dérivés du nom à la création, puis fixes même si on renomme.
export function newId(name, items) {
  const base = name.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'id';
  const taken = new Set(items.map((it) => it.id));
  let id = base;
  for (let n = 2; taken.has(id); n++) id = `${base}-${n}`;
  return id;
}

// Deux noms identiques aux espaces autour et à la casse près désignent le même jeu.
export function sameName(a, b) {
  return a.trim().toLowerCase() === b.trim().toLowerCase();
}

// Un nom qui commence par celui d'un jeu, sans le prolonger d'une lettre (« Megabonk 2 », « Elden Ring sans coup »),
// désigne une variante de ce jeu, à la casse et aux espaces autour près.
export function extendsName(name, base) {
  const [n, b] = [name, base].map((s) => s.trim().toLowerCase());
  return n.startsWith(b) && !/\p{L}/u.test(n[b.length] ?? '');
}

// Les jeux s'affichent par nom, quel que soit l'ordre enregistré.
export function byName(games) {
  return [...games].sort((a, b) => a.name.localeCompare(b.name, 'fr', { sensitivity: 'base' }));
}

// Plus grand nombre accepté par le backend (u32).
const MAX_COUNT = 4294967295;

export const isCount = (v, min) => Number.isInteger(v) && v >= min && v <= MAX_COUNT;

// Texte saisi dans un champ numérique : un nombre s'il n'a que des chiffres, sinon le texte tel quel
// (vide, il vaut '' ; autre chose, countError le signale).
export function parseCount(text) {
  const t = text.trim();
  return /^\d+$/.test(t) ? Number(t) : t;
}

// Le backend refuse sans explication un nombre négatif, décimal ou trop grand : on le signale dès la saisie.
export function countError(value) {
  if (value === undefined || value === '' || isCount(value, 0)) return '';
  return Number.isInteger(value) && value > MAX_COUNT ? 'nombre trop grand' : 'il faut un nombre entier, 0 ou plus';
}

// Premier champ numérique invalide de la configuration, '' s'il n'y en a pas.
export function numberError(cfg) {
  const fields = [[cfg.pause, 'Pause après chaque série']];
  for (const ex of cfg.exercises) {
    fields.push([ex.min, `${ex.name} › minimum`], [ex.max, `${ex.name} › maximum`], [ex.pause, `${ex.name} › pause`]);
  }
  for (const g of cfg.games) {
    for (const ev of g.events) {
      const at = `${g.name} › ${ev.name}`;
      fields.push([ev.draws, `${at} › exercices tirés`], [ev.maxPerSession, `${at} › limite par séance`], [ev.jokers, `${at} › jokers gagnés`]);
    }
  }
  for (const [value, label] of fields) {
    const error = countError(value);
    if (error) return `${label} : ${error}.`;
  }
  return '';
}

// Un champ obligatoire vidé vaut "" : on le ramène à 0 pour que le backend le refuse proprement.
// Un texte invalide est gardé pour que numberError le signale.
const orZero = (v) => (v === '' || v === undefined ? 0 : v);

export function normalized(cfg) {
  const c = clone(cfg);
  c.pause = orZero(c.pause);
  for (const ex of c.exercises) {
    ex.min = orZero(ex.min);
    ex.max = orZero(ex.max);
  }
  for (const g of c.games) {
    for (const ev of g.events) {
      ev.draws = orZero(ev.draws);
      ev.jokers = orZero(ev.jokers);
    }
  }
  return c;
}

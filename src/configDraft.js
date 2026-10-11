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

// Les jeux s'affichent par nom, quel que soit l'ordre enregistré.
export function byName(games) {
  return [...games].sort((a, b) => a.name.localeCompare(b.name, 'fr', { sensitivity: 'base' }));
}

// Plus grand nombre accepté par le backend (u32).
const MAX_COUNT = 4294967295;

export const isCount = (v, min) => Number.isInteger(v) && v >= min && v <= MAX_COUNT;

// Le backend refuse sans explication un nombre négatif, décimal ou trop grand : on le signale avant.
function countError(value, label) {
  if (value === undefined || isCount(value, 0)) return '';
  return Number.isInteger(value) && value > MAX_COUNT ? `${label} : nombre trop grand.` : `${label} : il faut un nombre entier, 0 ou plus.`;
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
    const error = countError(value, label);
    if (error) return error;
  }
  return '';
}

// Nombre facultatif saisi dans un champ : vidé, la clé disparaît.
export function setOptional(item, key, value) {
  if (value === '') delete item[key];
  else item[key] = Number(value);
}

// Un champ numérique vidé vaut "" : on le ramène à 0 pour que le backend le refuse proprement.
export function normalized(cfg) {
  const c = clone(cfg);
  c.pause = Number(c.pause) || 0;
  for (const ex of c.exercises) {
    ex.min = Number(ex.min) || 0;
    ex.max = Number(ex.max) || 0;
  }
  for (const g of c.games) {
    for (const ev of g.events) {
      ev.draws = Number(ev.draws) || 0;
      ev.jokers = Number(ev.jokers) || 0;
    }
  }
  return c;
}

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

// Un champ numérique vidé vaut "" : on le ramène à 0 pour que le backend le refuse proprement.
export function normalized(cfg) {
  const c = clone(cfg);
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

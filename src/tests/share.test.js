import { deflateSync, strToU8 } from 'fflate';
import { describe, expect, it } from 'vitest';
import { importGame, mergeExercises, parseExercises, parseGame, shareExercises, shareGame, unpack } from '../share.js';
import { config } from './app.js';

// Texte partagé écrit dans une version donnée du format, avec le contenu de `shared`.
function packed(version, shared, data = unpack(shared).data) {
  const payload = JSON.stringify({ version, kind: unpack(shared).kind, ...data });
  return shared.slice(0, shared.indexOf(':') + 1) + btoa(String.fromCharCode(...deflateSync(strToU8(payload))));
}

const exercises = [
  { id: 'squat', name: 'Squats', unit: 'rép.', min: 10, max: 15 },
  { id: 'pompes', name: 'Pompes', unit: 'rép.', min: 8, max: 12, pause: 90 },
];

describe('partage d\'un jeu', () => {
  it('partage le nom et les événements sans id ni exercices, puis les relit', () => {
    const text = shareGame(config.games[0]);
    expect(text).toMatch(/^squatsouls\/game\/elden-ring:/);
    expect(JSON.stringify(unpack(text).data)).not.toContain('"id"');
    expect(parseGame(text, 3)).toEqual({
      name: 'Elden Ring',
      events: [
        { id: 'mort', name: 'Mort', draws: 1, jokers: 0, maxPerSession: 15 },
        { id: 'boss-vaincu', name: 'Boss vaincu', draws: 1, jokers: 2 },
      ],
      clamped: false,
    });
  });

  it('ignore l\'identifiant placé devant le texte compressé', () => {
    const text = shareGame(config.games[0]);
    const payload = text.slice(text.indexOf(':') + 1);
    expect(parseGame(`squatsouls/exercises:${payload}`, 3)).toEqual(parseGame(text, 3));
    expect(parseGame(payload, 3)).toEqual(parseGame(text, 3));
  });

  it('partage l\'AppId Steam du jeu et relit un texte de la version 1, qui n\'en a pas', () => {
    const game = { ...config.games[0], steamAppId: 1245620 };
    expect(parseGame(shareGame(game), 3).steamAppId).toBe(1245620);
    const { steamAppId, ...v1 } = unpack(shareGame(game)).data;
    const read = parseGame(packed(1, shareGame(game), v1), 3);
    expect(read).toEqual(parseGame(shareGame(config.games[0]), 3));
    expect(read).not.toHaveProperty('steamAppId');
  });

  it('ramène le nombre tiré au nombre d\'exercices définis', () => {
    const text = shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 5, jokers: 0 }] });
    expect(parseGame(text, 2)).toMatchObject({ events: [{ draws: 2 }], clamped: true });
  });

  it('refuse un texte qui n\'est pas un jeu partagé valide', () => {
    const invalid = [
      'du texte',
      shareGame({ name: 'Nioh', events: [] }),
      shareGame({ name: ' ', events: [{ name: 'Mort', draws: 1 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 1, maxPerSession: 0 }] }),
      shareGame({ name: 'Nioh', events: [{ name: 'Mort', draws: 1, jokers: -1 }] }),
      shareGame({ name: 'Nioh', steamAppId: 0, events: [{ name: 'Mort', draws: 1 }] }),
      shareGame({ name: 'Nioh', steamAppId: '485510', events: [{ name: 'Mort', draws: 1 }] }),
    ];
    for (const text of invalid) expect(() => parseGame(text, 3)).toThrow('Ce texte n\'est pas un jeu partagé valide.');
  });

  it('indique où importer des exercices collés à la place d\'un jeu', () => {
    expect(() => parseGame(shareExercises(exercises), 3)).toThrow('Ce texte partage des exercices : l\'importer depuis l\'onglet Exercices, avec Importer.');
  });

  it('demande une mise à jour pour un texte d\'une version inconnue du format', () => {
    const text = packed(999, shareGame(config.games[0]));
    expect(() => parseGame(text, 3)).toThrow('Ce texte vient d\'une version plus récente de SquatSouls : la mettre à jour pour l\'importer.');
  });

  it('remplace le jeu du même nom en gardant son id, sinon l\'ajoute', () => {
    const events = [{ id: 'feu', name: 'Feu', draws: 1, jokers: 0 }];
    expect(importGame(config.games, { name: ' sekiro ', events }).map((g) => [g.id, g.events])).toEqual([
      ['er', config.games[0].events],
      ['sekiro', events],
    ]);
    expect(importGame(config.games, { name: 'Nioh', events }).at(-1)).toEqual({ id: 'nioh', name: 'Nioh', events });
  });

  it('reprend l\'AppId Steam importé, ou à défaut celui du jeu remplacé, même porté par un autre jeu', () => {
    const events = [{ id: 'feu', name: 'Feu', draws: 1, jokers: 0 }];
    const games = [{ ...config.games[0], steamAppId: 1245620 }, { ...config.games[1], steamAppId: 814380 }];
    expect(importGame(games, { name: 'Sekiro', events })[1].steamAppId).toBe(814380);
    expect(importGame(games, { name: 'Sekiro', steamAppId: 814381, events })[1].steamAppId).toBe(814381);
    expect(importGame(games, { name: 'Nioh', steamAppId: 485510, events }).at(-1).steamAppId).toBe(485510);
    expect(importGame(games, { name: 'Sekiro sans prothèse', steamAppId: 814380, events }).at(-1).steamAppId).toBe(814380);
  });
});

describe('partage des exercices', () => {
  it('partage les exercices sans id, avec leur pause propre, puis les relit', () => {
    const text = shareExercises(exercises);
    expect(text).toMatch(/^squatsouls\/exercises:/);
    expect(JSON.stringify(unpack(text).data)).not.toContain('"id"');
    expect(JSON.stringify(unpack(text).data)).toContain('"pause":90');
    expect(parseExercises(text)).toEqual(exercises.map(({ id, ...ex }) => ex));
    // Les espaces et retours à la ligne ajoutés par une messagerie sont ignorés.
    const cut = text.indexOf(':') + 10;
    expect(parseExercises(` ${text.slice(0, cut)}\n${text.slice(cut)} `)).toEqual(parseExercises(text));
  });

  it('refuse un texte qui n\'est pas une liste d\'exercices valide', () => {
    const invalid = [
      '',
      shareExercises([]),
      shareExercises([{ name: 'Squats', unit: 'rép.', min: 5, max: 0 }]),
      shareExercises([{ name: 'Squats', unit: 'rép.', min: 6, max: 5 }]),
      shareExercises([{ name: '', unit: 'rép.', min: 1, max: 5 }]),
      shareExercises([{ name: 'Squats', unit: 'rép.', min: 1, max: 5, pause: -1 }]),
    ];
    for (const text of invalid) expect(() => parseExercises(text)).toThrow('Ce texte n\'est pas une liste d\'exercices partagée valide.');
  });

  it('indique où importer un jeu collé à la place d\'exercices', () => {
    expect(() => parseExercises(shareGame(config.games[0]))).toThrow('Ce texte partage un jeu : l\'importer depuis l\'onglet Jeux, avec Importer un jeu.');
  });

  it('met à jour les exercices du même nom et ajoute les autres, sans toucher l\'original', () => {
    const imported = [
      { name: 'squats', unit: 's', min: 20, max: 30, pause: 45 },
      { name: 'Pompes', unit: 'rép.', min: 8, max: 12, pause: 90 },
      { name: 'Fentes', unit: 'rép.', min: 5, max: 8 },
    ];
    expect(mergeExercises(exercises, imported)).toEqual({
      exercises: [
        { id: 'squat', name: 'Squats', unit: 's', min: 20, max: 30, pause: 45 },
        exercises[1],
        { id: 'fentes', name: 'Fentes', unit: 'rép.', min: 5, max: 8 },
      ],
      added: 1,
      updated: 1,
    });
    // Sans pause propre dans l'import, l'exercice repasse à la pause globale.
    expect(mergeExercises(exercises, [{ name: 'Pompes', unit: 'rép.', min: 8, max: 12 }]).exercises[1]).toEqual({ id: 'pompes', name: 'Pompes', unit: 'rép.', min: 8, max: 12 });
    expect(exercises[0].unit).toBe('rép.');
  });
});

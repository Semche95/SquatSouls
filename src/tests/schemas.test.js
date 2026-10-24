import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync } from 'node:fs';
import Ajv from 'ajv/dist/2020.js';
import { catalogFiles } from '../../scripts/localized.js';

// Files of each language, found by name so that a new language is checked too, and the schema they all follow:
// the structure of a language comes from its schema, never from another language.
const kinds = [
  { name: 'interface texts', folder: '../locales/', file: /^([a-z]{2})\.json$/, schema: 'schema.json' },
  { name: 'site texts', folder: '../../site/locales/', file: /^([a-z]{2})\.json$/, schema: 'schema.json' },
];
const read = (folder, file) => JSON.parse(readFileSync(new URL(folder + file, import.meta.url), 'utf8'));

describe.each(kinds)('$name', ({ folder, file, schema }) => {
  const validate = new Ajv({ allErrors: true }).compile(read(folder, schema));
  const files = readdirSync(new URL(folder, import.meta.url)).filter((f) => file.test(f));

  it('exist at least in English and French', () => {
    expect(files.map((f) => f.match(file)[1])).toEqual(expect.arrayContaining(['en', 'fr']));
  });

  it.each(files)('%s follows the schema', (name) => {
    validate(read(folder, name));
    expect(validate.errors ?? []).toEqual([]);
  });
});

// The seed and the catalog are written once, each text in every language: their files follow their schemas, the
// catalog's sharing the definitions of the seed's.
describe('seed and catalog', () => {
  const seed = '../../src-tauri/src/app/config/';
  const catalog = '../../src-tauri/catalog/';
  const ajv = new Ajv({ allErrors: true }).addSchema(read(seed, 'seed.schema.json')).addSchema(read(catalog, 'schema.json'));
  const check = (schema, folder, file) => {
    const validate = ajv.getSchema(schema);
    validate(read(folder, file));
    expect(validate.errors ?? []).toEqual([]);
  };

  it('seed.json follows the schema', () => check('seed', seed, 'seed.json'));

  it.each(catalogFiles())('games/%s follows the schema', (file) => check('catalog', catalog, `games/${file}`));

  it('never gives the same id twice', () => {
    const games = catalogFiles().map((file) => read(catalog, `games/${file}`));
    const ids = [...read(seed, 'seed.json').exercises, ...games, ...games.flatMap((game) => game.events)].map((item) => item.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('is written at least in English and French', () => {
    expect(read(seed, 'seed.schema.json').$defs.languages.required).toEqual(expect.arrayContaining(['en', 'fr']));
  });
});

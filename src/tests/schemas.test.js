import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync } from 'node:fs';
import Ajv from 'ajv/dist/2020.js';
import { gameFiles } from '../../scripts/seed.js';

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

// The seed is written once, each text in every language: its files follow the parts of its schema.
describe('seed', () => {
  const folder = '../../src-tauri/src/app/seed/';
  const ajv = new Ajv({ allErrors: true }).addSchema(read(folder, 'schema.json'));
  const check = (part, file) => {
    const validate = ajv.getSchema(`seed#/$defs/${part}`);
    validate(read(folder, file));
    expect(validate.errors ?? []).toEqual([]);
  };

  it('exercises.json follows the schema', () => check('exercisesFile', 'exercises.json'));

  it.each(gameFiles())('games/%s follows the schema and is named after its game', (file) => {
    check('game', `games/${file}`);
    expect(`${read(folder, `games/${file}`).id}.json`).toBe(file);
  });

  it('is written at least in English and French', () => {
    expect(read(folder, 'schema.json').$defs.languages.required).toEqual(expect.arrayContaining(['en', 'fr']));
  });
});

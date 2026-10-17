import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync } from 'node:fs';
import Ajv from 'ajv/dist/2020.js';

// Files of each language, found by name so that a new language is checked too, and the schema they all follow:
// the structure of a language comes from its schema, never from another language.
const kinds = [
  { name: 'interface texts', folder: '../locales/', file: /^([a-z]{2})\.json$/, schema: 'schema.json' },
  { name: 'site texts', folder: '../../site/locales/', file: /^([a-z]{2})\.json$/, schema: 'schema.json' },
  { name: 'default configurations', folder: '../../src-tauri/src/app/', file: /^seed\.([a-z]{2})\.json$/, schema: 'seed.schema.json' },
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

// Seed and catalog read in one language, as the backend does it (app/config/seed.rs): the exercises of
// src-tauri/src/app/config/seed.json and every file of src-tauri/catalog/games/ in the order of their names, keeping
// only that language in the texts (`name`, `unit`).
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SEED = join(import.meta.dirname, '../src-tauri/src/app/config/seed.json');
const GAMES = join(import.meta.dirname, '../src-tauri/catalog/games');
const TEXTS = ['name', 'unit'];

const read = (file) => JSON.parse(readFileSync(file, 'utf8'));

export const catalogFiles = () => readdirSync(GAMES).filter((file) => file.endsWith('.json')).sort();

const localize = (value, language) => {
  if (Array.isArray(value)) return value.map((item) => localize(item, language));
  if (value === null || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value).map(([key, field]) => [
    key,
    TEXTS.includes(key) && field !== null && typeof field === 'object' ? field[language] : localize(field, language),
  ]));
};

// Seed in `language`: the global recovery and the exercises of the default configuration.
export const seed = (language) => localize(read(SEED), language);

// Games of the catalog in `language`.
export const catalogGames = (language) => catalogFiles().map((file) => localize(read(join(GAMES, file)), language));

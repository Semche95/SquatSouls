// Seed of one language, its exercises and every game of the catalog, assembled from src-tauri/src/app/seed/ as the backend does it
// (app/config/seed.rs): exercises.json and every file of games/ in the order of their names, keeping only
// that language in the texts (`name`, `unit`).
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const FOLDER = join(import.meta.dirname, '../src-tauri/src/app/seed');
const TEXTS = ['name', 'unit'];

const read = (file) => JSON.parse(readFileSync(join(FOLDER, file), 'utf8'));

export const gameFiles = () => readdirSync(join(FOLDER, 'games')).filter((file) => file.endsWith('.json')).sort();

const localize = (value, language) => {
  if (Array.isArray(value)) return value.map((item) => localize(item, language));
  if (value === null || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value).map(([key, field]) => [
    key,
    TEXTS.includes(key) && field !== null && typeof field === 'object' ? field[language] : localize(field, language),
  ]));
};

export const seed = (language) => localize({ ...read('exercises.json'), games: gameFiles().map((file) => read(`games/${file}`)) }, language);

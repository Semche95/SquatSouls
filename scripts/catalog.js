// Catalog of games published with each release: one catalog.<language>.json per language, the list of the games of
// src-tauri/catalog/ as share texts (src/sharing/share.js), so the app imports them as a pasted text, in the share
// format of that release. Run, writes them into the folder given as argument; the development server serves them
// (vite.config.js).
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createServer } from 'vite';
import { catalogGames } from './localized.js';

const root = join(import.meta.dirname, '..');
const schema = JSON.parse(readFileSync(join(root, 'src-tauri/src/app/config/seed.schema.json'), 'utf8'));

export const languages = schema.$defs.languages.required;

// Catalog in `language`, read from its files at each call. share.js imports the interface texts as JSON modules,
// which the Vite `server` loads as the app does.
export async function catalog(server, language) {
  const { shareGame } = await server.ssrLoadModule('/src/sharing/share.js');
  return `${JSON.stringify(catalogGames(language).map(shareGame))}\n`;
}

if (import.meta.main) {
  const server = await createServer({ root, server: { middlewareMode: true }, appType: 'custom', logLevel: 'error' });
  try {
    for (const language of languages) writeFileSync(join(process.argv[2], `catalog.${language}.json`), await catalog(server, language));
  } finally {
    await server.close();
  }
}

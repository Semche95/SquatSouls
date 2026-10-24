// Writes the catalog of games published with each release into the folder given as argument: one
// catalog.<language>.json per language of the default configuration, the list of its games as share texts
// (src/sharing/share.js), so the app imports them as a pasted text, in the share format of that release.
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createServer } from 'vite';
import { seed } from './seed.js';

const root = join(import.meta.dirname, '..');
const out = process.argv[2];
const schema = JSON.parse(readFileSync(join(root, 'src-tauri/src/app/seed/schema.json'), 'utf8'));

// share.js imports the interface texts as JSON modules, which Vite loads as the app does.
const server = await createServer({ root, server: { middlewareMode: true }, appType: 'custom', logLevel: 'error' });
try {
  const { shareGame } = await server.ssrLoadModule('/src/sharing/share.js');
  for (const language of schema.$defs.languages.required) {
    writeFileSync(join(out, `catalog.${language}.json`), `${JSON.stringify(seed(language).games.map(shareGame))}\n`);
  }
} finally {
  await server.close();
}

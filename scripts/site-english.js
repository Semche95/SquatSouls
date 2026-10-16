// Writes the English texts of site/locales/en.json into the index.html of the assembled site given as argument, so the
// page reads in English before site.js runs, or without it.
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = process.argv[2];
const texts = JSON.parse(readFileSync(join(dir, 'locales/en.json'), 'utf8'));
const escape = (text) => text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');
const page = join(dir, 'index.html');

// site.js gives the screenshots their src in the visitor's language: an English one in the page would also be
// downloaded by the other visitors. Without JavaScript, a copy in noscript shows instead.
const noscriptShot = (shot, key) =>
  `<noscript><img class="shot" src="screenshots/en/${shot}" alt="${escape(texts[key])}"></noscript>`;

writeFileSync(page, readFileSync(page, 'utf8')
  .replace(/(data-t="(\w+)"[^>]*>)</g, (_, tag, key) => `${tag}${escape(texts[key])}<`)
  .replace(/(id="cta"[^>]*>)</, (_, tag) => `${tag}${escape(texts.ctaAny)}<`)
  .replace(/<img[^>]*data-shot="([\w.-]+)"[^>]*data-alt="(\w+)"[^>]*>/g, (img, shot, key) => `${img}${noscriptShot(shot, key)}`));

# Contributing

Issues and pull requests are welcome. Before changing how the app behaves, read [docs/BEHAVIOR.md](docs/BEHAVIOR.md): it is the reference, and any change of behavior updates it in the same change.

## Rules

- Code, comments, test names and documentation are in English.
- No visible text in the code: every text goes through `t` and is added to `src/locales/schema.json` and to every language (`src/locales/en.json`, `src/locales/fr.json`). The Rust backend returns error codes, never translated text.
- French texts follow French typography: a no-break space (U+00A0) before `:`, a narrow no-break space (U+202F) before `?`, `!`, `;` and inside `« »`.
- A migration in `src-tauri/migrations/` is never modified once created: every schema change adds a new `V<n+1>__<name>.sql` file.
- Any change to the share format (`src/sharing/share.js`) increments `VERSION` and keeps the readers of the previous versions.
- In JavaScript, strings use single quotes.
- Tests live in `src/tests/` (interface) and `src-tauri/tests/` (backend).

## Layout

```
src/                     Vue 3 interface
  main.js                entry point
  components/            App.vue (header and tabs), one component per tab and its blocks
  composables/           shared state (provide/inject), backend calls, keyboard
  locales/               interface texts (en.json source and fallback, fr.json) and their schema
  i18n/                  vue-i18n setup, language switch, backend error messages, formatting
  config/                configuration draft, its limits and its summaries
  sharing/               share and import format, versioned, and the changes an import brings
  ui/                    reveal, select and tooltip behaviors
  session/               timer sounds
  logging/               interface errors written to the log file
  project/               links to the GitHub repository
  splash/                slogans shown under the title (English only), shared with the site
  style/                 palette and fonts shared with the site (theme.css), styles by domain imported in cascade order by index.css
  tests/                 interface tests (Vitest)
src-tauri/src/           Rust backend
  lib.rs                 startup and log files
  commands/              Tauri commands, by domain
  tray.rs                notification area icon
  app/                   business logic, seed (seed/: default exercises, one file per catalog game) and its schema
src-tauri/migrations/    SQL schema migrations
src-tauri/tests/         backend tests
docs/                    behavior reference, data and logs
site/                    presentation site, published on GitHub Pages by the Site workflow
```

## Adding a language

1. Add `src/locales/<code>.json` following `src/locales/schema.json` (the tests check it), and register it in `src/i18n/i18n.js` (with its plural rule if it differs from English).
2. Add it to the language list in `src/components/ConfigSettings.vue` and to the Steam Store languages in `src/i18n/locale.js`.
3. In `src-tauri/src/app/language.rs`, add it to `Language` and `LanguageSetting`, and to the system locale resolution.
4. Add it to the `languages` of `src-tauri/src/app/seed/schema.json`, then write its text beside the others in every name and unit of `src-tauri/src/app/seed/`.
5. For the site, add `site/locales/<code>.json` following `site/locales/schema.json`, its button in `site/index.html` and its code in `site/language.js`.
6. Update the Language section of [docs/BEHAVIOR.md](docs/BEHAVIOR.md).

## Before submitting

```bash
cd src-tauri && cargo test # backend tests
pnpm test                  # interface tests
pnpm lint                  # JavaScript and Vue lint
cd src-tauri && cargo clippy --all-targets -- -D warnings # Rust lint
```

Commits follow [Conventional Commits](https://www.conventionalcommits.org/), on a single line, one change per commit with its code, tests and documentation.

## Releasing

Set the version in `package.json` and `src-tauri/Cargo.toml`, then push a `v<version>` tag: the [Build workflow](https://github.com/Semche95/SquatSouls/actions/workflows/build.yml) creates a draft release, which the app offers once published. Its notes list the `feat`, `fix` and `perf` commits since the previous tag, then the `build` and `refactor` ones under Under the hood (`cliff.toml`). It signs the updates with the private key of the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repository secrets, matching the public key of `src-tauri/tauri.conf.json`.

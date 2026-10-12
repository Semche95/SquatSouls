# Contributing

Issues and pull requests are welcome. Before changing how the app behaves, read [docs/BEHAVIOR.md](docs/BEHAVIOR.md): it is the reference, and any change of behavior updates it in the same change.

## Rules

- Code, comments, test names and documentation are in English.
- No visible text in the code: every text goes through `t` and is added to both `src/locales/en.json` and `src/locales/fr.json`. The Rust backend returns error codes, never translated text.
- A migration in `src-tauri/migrations/` is never modified once created: every schema change adds a new `V<n+1>__<name>.sql` file.
- Any change to the share format (`src/share.js`) increments `VERSION` and keeps the readers of the previous versions.
- In JavaScript, strings use single quotes.
- Tests live in `src/tests/` (interface) and `src-tauri/tests/` (backend).

## Layout

```
src/                     Vue 3 interface
  App.vue                header and tabs
  components/            one component per tab and its blocks
  composables/           shared state (provide/inject), backend calls, keyboard
  locales/               interface texts (en.json source and fallback, fr.json)
  i18n.js                vue-i18n setup, language switch, backend error messages
  share.js               share and import format, versioned
  slogans.js             slogans shown under the title, in English only
  tests/                 interface tests (Vitest)
src-tauri/src/           Rust backend
  lib.rs                 startup and log files
  commands.rs            Tauri commands
  tray.rs                notification area icon
  app/                   business logic, default configuration (seed.en.json, seed.fr.json)
src-tauri/migrations/    SQL schema migrations
src-tauri/tests/         backend tests
docs/                    behavior reference, data and logs
```

## Adding a language

1. Add `src/locales/<code>.json` with every key of `en.json`, and register it in `src/i18n.js` (with its plural rule if it differs from English).
2. Add it to the language list in `src/components/ConfigSettings.vue` and to the Steam Store languages in `src/locale.js`.
3. In `src-tauri/src/app/language.rs`, add it to `Language` and `LanguageSetting`, and to the system locale resolution.
4. Add `src-tauri/src/app/seed.<code>.json` (same ids, numbers and AppIds as `seed.en.json`) and load it in `src-tauri/src/app/config.rs`.
5. Update the Language section of [docs/BEHAVIOR.md](docs/BEHAVIOR.md).

## Before submitting

```bash
cd src-tauri && cargo test # backend tests
pnpm test                  # interface tests
```

Commits follow [Conventional Commits](https://www.conventionalcommits.org/), on a single line, one change per commit with its code, tests and documentation.

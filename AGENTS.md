# AGENTS.md

Tauri 2 (Rust) + Vue 3 (Vite) desktop app. See `README.md` for usage, `docs/BEHAVIOR.md` for behavior and `docs/DATA-AND-LOGS.md` for storage and logs.

## Business rules

The business rules are described in `docs/BEHAVIOR.md`, which is the reference.

- Read it before changing the app's behavior.
- Update it in the same change as soon as a rule changes or is added (draw, limits, jokers, session, history, configuration validation).

## Structure

- `src/`: Vue interface (`App.vue` header and tabs, `components/` one component per tab and its blocks, `composables/` shared state via provide/inject, `format.js`, `configDraft.js` and `share.js` (game and exercise sharing) pure functions, `i18n.js` vue-i18n instance (`t`, current language, `setLanguage`, backend error messages), `locales/` translated texts (`en.json` source and fallback, `fr.json`), `locale.js` Steam Store language of the interface language, `log.js` interface errors sent to the log file, `slogans.js` slogans shown under the title, `tooltip.js` tooltip directive (floating-vue) and its options, `style.css`)
- `src-tauri/src/`: Rust backend (`lib.rs` startup and log files, `commands.rs` Tauri commands, `state.rs` shared state and configuration loading, `tray.rs` notification area icon and hidden window, `app/` business logic: `config.rs` configuration, `seed.en.json` and `seed.fr.json` default configuration by language, `language.rs` language setting and its resolution, `error.rs` errors sent as codes, `session.rs` draw and session, `history.rs` history summary, `steam.rs` detection of the running Steam game, `store.rs` SQLite storage)
- `src-tauri/migrations/`: SQL schema migrations, one per `V<n>__<name>.sql` file (rules in its `README.md`), dated in the database when applied. A migration is never modified once created, a new one is always added
- `src/tests/`: interface tests (Vitest), one file per tab or module; `app.js` shared data and mounting, `setup.js` Tauri backend mock
- `docs/`: `BEHAVIOR.md` business rules, `DATA-AND-LOGS.md` database, migrations and logs
- `src-tauri/tests/`: backend tests (Cargo integration tests on the public API of `app`), one file per module

## Commands

```bash
pnpm tauri dev          # run in development
pnpm tauri build        # production build
cd src-tauri && cargo test # backend tests
pnpm test                   # interface tests (Vitest)
```

## Conventions

- No visible text hardcoded in the interface: every text goes through `t`, and every key is added to both `src/locales/en.json` (source language) and `src/locales/fr.json`, with named placeholders and plurals rather than concatenated pieces. Rust holds no translated text: it returns error codes with their parameters (`AppError`), and the tray and notification texts come from the interface. Code (identifiers, comments, developer logs, test names) and documentation in English.
- Never modify, rename, merge or delete a migration in `src-tauri/migrations/` once created, even uncommitted, even to fix a mistake, not even its comments (refinery checksums their content): every schema change goes through a new `V<n+1>__<name>.sql` file. Migration comments are written in English from the start, so they never need translating.
- In JavaScript, strings in single quotes, never double, escaping the apostrophe (`'moins d\'une minute'`).
- A disabled button must look disabled: check that `style.css` gives it a `:disabled` style (reduced opacity, no pointer cursor nor hover effect), rather than relying on the `disabled` attribute alone. The reason it is disabled is shown in a tooltip by wrapping it in `DisabledTip` (`components/DisabledTip.vue`), never with a `title`.
- Any change to the share/import format (`share.js`) is versioned: increment `VERSION`, add the new version's reader to `READERS` while keeping those of the previous ones, and test importing texts from the old versions.
- Tests only live in `src/tests/` (interface) and `src-tauri/tests/` (backend): no `*.test.js` file elsewhere in `src/`, no `#[cfg(test)]` module in `src-tauri/src/`.
- Never mention in the sources (comments included) a file or folder ignored by git (`.claude/`, skills, `todo.md`): the reference would be broken for anyone cloning the repository.
- Follow the DRY rule everywhere (Rust, Vue, CSS, tests): factor out any duplicated logic, value or style rather than copying it.
- Run `cargo test` after any change to the Rust code, `pnpm test` after any change to `src/`.
- Never commit without the user's explicit agreement: accepting an operation that leads to it is not agreement; do the work locally, then ask.
- Never push (`git push`, force push included), even on explicit request: the user pushes themselves.
- Never commit the files in `.claude/` (skills, settings) unless explicitly asked: stage files one by one and check `git diff --cached --stat` before each commit.
- Commits in Conventional Commits, a single line, without `Co-Authored-By`.
- Each commit is consistent: a single change, with its code, tests and documentation, and the tests pass. Split by file hunk (not by whole file) when a file mixes several changes.

## Token economy

- Do not read `node_modules/`, `src-tauri/target/`, `src-tauri/gen/`, `dist/` or lock files (`package-lock.json`, `Cargo.lock`).
- Search with `grep` before opening a file, and only read the useful part of large files.
- Hand broad searches or verbose tasks (exploring several files, analyzing logs) to a subagent when only the conclusion matters: it keeps the details in its own context.
- Filter long outputs (`cargo test 2>&1 | tail`, `cargo build -q`).
- Short answers: no summary of the modified code, no rereading of a file just written.

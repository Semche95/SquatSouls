# AGENTS.md

Tauri 2 (Rust) + Vue 3 (Vite) desktop app. See `README.md` for usage, `docs/BEHAVIOR.md` for behavior and `docs/DATA-AND-LOGS.md` for storage and logs.

## Business rules

The business rules are described in `docs/BEHAVIOR.md`, which is the reference.

- Read it before changing the app's behavior.
- Update it in the same change as soon as a rule changes or is added (draw, limits, jokers, session, history, configuration validation).
- The app is meant to be kind and encouraging, never guilt-inducing: weigh every change, rule or text against that.

## Structure

- `src/`: Vue interface, one component per tab in `components/`, shared state in `composables/` (provide/inject). `ConfigFrame.vue` is the frame shared by the Program and Settings tabs. `locales/en.json` is the source and fallback language.
- `src-tauri/src/`: Rust backend. The business logic lives in `app/`, the default configuration in `app/seed.en.json` and `app/seed.fr.json`.
- `src-tauri/migrations/`: SQL schema migrations, one per `V<n>__<name>.sql` file (rules in its `README.md`).
- `src/tests/`: interface tests (Vitest), one file per tab or module; `app.js` shared data and mounting, `setup.js` Tauri backend mock.
- `src-tauri/tests/`: backend tests (Cargo integration tests on the public API of `app`), one file per module.

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
- The interface tests run in English and check the English texts, test data included. French only appears where the test is about French itself: switching the language, French plurals or formatting, French input the app must recognize.
- The CI also runs the tests on Windows, which refuses to delete an open file: in the backend tests, close every connection and `Store` on a database (`drop`, or a temporary that is not kept) before removing its file.
- Never mention in the sources (comments included) a file or folder that is not in the repository: the reference would be broken for anyone cloning the repository.
- Follow the DRY rule everywhere (Rust, Vue, CSS, tests): factor out any duplicated logic, value or style rather than copying it.
- Run `cargo test` after any change to the Rust code, `pnpm test` after any change to `src/`.
- Before asking for any validation, always write the complete proposal in the chat (structure, files, choices), then ask a short question. Never put the content to validate only in an interactive question.
- Never commit without the user's explicit agreement: accepting an operation that leads to it is not agreement; do the work locally, then ask.
- Never push (`git push`, force push included), even on explicit request: the user pushes themselves.
- Stage files one by one and check `git diff --cached --stat` before each commit.
- Commits in Conventional Commits, a single line, without `Co-Authored-By`.
- Each commit is consistent: a single change, with its code, tests and documentation, and the tests pass. Split by file hunk (not by whole file) when a file mixes several changes.

## Token economy

- Do not read `node_modules/`, `src-tauri/target/`, `src-tauri/gen/`, `dist/` or lock files (`package-lock.json`, `Cargo.lock`).
- Search with `grep` before opening a file, and only read the useful part of large files.
- Hand broad searches or verbose tasks (exploring several files, analyzing logs) to a subagent when only the conclusion matters: it keeps the details in its own context.
- Filter long outputs (`cargo test 2>&1 | tail`, `cargo build -q`).
- Short answers: no summary of the modified code, no rereading of a file just written.

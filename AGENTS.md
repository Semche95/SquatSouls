# AGENTS.md

Tauri 2 (Rust) + Vue 3 (Vite) desktop app. See `README.md` for usage, `docs/BEHAVIOR.md` for behavior and `docs/DATA-AND-LOGS.md` for storage and logs.

## Business rules

The business rules are described in `docs/BEHAVIOR.md`, which is the reference.

- Read it before changing the app's behavior.
- Update it in the same change as soon as a rule changes or is added (draw, limits, jokers, session, history, configuration validation).
- The app is meant to be kind and encouraging, never guilt-inducing, and to frame without imposing: it gives a structure, never an obligation, and each user stays free and responsible for their training. Weigh every change, rule or text against that.

## Structure

- `src/`: Vue interface, one component per tab in `components/`, shared state in `composables/` (provide/inject). `ConfigFrame.vue` is the frame shared by the Program and Settings tabs. `locales/en.json` is the source and fallback language. `style/`: styles, one file per domain, imported in cascade order by `style/index.css` (`wide.css`, the wide window layout, last).
- `src-tauri/src/`: Rust backend. The business logic lives in `app/`, the default configuration in `app/seed.en.json` and `app/seed.fr.json`.
- `src-tauri/migrations/`: SQL schema migrations, one per `V<n>__<name>.sql` file (rules in its `README.md`).
- `src/tests/`: interface tests (Vitest), one file per tab or module; `app.js` shared data and mounting, `setup.js` Tauri backend mock.
- `src-tauri/tests/`: backend tests (Cargo integration tests on the public API of `app`), one file per module.
- `site/`: presentation site published on GitHub Pages by `.github/workflows/site.yml`, which adds `src/theme.css` (palette and fonts shared with the app), `src/slogans.js`, the icon and `docs/screenshots/`. Its texts are in `site/locales/en.json` and `site/locales/fr.json`; the workflow writes the English ones into `index.html` with `scripts/site-english.js`, so the page reads before its JavaScript runs.

## Commands

```bash
pnpm tauri dev          # run in development
pnpm tauri build        # production build
cd src-tauri && cargo test # backend tests
pnpm test                   # interface tests (Vitest)
```

## Conventions

- No visible text hardcoded in the interface: every text goes through `t`, and every key is added to both `src/locales/en.json` (source language) and `src/locales/fr.json`, with named placeholders and plurals rather than concatenated pieces. Rust holds no translated text: it returns error codes with their parameters (`AppError`), and the tray and notification texts come from the interface. Code (identifiers, comments, developer logs, test names) and documentation in English.
- Each language is written the way it is naturally said, not translated word for word: the French and English versions of a text may differ in structure as long as they say the same thing (for example a statement in French, "Par défaut, celle de tous les exercices.", and an instruction in English, "Leave blank to use the general recovery.").
- French texts (app, site and seed) address the user consistently: "tu" when the app talks to them in sentences (onboarding, safety tips, end of session, About, empty states, hints that tell how to do something, the whole site); the infinitive for actions (buttons, menus, labels, placeholders, tooltips of disabled buttons) and for the remedy in errors and system messages, which stay factual ("Libérer de la place, puis réessayer."); an impersonal form ("on", "il faut", passive) to state a rule or a fact, never to replace "tu" in a sentence addressed to the user. Never "vous". Texts in every language speak to everyone: no gendered word, and in French no adjective or participle agreeing with the user ("Un boss te mène la vie dure ?" rather than "Bloqué sur un boss ?").
- French texts (`fr.json` locales, `seed.fr.json`) follow French typography: a no-break space (U+00A0) before `:`, a narrow no-break space (U+202F) before `?`, `!`, `;` and inside `« »`, never an ordinary space.
- Never modify, rename, merge or delete a migration in `src-tauri/migrations/` once created, even uncommitted, even to fix a mistake, not even its comments (refinery checksums their content): every schema change goes through a new `V<n+1>__<name>.sql` file. Migration comments are written in English from the start, so they never need translating.
- In JavaScript, strings in single quotes, never double, escaping the apostrophe (`'moins d\'une minute'`).
- A disabled button must look disabled: check that `style/base.css` gives it a `:disabled` style (reduced opacity, no pointer cursor nor hover effect), rather than relying on the `disabled` attribute alone. The reason it is disabled is shown in a tooltip by wrapping it in `DisabledTip` (`components/DisabledTip.vue`), never with a `title`.
- Any change to the share/import format (`share.js`) is versioned: increment `VERSION`, add the new version's reader to `READERS` while keeping those of the previous ones, and test importing texts from the old versions.
- Tests only live in `src/tests/` (interface) and `src-tauri/tests/` (backend): no `*.test.js` file elsewhere in `src/`, no `#[cfg(test)]` module in `src-tauri/src/`.
- The interface tests run in English and check the English texts, test data included. French only appears where the test is about French itself: switching the language, French plurals or formatting, French input the app must recognize.
- The CI also runs the tests on Windows, which refuses to delete an open file: in the backend tests, close every connection and `Store` on a database (`drop`, or a temporary that is not kept) before removing its file.
- Never mention in the sources (comments included) a file or folder that is not in the repository: the reference would be broken for anyone cloning the repository.
- Follow the DRY rule everywhere (Rust, Vue, CSS, tests): factor out any duplicated logic, value or style rather than copying it.
- One file, one responsibility: if describing a file requires "and", split it. Past about 300 lines in a source file (Rust, JS, Vue) or 400 in a test file, check before adding to it whether it holds several responsibilities, and extract them as part of the change rather than growing it further. The length is a trigger to look, not a limit in itself.
- Split by domain, never into generic `utils`, `helpers` or `common` files: a Rust module that grows becomes a folder (`app/session/` with `mod.rs` and one file per concern), a composable that mixes concerns becomes several, and `commands.rs` is grouped by domain. Test files follow the same rule.
- A split is its own commit: a pure move with no behavior change, tests passing before and after, never mixed with a feature or a fix.
- The `//!` comment of a Rust `mod.rs` documents the whole folder and lists its concerns, as `app/mod.rs` does: update it in the same change as soon as a file of the folder is added, removed or changes concern.
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

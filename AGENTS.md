# AGENTS.md

Tauri 2 (Rust) + Vue 3 (Vite) desktop app. See `README.md` for usage and `docs/DATA-AND-LOGS.md` for storage and logs.

Keep this file up to date in the same change as soon as a change makes it inaccurate or incomplete: structure, commands, conventions or file names it mentions.

## Business rules

The business rules are described in `docs/BEHAVIOR.md`, which is the reference.

- Read it before changing the app's behavior.
- Update it in the same change as soon as a rule changes or is added (draw, limits, jokers, session, history, configuration validation).
- The app is meant to be kind and encouraging, never guilt-inducing, and to frame without imposing: it gives a structure, never an obligation, and each user stays free and responsible for their training. Weigh every change, rule or text against that. This framing is about the people who train, not about the configuration: a technical constraint is not a structure given to someone, and is kept only where the app needs it to work.

## Structure

- `src/`: Vue interface, grouped by domain around its single entry point `main.js`. One component per tab in `components/` (`App.vue` the root, `ConfigFrame.vue` the frame shared by the Program and Settings tabs), shared state in `composables/` (provide/inject); `i18n/` the language setup and formatting, `config/` the configuration draft and its summaries, `sharing/` the share format and import changes, `ui/` the reveal, select, tooltip and stuck bar behaviors, `session/` the sounds, `logging/` the error log, `project/` the repository links. `locales/en.json` is the source and fallback language, `locales/schema.json` the structure every language follows. `splash/slogans.js`: the slogans shown under the title, shared with the site, in English only. `style/`: styles, `theme.css` the palette and fonts shared with the site, then one file per domain, imported in cascade order by `style/index.css` (`wide.css`, the wide window layout, last).
- `src-tauri/src/`: Rust backend. The business logic lives in `app/`, the default configuration in `app/seed.en.json` and `app/seed.fr.json`, whose structure is `app/seed.schema.json`.
- `src-tauri/migrations/`: SQL schema migrations, one per `V<n>__<name>.sql` file (rules in its `README.md`).
- `src/tests/`: interface tests (Vitest), one file per tab or module; `app.js` shared data, mounting and the save bar, `program.js` the Program tab's game, sub-tabs and edit dialogs, `setup.js` Tauri backend mock, `schemas.test.js` the language files against their schemas.
- `src-tauri/tests/`: backend tests (Cargo integration tests on the public API of `app`, and of `state` for what links the session to the store), one file per module.
- `site/`: presentation site published on GitHub Pages by `.github/workflows/site.yml`, which adds `src/style/theme.css` (palette and fonts shared with the app), `src/splash/slogans.js`, the icon and `docs/screenshots/`. Its texts are in `site/locales/en.json` and `site/locales/fr.json`, following `site/locales/schema.json`; the workflow writes the English ones into `index.html` with `scripts/site-english.js`, so the page reads before its JavaScript runs.
- `patches/`: fixes of dependencies applied by pnpm on install (`pnpm patch`), listed in `pnpm-workspace.yaml`, one per bug not fixed upstream yet: `floating-vue` resizing a tooltip already gone (covered by `src/tests/tooltip.test.js`). Check whether a new version of the dependency still needs it before upgrading it.

## Commands

```bash
pnpm tauri dev          # run in development
pnpm tauri build        # production build
cd src-tauri && cargo test # backend tests, after any change to the Rust code
pnpm test                   # interface tests (Vitest), after any change to src/
pnpm lint                   # ESLint on the JavaScript and Vue files, after any change to them
cd src-tauri && cargo clippy --all-targets -- -D warnings # Rust lint, after any change to the Rust code
```

## Conventions

- No visible text hardcoded in the interface: every text goes through `t`, and every key is added to every language, `src/locales/en.json` and `src/locales/fr.json`, with named placeholders and plurals rather than concatenated pieces. Rust holds no translated text: it returns error codes with their parameters (`AppError`), and the tray and notification texts come from the interface. Code (identifiers, comments, developer logs, test names) and documentation in English.
- A new language starts from the schemas (`src/locales/schema.json`, `site/locales/schema.json`, `src-tauri/src/app/seed.schema.json`), never from another language's file. Any change of structure (key, placeholder, plural, seed field) goes into its schema in the same change; `schemas.test.js` checks every language file against it, and the backend tests check `seed.schema.json` against the configuration structures and limits.
- Each language is written the way it is naturally said, not translated word for word: the French and English versions of a text may differ in structure as long as they say the same thing (for example a statement in French, "Par défaut, celle de tous les exercices.", and an instruction in English, "Leave blank to use the general recovery.").
- French texts (app, site and seed) address the user consistently: "tu" when the app talks to them in sentences (onboarding, safety tips, end of session, About, empty states, hints that tell how to do something, the whole site); the infinitive for actions (buttons, menus, labels, placeholders, tooltips of disabled buttons) and for the remedy in errors and system messages, which stay factual ("Libérer de la place, puis réessayer."); an impersonal form ("on", "il faut", passive) to state a rule or a fact, never to replace "tu" in a sentence addressed to the user. Never "vous". Texts in every language speak to everyone: no gendered word, and in French no adjective or participle agreeing with the user ("Un boss te mène la vie dure ?" rather than "Bloqué sur un boss ?").
- French texts (`fr.json` locales, `seed.fr.json`) follow French typography: a no-break space (U+00A0) before `:`, a narrow no-break space (U+202F) before `?`, `!`, `;` and inside `« »`, never an ordinary space.
- Never modify, rename, merge or delete a migration in `src-tauri/migrations/` once created, even uncommitted, even to fix a mistake, not even its comments (refinery checksums their content): every schema change goes through a new `V<n+1>__<name>.sql` file. Migration comments are written in English from the start, so they never need translating.
- In JavaScript, strings in single quotes, never double, escaping the apostrophe (`'moins d\'une minute'`).
- Every checkbox is a switch, never a native checkbox: `style/controls.css` styles every `input type="checkbox"` as one, for the whole app.
- Never mark a state or an emphasis with a thicker or colored left border (accent bar, side stripe): use the background, the text color or an existing control instead.
- A disabled button must look disabled: check that `style/base.css` gives it a `:disabled` style (reduced opacity, no pointer cursor nor hover effect), rather than relying on the `disabled` attribute alone. The reason it is disabled is shown in a tooltip by wrapping it in `DisabledTip` (`components/DisabledTip.vue`), never with a `title`.
- Any change to the share/import format (`share.js`) is versioned: increment `VERSION`, add the new version's reader to `READERS` while keeping those of the previous ones, and test importing texts from the old versions.
- Tests only live in `src/tests/` (interface) and `src-tauri/tests/` (backend): no `*.test.js` file elsewhere in `src/`, no `#[cfg(test)]` module in `src-tauri/src/`.
- The interface tests run in English and check the English texts, test data included. French only appears where the test is about French itself: switching the language, French plurals or formatting, French input the app must recognize.
- The CI also runs the tests on Windows, which refuses to delete an open file: in the backend tests, close every connection and `Store` on a database (`drop`, or a temporary that is not kept) before removing its file.
- Never mention in the sources (comments included) a file or folder that is not in the repository: the reference would be broken for anyone cloning the repository.
- Follow the DRY rule everywhere (Rust, Vue, CSS, tests): factor out any duplicated logic, value or style rather than copying it.
- One file, one responsibility: if describing a file requires "and", split it. Past about 300 lines in a source file (Rust, JS, Vue) or 400 in a test file, check before adding to it whether it holds several responsibilities, and extract them as part of the change rather than growing it further. The length is a trigger to look, not a limit in itself.
- Split by domain, never into generic `utils`, `helpers` or `common` files: a Rust module that grows becomes a folder (`app/session/` with `mod.rs` and one file per concern), a composable that mixes concerns becomes several, and `commands/` is grouped by domain. Test files follow the same rule.
- A split is its own commit: a pure move with no behavior change, tests passing before and after, never mixed with a feature or a fix.
- The `//!` comment of a Rust `mod.rs` documents the whole folder and lists its concerns, as `app/mod.rs` does: update it in the same change as soon as a file of the folder is added, removed or changes concern.
- Before asking for any validation, always write the complete proposal in the chat (structure, files, choices), then ask a short question. Never put the content to validate only in an interactive question.
- Never commit without the user's explicit agreement: accepting an operation that leads to it is not agreement; do the work locally, then ask.
- Never push (`git push`, force push included), even on explicit request: the user pushes themselves.
- Stage files one by one and check `git diff --cached --stat` before each commit.
- Commits in Conventional Commits, a single line, without `Co-Authored-By`.
- Each commit is consistent: a single change, with its code, tests and documentation, and the tests pass. Split by file hunk (not by whole file) when a file mixes several changes.

## Complete implementations

- A rule or constraint about a state of the app (for example "the menu is locked during a session") is implemented on the state itself, never on the gesture that triggers it: not on a given button, click or handler, but on the source of truth the interface depends on (store, shared state, state machine).
- Before implementing, identify every path that produces or changes that state (buttons, keyboard shortcuts, routes, system events, session resume, restore at startup, calls from the backend...) and list them in the answer. Every existing path is handled; a path added later inherits the behavior without extra code, since the rule is carried by the state, not by the entry point.
- No workaround that satisfies the request on the surface without honoring its intent: a condition added in a single handler, a duplicated local flag, a visual-only disabling with no guard in the logic, a hardcoded value to pass a test case. If the clean solution needs a refactor or touches several files, do it, or stop and explain why rather than deliver a hack.
- When the request is ambiguous about its scope ("when the session starts": which starts?), take the broadest reading consistent with its intent and say so in one sentence, rather than silently picking the narrowest one.
- When adding a new entry point to an existing behavior, first look for every constraint already applied to the existing entry points (`grep` on the state concerned, not only on the button name) and check that the new path honors them. Add a test per entry path, or failing that a described manual check.
- At the end of the task, state explicitly the paths covered, the paths not covered if any, and every compromise made. Never present as done a work that covers only part of the paths.

## Token economy

- Do not read `node_modules/`, `src-tauri/target/`, `src-tauri/gen/`, `dist/` or lock files (`pnpm-lock.yaml`, `Cargo.lock`).
- Search with `grep` before opening a file, and only read the useful part of large files.
- Hand broad searches or verbose tasks (exploring several files, analyzing logs) to a subagent when only the conclusion matters: it keeps the details in its own context.
- Filter long outputs (`cargo test 2>&1 | tail`, `cargo build -q`).
- Short answers: no summary of the modified code, no rereading of a file just written.

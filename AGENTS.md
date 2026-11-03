# AGENTS.md

Tauri 2 (Rust) + Vue 3 (Vite) desktop app. See `README.md` for usage and `docs/DATA-AND-LOGS.md` for storage and logs.

Keep this file and the rules of `.claude/rules/` up to date in the same change as soon as a change makes them inaccurate or incomplete: structure, commands, conventions or file names they mention.

## Business rules

The business rules are described in `docs/BEHAVIOR.md`, which is the reference.

- Read it before changing the app's behavior.
- Update it in the same change as soon as a rule changes or is added (draw, limits, jokers, session, history, configuration validation).
- The app is meant to be kind and encouraging, never guilt-inducing, and to frame without imposing: it gives a structure, never an obligation, and each user stays free and responsible for their training. Weigh every change, rule or text against that. This framing is about the people who train, not about the configuration: a technical constraint is not a structure given to someone, and is kept only where the app needs it to work.

## Structure

- `src/`: Vue interface, grouped by domain around `main.js`; one component per tab in `components/`, shared state in `composables/` (provide/inject). `locales/en.json` is the source and fallback language, `locales/schema.json` the structure every language follows. `splash/slogans.js` (English only) and `style/theme.css` are shared with the site. `style/index.css` imports the styles in cascade order, `wide.css` last, with the same width as `ui/wide.js`.
- `src-tauri/src/`: Rust backend, business logic in `app/`. `app/config/seed.json` (structure `seed.schema.json`, every language) holds the exercises a new database receives.
- `src-tauri/catalog/`: the game catalog, one file per game in `games/` (structure `schema.json`, every language). Not built into the app: `scripts/catalog.js` makes one catalog per language, published with each release.
- `src-tauri/migrations/`: SQL migrations, one per `V<n>__<name>.sql` file (rules in its `README.md`).
- `src/tests/`: interface tests (Vitest), `setup.js` the Tauri backend mock, `app.js` and `program.js` the shared mounting and data. `src-tauri/tests/`: backend integration tests, `catalog_config/` their configuration (default one and every catalog game).
- `site/`: presentation site published by `.github/workflows/site.yml`, texts in `site/locales/`.
- `patches/`: pnpm patches of dependencies, one per bug not fixed upstream yet (`floating-vue`, covered by `src/tests/tooltip.test.js`). Check whether a new version still needs it before upgrading the dependency.

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

Topic conventions are in `.claude/rules/`, one file per topic (languages, French texts, migrations, interface, share format, tests, git); the `paths:` of a rule list the files it applies to. Read the rules matching the files you change.

- Code (identifiers, comments, developer logs, test names) and documentation in English.
- In JavaScript, strings in single quotes, never double, escaping the apostrophe (`'moins d\'une minute'`).
- Never mention in the sources (comments included) a file or folder that is not in the repository: the reference would be broken for anyone cloning the repository.
- Follow the DRY rule everywhere (Rust, Vue, CSS, tests): factor out any duplicated logic, value or style rather than copying it.
- One file, one responsibility: if describing a file requires "and", split it. Past about 300 lines in a source file (Rust, JS, Vue) or 400 in a test file, check before adding to it whether it holds several responsibilities, and extract them as part of the change rather than growing it further. The length is a trigger to look, not a limit in itself.
- Split by domain, never into generic `utils`, `helpers` or `common` files: a Rust module that grows becomes a folder (`app/session/` with `mod.rs` and one file per concern), a composable that mixes concerns becomes several, and `commands/` is grouped by domain. Test files follow the same rule.
- A split is its own commit: a pure move with no behavior change, tests passing before and after, never mixed with a feature or a fix.
- The `//!` comment of a Rust `mod.rs` documents the whole folder and lists its concerns, as `app/mod.rs` does: update it in the same change as soon as a file of the folder is added, removed or changes concern.
- Before asking for any validation, always write the complete proposal in the chat (structure, files, choices), then ask a short question. Never put the content to validate only in an interactive question.

## Complete implementations

- A rule about a state of the app (for example "the menu is locked during a session") is carried by the state itself (store, shared state, state machine), never by the gesture that triggers it. Before implementing, list in the answer every path that produces or changes that state (buttons, shortcuts, system events, session resume, restore at startup, backend calls...): each one is handled, and a later path inherits the rule.
- No surface workaround: no condition in a single handler, duplicated local flag, visual-only disabling or value hardcoded for a test. If the clean solution needs a refactor, do it, or stop and explain why. An ambiguous scope takes the broadest reading consistent with the intent, said in one sentence.
- A new entry point to an existing behavior honors every constraint of the existing ones (`grep` on the state, not on the button), with a test per path or a described manual check. At the end, state the paths covered and not covered and every compromise.

## Token economy

- Do not read `node_modules/`, `src-tauri/target/`, `src-tauri/gen/`, `dist/` or lock files (`pnpm-lock.yaml`, `Cargo.lock`).
- Search with `grep` before opening a file, and only read the useful part of large files.
- Filter long outputs (`cargo test 2>&1 | tail`, `cargo build -q`).
- Short answers: no summary of the modified code, no rereading of a file just written.

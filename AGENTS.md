# AGENTS.md

Appli desktop Tauri 2 (Rust) + Vue 3 (Vite). Voir `README.md` pour l'usage.

## Structure

- `src/` : interface Vue (`App.vue`, `main.js`, `style.css`)
- `src-tauri/src/` : backend Rust (`lib.rs` commandes Tauri, `fitness.rs` configuration, tirage et séance)

## Commandes

```bash
npm run tauri dev          # lancer en développement
npm run tauri build        # build de production
cd src-tauri && cargo test # tests
```

## Conventions

- Textes de l'interface et documentation en français.
- Lancer `cargo test` après toute modification du code Rust.
- Commits en Conventional Commits, une seule ligne, sans `Co-Authored-By`.

## Économie de tokens

- Ne pas lire `node_modules/`, `src-tauri/target/`, `src-tauri/gen/`, `dist/` ni les fichiers de verrouillage (`package-lock.json`, `Cargo.lock`).
- Chercher avec `grep` avant d'ouvrir un fichier, et ne lire que la partie utile des gros fichiers.
- Confier à un sous-agent les recherches larges ou les tâches verbeuses (exploration de plusieurs fichiers, analyse de logs) quand seule la conclusion compte : il garde le détail dans son propre contexte.
- Filtrer les sorties longues (`cargo test 2>&1 | tail`, `cargo build -q`).
- Réponses courtes : pas de récapitulatif du code modifié, pas de relecture d'un fichier qu'on vient d'écrire.

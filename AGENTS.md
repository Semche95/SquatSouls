# AGENTS.md

Appli desktop Tauri 2 (Rust) + Vue 3 (Vite). Voir `README.md` pour l'usage.

## Règles métier

Les règles métier sont décrites dans la section « Règles métier » de `README.md`, qui fait référence.

- La lire avant de modifier le comportement de l'appli.
- La mettre à jour dans la même modification dès qu'une règle change ou s'ajoute (tirage, limites, jokers, séance, historique, validation de la configuration, configuration par défaut).

## Structure

- `src/` : interface Vue (`App.vue` en-tête et onglets, `components/` un composant par onglet et ses blocs, `composables/` état partagé via provide/inject, `format.js` et `configDraft.js` fonctions pures, `style.css`)
- `src-tauri/src/` : backend Rust (`lib.rs` lancement, `commands.rs` commandes Tauri, `state.rs` état partagé et chargement de la configuration, `fitness/` logique métier : `config.rs` configuration, `session.rs` tirage et séance, `history.rs` historique et bilan)
- `src-tauri/tests/` : tests du backend (tests d'intégration Cargo sur l'API publique de `fitness`), un fichier par module

## Commandes

```bash
pnpm tauri dev          # lancer en développement
pnpm tauri build        # build de production
cd src-tauri && cargo test # tests backend
pnpm test                   # tests interface (Vitest)
```

## Conventions

- Textes de l'interface et documentation en français.
- Un bouton désactivé doit se voir comme tel : vérifier que `style.css` lui donne un style `:disabled` (opacité réduite, pas de curseur de clic ni d'effet au survol), sans se contenter de l'attribut `disabled`.
- Suivre la règle DRY partout (Rust, Vue, CSS, tests) : factoriser toute logique, valeur ou style dupliqué plutôt que le recopier.
- Lancer `cargo test` après toute modification du code Rust, `pnpm test` après toute modification de `src/`.
- Commits en Conventional Commits, une seule ligne, sans `Co-Authored-By`.

## Économie de tokens

- Ne pas lire `node_modules/`, `src-tauri/target/`, `src-tauri/gen/`, `dist/` ni les fichiers de verrouillage (`package-lock.json`, `Cargo.lock`).
- Chercher avec `grep` avant d'ouvrir un fichier, et ne lire que la partie utile des gros fichiers.
- Confier à un sous-agent les recherches larges ou les tâches verbeuses (exploration de plusieurs fichiers, analyse de logs) quand seule la conclusion compte : il garde le détail dans son propre contexte.
- Filtrer les sorties longues (`cargo test 2>&1 | tail`, `cargo build -q`).
- Réponses courtes : pas de récapitulatif du code modifié, pas de relecture d'un fichier qu'on vient d'écrire.

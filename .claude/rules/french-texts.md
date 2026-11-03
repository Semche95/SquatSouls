---
paths:
  - "src/locales/fr.json"
  - "site/locales/fr.json"
  - "src-tauri/src/app/config/seed.json"
  - "src-tauri/catalog/games/**"
---

# French texts

- French texts (app, site, seed and catalog) address the user consistently: "tu" when the app talks to them in sentences (onboarding, safety tips, end of session, About, empty states, hints that tell how to do something, the whole site); the infinitive for actions (buttons, menus, labels, placeholders, tooltips of disabled buttons) and for the remedy in errors and system messages, which stay factual ("Libérer de la place, puis réessayer."); an impersonal form ("on", "il faut", passive) to state a rule or a fact, never to replace "tu" in a sentence addressed to the user. Never "vous". No adjective or participle agreeing with the user ("Un boss te mène la vie dure ?" rather than "Bloqué sur un boss ?").
- French texts (`fr.json` locales, the `fr` texts of the seed and the catalog) follow French typography: a no-break space (U+00A0) before `:`, a narrow no-break space (U+202F) before `?`, `!`, `;` and inside `« »`, never an ordinary space.

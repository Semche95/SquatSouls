---
paths:
  - "src-tauri/migrations/**"
---

# Migrations

- Never modify, rename, merge or delete a migration in `src-tauri/migrations/` once created, even uncommitted, even to fix a mistake, not even its comments (refinery checksums their content): every schema change goes through a new `V<n+1>__<name>.sql` file. Migration comments are written in English from the start, so they never need translating.

# SQLite schema migrations

Applied at launch by refinery, in `V<n>` order, and dated in the `refinery_schema_history` table.

**A migration is never modified once created** (neither renamed, merged nor deleted, not even its comments: refinery checksums its content), even before being committed and even to fix a mistake: every schema change goes through a new `V<n+1>__<name>.sql` file. Comments are written in English from the start.

The comments of V4 and V5 were translated to English once: `PRE_TRANSLATION_CHECKSUMS` in `store/schema.rs` replaces their former checksums in databases that applied them before.

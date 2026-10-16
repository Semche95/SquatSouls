//! Schema migrations, and the checksums of the migrations translated to English.

use refinery::Target;
use rusqlite::{Connection, Result};

// Schema migrations (`migrations/V<n>__<name>.sql`), dated in `refinery_schema_history`
// when applied. A migration is never modified once created, even before being committed:
// every schema change, fixes included, goes through a new migration.
refinery::embed_migrations!("migrations");

/// Applies in a transaction the migrations the database has not received yet,
/// and refuses a database modified by an unknown migration, from a newer version of the app.
pub(super) fn migrate(conn: &mut Connection) -> Result<()> {
    let run = |conn: &mut Connection, target| {
        migrations::runner()
            .set_grouped(true)
            .set_target(target)
            .run(conn)
            .map(drop)
            .map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                    Some(format!("database schema not updated: {e}")),
                )
            })
    };
    // A database migrated before refinery records in `user_version` the number of migrations already
    // applied: they are added to the history without being replayed.
    let legacy: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if legacy > 0 {
        run(conn, Target::FakeVersion(legacy))?;
        conn.pragma_update(None, "user_version", 0)?;
    }
    update_translated_checksums(conn)?;
    run(conn, Target::Latest)
}

/// Checksums of the migrations as applied before their comments were translated to English, by version.
pub const PRE_TRANSLATION_CHECKSUMS: [(i32, u64); 2] =
    [(4, 13746842255044008563), (5, 15443599466271548012)];

/// Replaces in the history the pre-translation checksums with the current ones, so that refinery
/// does not refuse databases that applied those migrations before.
fn update_translated_checksums(conn: &Connection) -> Result<()> {
    let has_history: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE name = 'refinery_schema_history')",
        [],
        |r| r.get(0),
    )?;
    if !has_history {
        return Ok(());
    }
    let runner = migrations::runner();
    for (version, old) in PRE_TRANSLATION_CHECKSUMS {
        let current = runner
            .get_migrations()
            .iter()
            .find(|m| m.version() == version)
            .expect("translated migration")
            .checksum();
        conn.execute(
            "UPDATE refinery_schema_history SET checksum = ?1 WHERE version = ?2 AND checksum = ?3",
            (current.to_string(), version, old.to_string()),
        )?;
    }
    Ok(())
}

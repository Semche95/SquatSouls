//! Copy of the database for the next launch: made at launch before the migrations and regularly while
//! the app runs, put back in place of a damaged database at launch, and otherwise the reason it cannot
//! be opened.

use super::Store;
use chrono::Local;
use rusqlite::{Connection, ErrorCode, OpenFlags};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Files SQLite keeps beside a database, moved with it.
const COMPANIONS: [&str; 3] = ["-journal", "-wal", "-shm"];

/// Why the database cannot be opened, as told to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenFailure {
    /// Damaged, without a usable copy of the previous launch.
    Damaged,
    /// Read-only, not allowed or used by another program.
    Denied,
    DiskFull,
    /// The migrations failed, or the database comes from a newer version: an update brings the fix.
    NotPrepared,
}

impl OpenFailure {
    /// Code translated by the interface.
    pub fn code(self) -> &'static str {
        match self {
            OpenFailure::Damaged => "database_damaged",
            OpenFailure::Denied => "database_denied",
            OpenFailure::DiskFull => "database_full",
            OpenFailure::NotPrepared => "database_not_prepared",
        }
    }

    pub fn of_io(e: &io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::StorageFull => OpenFailure::DiskFull,
            _ => OpenFailure::Denied,
        }
    }

    fn of_sqlite(e: &rusqlite::Error) -> Self {
        match e.sqlite_error_code() {
            Some(ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase) => OpenFailure::Damaged,
            Some(
                ErrorCode::ReadOnly
                | ErrorCode::CannotOpen
                | ErrorCode::PermissionDenied
                | ErrorCode::DatabaseBusy
                | ErrorCode::DatabaseLocked,
            ) => OpenFailure::Denied,
            Some(ErrorCode::DiskFull) => OpenFailure::DiskFull,
            _ => OpenFailure::NotPrepared,
        }
    }
}

/// Reason the database cannot be opened, with the error behind it for the log.
#[derive(Debug)]
pub struct LaunchError {
    pub failure: OpenFailure,
    pub cause: String,
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self.failure, self.cause)
    }
}

impl From<rusqlite::Error> for LaunchError {
    fn from(e: rusqlite::Error) -> Self {
        LaunchError {
            failure: OpenFailure::of_sqlite(&e),
            cause: e.to_string(),
        }
    }
}

impl From<io::Error> for LaunchError {
    fn from(e: io::Error) -> Self {
        LaunchError {
            failure: OpenFailure::of_io(&e),
            cause: e.to_string(),
        }
    }
}

/// Database opened at launch.
pub struct Launched {
    pub store: Store,
    /// The database was damaged and replaced by the copy of the previous launch.
    pub restored: bool,
    /// The copy for the next launch could not be made: the launch goes on without it.
    pub copy_error: Option<rusqlite::Error>,
}

/// Copy made at each launch: `squatsouls.db` gives `squatsouls.previous.db`.
pub fn previous_path(path: &Path) -> PathBuf {
    sibling(path, "previous")
}

/// Damaged database set aside, never deleted: `squatsouls.damaged-20261017-143000.db`.
fn damaged_path(path: &Path) -> PathBuf {
    sibling(
        path,
        &format!("damaged-{}", Local::now().format("%Y%m%d-%H%M%S")),
    )
}

fn sibling(path: &Path, tag: &str) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!("{stem}.{tag}.db"))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Opens the database and checks it is intact and writable. A transaction rolled back writes a table, so
/// that a read-only file or folder is found now rather than at the first save.
fn open_checked(path: &Path, flags: OpenFlags) -> rusqlite::Result<Connection> {
    let conn = Connection::open_with_flags(path, flags)?;
    check(&conn)?;
    if !flags.contains(OpenFlags::SQLITE_OPEN_READ_WRITE) {
        return Ok(conn);
    }
    let written = conn.execute_batch("BEGIN IMMEDIATE; CREATE TABLE launch_check (x); ROLLBACK;");
    if written.is_err() {
        let _ = conn.execute_batch("ROLLBACK");
    }
    written.map(|_| conn)
}

/// Fails on a damaged database.
fn check(conn: &Connection) -> rusqlite::Result<()> {
    let check: String = conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
    if check != "ok" {
        return Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
            Some(check),
        ));
    }
    Ok(())
}

/// Copies the database to `previous`, through a temporary file renamed once complete: a copy cut short
/// never replaces the previous one.
fn copy(conn: &Connection, previous: &Path) -> rusqlite::Result<()> {
    let tmp = with_suffix(previous, ".tmp");
    let _ = std::fs::remove_file(&tmp);
    conn.execute("VACUUM INTO ?1", [tmp.to_string_lossy()])?;
    std::fs::rename(&tmp, previous).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_IOERR),
            Some(e.to_string()),
        )
    })
}

/// Sets the damaged database aside with its companion files, and puts the copy of the previous launch in
/// its place. The copy is written first: if it fails, nothing has moved.
fn restore(path: &Path, previous: &Path) -> io::Result<()> {
    let tmp = with_suffix(path, ".tmp");
    std::fs::copy(previous, &tmp)?;
    let damaged = damaged_path(path);
    for suffix in std::iter::once("").chain(COMPANIONS) {
        let file = with_suffix(path, suffix);
        if file.exists() {
            std::fs::rename(&file, with_suffix(&damaged, suffix))?;
        }
    }
    std::fs::rename(&tmp, path)
}

impl Store {
    /// Copies the database for the next launch while the app runs, if it is still intact: a damaged one
    /// never replaces the copy. An in-memory database has nothing to copy.
    pub fn save_copy(&self) -> rusqlite::Result<()> {
        let Some(path) = self.conn.path().filter(|p| !p.is_empty()) else {
            return Ok(());
        };
        check(&self.conn)?;
        copy(&self.conn, &previous_path(Path::new(path)))
    }

    /// Opens the database at launch. Intact, it is copied for the next launch, then migrated. Damaged, it is
    /// replaced by the copy of the previous launch if that one is intact; otherwise it is left as it is.
    pub fn launch(path: &Path, system_locale: Option<&str>) -> Result<Launched, LaunchError> {
        let previous = previous_path(path);
        let (conn, restored) = match open_checked(path, OpenFlags::default()) {
            Ok(conn) => (conn, false),
            Err(e) if OpenFailure::of_sqlite(&e) == OpenFailure::Damaged => {
                let usable = previous.exists()
                    && open_checked(&previous, OpenFlags::SQLITE_OPEN_READ_ONLY).is_ok();
                if !usable {
                    return Err(e.into());
                }
                restore(path, &previous)?;
                (open_checked(path, OpenFlags::default())?, true)
            }
            Err(e) => return Err(e.into()),
        };
        let copy_error = if restored {
            None
        } else {
            copy(&conn, &previous).err()
        };
        Ok(Launched {
            store: Self::init(conn, system_locale)?,
            restored,
            copy_error,
        })
    }
}

use super::*;
use std::path::{Path, PathBuf};

/// Folder of its own, for the database and the files beside it.
fn temp_folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("squatsouls-launch-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Launches and saves the global recovery, so that the copy of the next launch can be recognized.
fn launch_with_recovery(path: &Path, recovery: u32) {
    let mut store = Store::launch(path, LOCALE).unwrap().store;
    let mut cfg = store.config().unwrap();
    cfg.recovery = recovery;
    store.save_config(&cfg).unwrap();
}

fn failure(path: &Path) -> OpenFailure {
    Store::launch(path, LOCALE).err().expect("refused").failure
}

#[test]
fn intact_database_is_copied_for_the_next_launch() {
    let dir = temp_folder("copied");
    let path = dir.join("squatsouls.db");
    launch_with_recovery(&path, 90);
    let launched = Store::launch(&path, LOCALE).unwrap();
    assert!(!launched.restored && launched.copy_error.is_none());
    drop(launched);
    let previous = Store::open(&previous_path(&path), LOCALE)
        .unwrap()
        .config()
        .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(previous.recovery, 90);
}

#[test]
fn damaged_database_is_replaced_by_the_copy_of_the_previous_launch_and_set_aside() {
    let dir = temp_folder("restored");
    let path = dir.join("squatsouls.db");
    launch_with_recovery(&path, 90);
    drop(Store::launch(&path, LOCALE).unwrap());
    std::fs::write(&path, "not a database").unwrap();
    let launched = Store::launch(&path, LOCALE).unwrap();
    let recovery = launched.store.config().unwrap().recovery;
    let restored = launched.restored;
    drop(launched);
    let damaged: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|f| f.to_string_lossy().contains(".damaged-"))
        .map(|f| std::fs::read_to_string(f).unwrap())
        .collect();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(restored);
    assert_eq!(recovery, 90);
    assert_eq!(damaged, ["not a database"]);
}

#[test]
fn damaged_database_without_an_intact_copy_is_left_as_it_is() {
    let dir = temp_folder("damaged");
    let path = dir.join("squatsouls.db");
    std::fs::write(&path, "not a database").unwrap();
    let without_copy = failure(&path);
    std::fs::write(previous_path(&path), "not a database either").unwrap();
    let with_damaged_copy = failure(&path);
    let left = std::fs::read_to_string(&path).unwrap();
    let files = std::fs::read_dir(&dir).unwrap().count();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(without_copy, OpenFailure::Damaged);
    assert_eq!(with_damaged_copy, OpenFailure::Damaged);
    assert_eq!(left, "not a database");
    assert_eq!(files, 2, "nothing set aside");
}

#[test]
fn newer_database_is_not_prepared() {
    let dir = temp_folder("newer");
    let path = dir.join("squatsouls.db");
    drop(Store::launch(&path, LOCALE).unwrap());
    connect(&path)
        .execute(
            "INSERT INTO refinery_schema_history (version, name, applied_on, checksum)
             VALUES (999, 'future', '2100-01-01T00:00:00Z', '0')",
            [],
        )
        .unwrap();
    let refused = failure(&path);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(refused, OpenFailure::NotPrepared);
}

#[test]
fn read_only_database_is_denied() {
    let dir = temp_folder("read-only");
    let path = dir.join("squatsouls.db");
    drop(Store::launch(&path, LOCALE).unwrap());
    let writable = std::fs::metadata(&path).unwrap().permissions();
    let mut read_only = writable.clone();
    read_only.set_readonly(true);
    std::fs::set_permissions(&path, read_only).unwrap();
    let refused = failure(&path);
    std::fs::set_permissions(&path, writable).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(refused, OpenFailure::Denied);
}

#[test]
fn copy_is_renewed_while_the_app_runs() {
    let dir = temp_folder("renewed");
    let path = dir.join("squatsouls.db");
    let mut store = Store::launch(&path, LOCALE).unwrap().store;
    let mut cfg = store.config().unwrap();
    cfg.recovery = 75;
    store.save_config(&cfg).unwrap();
    store.save_copy().unwrap();
    drop(store);
    let previous = Store::open(&previous_path(&path), LOCALE)
        .unwrap()
        .config()
        .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(previous.recovery, 75);
}

#[test]
fn in_memory_database_has_nothing_to_copy() {
    assert!(store().save_copy().is_ok());
}

#[test]
fn only_the_three_most_recent_damaged_databases_are_kept() {
    let dir = temp_folder("pruned");
    let path = dir.join("squatsouls.db");
    drop(Store::launch(&path, LOCALE).unwrap());
    let older = ["20200101-000000", "20200102-000000", "20200103-000000"];
    for date in older {
        std::fs::write(dir.join(format!("squatsouls.damaged-{date}.db")), "old").unwrap();
    }
    std::fs::write(
        dir.join("squatsouls.damaged-20200101-000000.db-journal"),
        "old",
    )
    .unwrap();
    std::fs::write(&path, "not a database").unwrap();
    drop(Store::launch(&path, LOCALE).unwrap());
    let mut damaged: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|f| f.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|f| f.contains(".damaged-"))
        .collect();
    damaged.sort();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(damaged.len(), 3, "{damaged:?}");
    assert_eq!(damaged[0], "squatsouls.damaged-20200102-000000.db");
    assert_eq!(damaged[1], "squatsouls.damaged-20200103-000000.db");
}

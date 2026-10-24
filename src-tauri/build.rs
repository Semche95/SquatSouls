use std::{env, fs, path::Path};

/// Lists the game files of the default configuration, in the order of their names, so that adding a game
/// only takes its file.
fn seed_games() {
    println!("cargo:rerun-if-changed=src/app/seed/games");
    let folder = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/app/seed/games");
    let mut files: Vec<_> = fs::read_dir(&folder)
        .expect("seed games folder")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    let includes: String = files
        .iter()
        .map(|path| format!("include_str!({path:?}),\n"))
        .collect();
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("seed_games.rs");
    fs::write(out, format!("&[\n{includes}]")).unwrap();
}

fn main() {
    // Recompiles `embed_migrations!` as soon as a migration is added to the folder.
    println!("cargo:rerun-if-changed=migrations");
    seed_games();
    tauri_build::build()
}

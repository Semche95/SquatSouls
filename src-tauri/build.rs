fn main() {
    // Recompiles `embed_migrations!` as soon as a migration is added to the folder.
    println!("cargo:rerun-if-changed=migrations");
    tauri_build::build()
}

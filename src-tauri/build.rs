fn main() {
    // Recompile `embed_migrations!` dès qu'une migration s'ajoute au dossier.
    println!("cargo:rerun-if-changed=migrations");
    tauri_build::build()
}

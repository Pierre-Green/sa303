use std::path::Path;
use std::{env, fs};

fn main() {
    embed_cluster_seeds();
    tauri_build::build()
}

/// Recense les grappes d'exemple de `assets/clusters/` et génère la table que
/// `seed.rs` inclut. Passer par le build plutôt que par une liste écrite à la
/// main : déposer un `.json` dans le dossier suffit alors à l'embarquer, il n'y
/// a pas de second endroit à penser à mettre à jour.
fn embed_cluster_seeds() {
    let dir = Path::new("assets/clusters");
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut files: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("lecture de {}: {e}", dir.display()))
        .map(|entry| entry.expect("entrée de dossier illisible").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let name = path
                .file_name()
                .expect("un fichier a forcément un nom")
                .to_string_lossy()
                .into_owned();
            println!("cargo:rerun-if-changed={}", path.display());
            name
        })
        .collect();
    // L'ordre de `read_dir` dépend du système de fichiers : on le fixe pour que
    // deux compilations produisent le même binaire, et les grappes le même
    // ordre à l'écran.
    files.sort();

    // Chemins absolus : le fichier généré est inclus depuis `OUT_DIR`, donc un
    // chemin relatif s'y résoudrait, pas depuis la racine du crate.
    let root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR fourni par cargo");
    let entries: String = files
        .iter()
        .map(|name| format!("    include_str!(\"{root}/assets/clusters/{name}\"),\n"))
        .collect();
    let generated = format!(
        "// Généré par build.rs — ne pas éditer. Source : src-tauri/assets/clusters/\nconst CLUSTER_SEED_JSON: [&str; {}] = [\n{entries}];\n",
        files.len()
    );

    let out =
        Path::new(&env::var("OUT_DIR").expect("OUT_DIR fourni par cargo")).join("cluster_seeds.rs");
    fs::write(&out, generated).unwrap_or_else(|e| panic!("écriture de {}: {e}", out.display()));
}

use std::path::Path;
use std::{env, fs};

/// Catalogues embarqués : un dossier d'assets par famille, et le nom de la
/// constante Rust générée pour lui. Ce sont les composants « de base » : le
/// logiciel les livre, une mise à jour les remplace, l'utilisateur ne les
/// modifie pas.
const ASSET_SETS: [(&str, &str); 4] = [
    ("speakers", "SPEAKER_ASSETS"),
    ("bumpers", "BUMPER_ASSETS"),
    ("bumper-bars", "BUMPER_BAR_ASSETS"),
    ("clusters", "CLUSTER_ASSETS"),
];

fn main() {
    embed_assets();
    tauri_build::build()
}

/// Recense les assets de chaque famille et génère les tables que `seed.rs`
/// inclut. Passer par le build plutôt que par des listes écrites à la main :
/// déposer un `.json` dans un dossier suffit alors à l'embarquer, il n'y a pas
/// de second endroit à penser à mettre à jour.
fn embed_assets() {
    // Chemins absolus : le fichier généré est inclus depuis `OUT_DIR`, donc un
    // chemin relatif s'y résoudrait, pas depuis la racine du crate.
    //
    // Séparateurs forcés en `/` : ces chemins finissent dans un littéral Rust,
    // où les antislashes de Windows seraient lus comme des échappements
    // (`D:\a\...` donnerait une erreur sur `\a`). `include_str!` accepte les
    // barres obliques sur toutes les plateformes, donc c'est aussi le seul
    // moyen d'obtenir le même fichier généré partout.
    let root = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR fourni par cargo")
        .replace('\\', "/");

    let mut generated = String::from(
        "// Généré par build.rs — ne pas éditer. Source : src-tauri/assets/\n\
         // Chaque entrée est (nom de fichier, contenu JSON).\n",
    );
    for (dir_name, const_name) in ASSET_SETS {
        let dir = Path::new("assets").join(dir_name);
        println!("cargo:rerun-if-changed={}", dir.display());

        let mut files: Vec<String> = fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("lecture de {}: {e}", dir.display()))
            .map(|entry| entry.expect("entrée de dossier illisible").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .map(|path| {
                println!("cargo:rerun-if-changed={}", path.display());
                path.file_name()
                    .expect("un fichier a forcément un nom")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        // L'ordre de `read_dir` dépend du système de fichiers : on le fixe pour
        // que deux compilations produisent le même binaire, et les catalogues
        // le même ordre à l'écran.
        files.sort();

        let entries: String = files
            .iter()
            .map(|name| {
                // La racine n'a plus d'antislash, mais un nom de fichier peut en
                // contenir un (légal sous Unix), tout comme un guillemet : on
                // échappe ce qui part dans un littéral plutôt que de parier.
                let path = escape(&format!("{root}/assets/{dir_name}/{name}"));
                format!(
                    "    (\"{}\", include_str!(\"{path}\")),\n",
                    escape(name.trim_end_matches(".json"))
                )
            })
            .collect();
        generated.push_str(&format!(
            "pub const {const_name}: [(&str, &str); {}] = [\n{entries}];\n",
            files.len()
        ));
    }

    let out = Path::new(&env::var("OUT_DIR").expect("OUT_DIR fourni par cargo")).join("assets.rs");
    fs::write(&out, generated).unwrap_or_else(|e| panic!("écriture de {}: {e}", out.display()));
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

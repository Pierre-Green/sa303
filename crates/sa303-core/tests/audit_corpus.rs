//! Harnais de validation externe (docs/audit/validation-2026-09) : recalcule
//! avec le solveur réel un corpus de grappes décrit en JSON et écrit l'export
//! d'audit correspondant, pour comparaison avec le solveur de référence Python.
//!
//! Ignoré par défaut : il lit et écrit des fichiers hors du crate.
//!
//! ```sh
//! SA303_CORPUS_IN=docs/audit/validation-2026-09/corpus-definitions.json \
//! SA303_CORPUS_OUT=docs/audit/validation-2026-09/corpus-export.json \
//! cargo test -p sa303-core --test audit_corpus -- --ignored
//! ```

use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::Cluster;
use sa303_core::export::build_audit_export;
use sa303_core::settings::Settings;
use sa303_core::speaker::SpeakerModel;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Definitions {
    speakers: Vec<SpeakerModel>,
    bumpers: Vec<BumperModel>,
    #[serde(default)]
    bumper_bars: Vec<BumperBarModel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorpusIn {
    settings: Settings,
    definitions: Definitions,
    clusters: Vec<Cluster>,
}

#[test]
#[ignore]
fn export_corpus_for_external_validation() {
    let input = std::env::var("SA303_CORPUS_IN").expect("SA303_CORPUS_IN");
    let output = std::env::var("SA303_CORPUS_OUT").expect("SA303_CORPUS_OUT");
    let text = std::fs::read_to_string(&input).expect("lecture du corpus");
    let corpus: CorpusIn = serde_json::from_str(&text).expect("corpus JSON");
    let export = build_audit_export(
        "corpus".into(),
        &corpus.clusters,
        &corpus.definitions.speakers,
        &corpus.definitions.bumpers,
        &corpus.definitions.bumper_bars,
        &corpus.settings,
    );
    let json = serde_json::to_string_pretty(&export).expect("sérialisation");
    std::fs::write(&output, json).expect("écriture de l'export");
    eprintln!(
        "{} grappes calculées, {} impossibles -> {output}",
        export.clusters.len(),
        export.impossible.len()
    );
}

//! Noyau de calcul SA303 : chaque domaine métier (enceinte, bumper, barre de
//! déport, réglages, grappe/stack, tirette) est un module à part entière,
//! avec son propre modèle persisté et sa propre géométrie (brief §1). Sans
//! dépendance à Tauri — tout le calcul vit ici, le front n'affiche que. Ce
//! fichier n'est qu'un point d'entrée : chaque module a sa propre
//! responsabilité (voir le commentaire en tête de chacun).

pub mod bumper;
pub mod checks;
pub mod cluster;
pub mod settings;
pub mod speaker;
pub mod tie;
pub mod vector;
pub mod wst;

pub use cluster::{
    compute_aggregate, compute_cluster, AggregateReport, BumperView, ClusterResult,
    CompartmentReport, ImpossibleClusterReport, ImpossibleConfiguration, LoadCaseReport,
};

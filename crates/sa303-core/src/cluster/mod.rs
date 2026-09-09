//! Domaine "grappe/stack" (brief §4, §5, §9) : modèle persisté (`model`),
//! cinématique d'assemblage (`kinematics`), statique d'une jonction
//! (`joint`), assemblage complet et décision du solveur pour une grappe
//! entière (`solver`), sélection des pires cas parmi les clusters
//! (`worst_cases_selector`), et agrégat de ces pires cas sur toutes les
//! configurations (`aggregate`). Sans dépendance à Tauri (brief §1) — tout le
//! calcul vit ici, le front n'affiche que.

mod aggregate;
mod joint;
mod kinematics;
mod model;
mod solver;
mod worst_cases_selector;

pub use aggregate::{
    compute_aggregate, AggregateReport, CompartmentReport, ImpossibleClusterReport, LoadCaseReport,
};
pub use joint::{compute_joint, JointInput, JointResult};
pub use kinematics::{
    build_cluster, phi_initial_free_hang, phi_initial_stack, solve_pickup_x_for_imposed_tilt,
    weighted_cg, ChainSpeaker, SpeakerInstance,
};
pub use model::{Cluster, Compartment, JointSetting};
pub use solver::{compute_cluster, BumperView, ClusterResult, ImpossibleConfiguration};
pub use worst_cases_selector::{select_block_a, select_block_b, LoadCase};

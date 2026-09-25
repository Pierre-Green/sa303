//! Domaine « pull-back » : second point de levage au pied de la grappe, qui
//! tire verticalement vers le haut (180° ± tolérance, convention §2). Un câble
//! ne peut que tirer, jamais pousser : ce module délimite la plage de
//! directions utilisables et suggère la verticale, sans décider à la place de
//! l'utilisateur.

mod tension;

pub use tension::{
    clamp_to_pull_back_range, is_within_pull_back_window, optimal_pull_back_direction, pull_back_usable_range_deg,
    pull_back_default_angle_deg, pull_back_tension, pull_back_valid_angle_range_deg, PullBackForce,
    DEFAULT_PULL_BACK_TOLERANCE_DEG, PULL_BACK_STATIC_MARGIN_DEG, PULL_BACK_VERTICAL_DEG,
};

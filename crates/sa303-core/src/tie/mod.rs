//! Domaine "tirette basse" : tension et direction. Un câble ne peut que
//! tirer, jamais pousser, ce qui délimite une plage de directions
//! physiquement valables pour un point d'accroche et un point de levage
//! donnés — l'utilisateur choisit son propre point d'ancrage réel dedans ;
//! ce module ne fait que délimiter et suggérer, jamais décider à sa place.

mod tension;

pub use tension::{
    optimal_tie_direction, tie_default_angle_deg, tie_tension, tie_valid_angle_range_deg, TieForce,
};

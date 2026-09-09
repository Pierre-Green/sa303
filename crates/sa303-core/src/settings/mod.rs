//! Domaine "réglages" (brief §6, §9) : coefficients de sécurité/dynamique,
//! spécification du sandwich flancs/barre pour `crate::checks`, et mapping
//! d'axes outil pour l'export.

mod model;

pub use model::{AxisMapping, PinSpec, PlateSpec, Settings};

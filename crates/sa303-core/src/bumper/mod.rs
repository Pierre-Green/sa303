//! Domaine "bumper" (capotage) : modèle persisté — compatibilité déclarée par
//! enceinte et par compartiment (`model`) — et géométrie dérivée — silhouette,
//! hauteur de manille (`geometry`). La barre de déport associée
//! (SA303-BUMPER-BAR) vit ici aussi (`bumper_bar`) : c'est le bumper actif
//! qui déclare quelle barre s'applique, jamais l'inverse.

mod bumper_bar;
mod geometry;
mod model;

pub use bumper_bar::{BumperBarCompatibility, BumperBarModel};
pub use geometry::{bumper_outline_top, bumper_pickup_height};
pub use model::{BumperCompatibility, BumperModel};

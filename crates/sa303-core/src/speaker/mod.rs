//! Domaine "enceinte" (brief §3) : description physique persistée
//! (`model`), géométrie dérivée — trous fixes, couronne, silhouette —
//! (`geometry`), et rapport de cohérence géométrique affiché en lecture
//! seule sur la page "Équipement et enceinte" (`report`). Jamais de statique
//! (forces, tensions) ici : ça vit dans `crate::cluster` et `crate::tie`.

mod geometry;
mod model;
mod report;

pub use geometry::{is_odd_splay, speaker_outline, CrownRow, SpeakerGeometry};
pub use model::{
    BelowCompatibility, Crown, Hinge, SpeakerAcousticsModel, SpeakerMechanicalModel, SpeakerModel,
    SplayRange, WaveguideFront,
};
pub use report::{
    geometry_report, CrownHoleReport, GeometryInconsistency, SpeakerGeometryReport,
    LEVER_TOLERANCE_MM,
};

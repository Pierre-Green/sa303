//! Domaine "enceinte" (brief §3) : description physique persistée
//! (`model`), géométrie dérivée — trous fixes, couronne, silhouette —
//! (`geometry`), et rapport de cohérence géométrique affiché en lecture
//! seule sur la page "Équipement et enceinte" (`report`). Jamais de statique
//! (forces, tensions) ici : ça vit dans `crate::cluster` et `crate::pull_back`.

mod geometry;
mod model;
mod report;

pub use geometry::{
    check_rear_bar, speaker_outline, BarInconsistency, BarWarning, CrownRow, JointOffset,
    SpeakerGeometry, BAR_FIT_TOLERANCE_MM,
};
pub use model::{
    BarHole, BarHoles, BelowCompatibility, Crown, GuideMeasurement, Hinge, PolarHole, RearBar,
    SpeakerAcousticsModel, SpeakerMechanicalModel, SpeakerModel, SplayRange, WaveguideFront,
};
pub use report::{
    geometry_report, CrownHoleReport, GeometryInconsistency, SpeakerGeometryReport,
    LEVER_TOLERANCE_MM,
};

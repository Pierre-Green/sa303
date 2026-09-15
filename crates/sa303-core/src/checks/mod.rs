//! Vérifications mécaniques (brief §6) : une seule catégorie pour l'instant
//! (le sandwich flancs/barre), regroupée dans son propre fichier plutôt que
//! d'accumuler d'autres familles de contrôle ici au fil du temps.

mod bar;
mod sandwich;

pub use bar::{
    check_bar, BarCheck, BarGeometryWarning, BarLoads, BarProfilePoint, BarSectionCheck,
    BarTearOutCheck, MOMENT_CONTINUITY_TOLERANCE_NMM, PROFILE_SAMPLE_MM, SWEEP_STEP_MM,
};
pub use sandwich::{utilization, utilization_breakdown, SandwichSpec, UtilizationBreakdown};

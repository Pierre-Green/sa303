//! Vérifications mécaniques (brief §6) : une seule catégorie pour l'instant
//! (le sandwich flancs/barre), regroupée dans son propre fichier plutôt que
//! d'accumuler d'autres familles de contrôle ici au fil du temps.

mod sandwich;

pub use sandwich::{utilization, utilization_breakdown, SandwichSpec, UtilizationBreakdown};

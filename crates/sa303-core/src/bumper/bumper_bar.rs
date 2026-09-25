//! Barre de déport (SA303-BUMPER-BAR ou équivalent) : composant d'équipement
//! à part entière, jamais lié à une grappe directement — c'est le bumper
//! actif qui détermine quelle barre s'applique (`compatible_bumpers`), voir
//! `crate::cluster::solver`. Ses trous d'accroche sont cotés un par un
//! (`BumperBarGeometry`), arc compris ; sans eux, seule sa portée maximale
//! entre dans le solveur (ancien modèle continu).

use serde::{Deserialize, Serialize};

/// Une barre de déport est utilisable avec un bumper donné.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperBarCompatibility {
    pub bumper_model_id: String,
}

/// SA303-BUMPER-BAR (ou équivalent) : composant d'équipement à part entière.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperBarModel {
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    /// Portée maximale de déport depuis le centre du bumper, mm. Au-delà,
    /// même cette barre ne suffit plus : il faut un pull-back en renfort.
    pub max_deport_mm: f64,
    /// Trous d'accroche et pattes de liaison, repère barre. `None` pour un
    /// fichier écrit avant qu'ils ne soient déclarés (brief §8).
    #[serde(default)]
    pub geometry: Option<super::BumperBarGeometry>,
    pub compatible_bumpers: Vec<BumperBarCompatibility>,
}

//! Barre de déport (SA303-BUMPER-BAR ou équivalent) : composant d'équipement
//! à part entière, jamais lié à une grappe directement — c'est le bumper
//! actif qui détermine quelle barre s'applique (`compatible_bumpers`), voir
//! `crate::cluster::solver`. Ses trous d'accroche sont cotés un par un
//! (`BumperBarGeometry`), arc compris.

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
    /// Trous d'accroche et pattes de liaison, repère barre.
    pub geometry: super::BumperBarGeometry,
    pub compatible_bumpers: Vec<BumperBarCompatibility>,
}

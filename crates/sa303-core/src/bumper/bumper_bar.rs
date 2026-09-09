//! Barre de déport (SA303-BUMPER-BAR ou équivalent) : composant d'équipement
//! à part entière, jamais lié à une grappe directement — c'est le bumper
//! actif qui détermine quelle barre s'applique (`compatible_bumpers`), voir
//! `crate::cluster::solver`. Jamais modélisée géométriquement (ses perçages
//! ne sont pas à hauteur constante) : seule sa portée maximale entre dans le
//! solveur.

use serde::{Deserialize, Serialize};

/// Une barre de déport est utilisable avec un bumper donné.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperBarCompatibility {
    pub bumper_model_id: String,
}

/// SA303-BUMPER-BAR (ou équivalent) : composant d'équipement à part entière,
/// jamais modélisé géométriquement (ses perçages ne sont pas à hauteur
/// constante) — seule sa portée maximale de déport entre dans le solveur.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperBarModel {
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    /// Portée maximale de déport depuis le centre du bumper, mm. Au-delà,
    /// même cette barre ne suffit plus : il faut une tirette en renfort.
    pub max_deport_mm: f64,
    pub compatible_bumpers: Vec<BumperBarCompatibility>,
}

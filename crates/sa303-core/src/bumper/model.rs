//! Bumper de capotage : posé sur l'enceinte du haut en vol, ou sous
//! l'enceinte du bas en stack. Sa compatibilité est déclarée par enceinte et
//! par compartiment (un bumper peut être prévu pour le vol sans être posable
//! au sol, ou l'inverse).

use serde::{Deserialize, Serialize};

/// Un bumper est utilisable sur une enceinte donnée, éventuellement seulement
/// dans un des deux compartiments (ex. un bumper prévu pour le vol mais pas
/// posable au sol).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperCompatibility {
    /// Alias : anciennement `boxModelId` (brief §8, voir `Cluster::speaker_model_id`).
    #[serde(alias = "boxModelId")]
    pub speaker_model_id: String,
    pub flown: bool,
    pub stacked: bool,
}

/// Bumper de capotage, posé sur l'enceinte du haut en vol ou sous l'enceinte
/// du bas en stack. En vol, porte la barre de déport qui permet de décaler le
/// point d'accroche hors de l'aplomb du bumper.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperModel {
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    /// Profondeur avant-arrière du bumper, mm.
    pub depth: f64,
    /// Épaisseur du bumper, mm.
    pub height: f64,
    /// Hauteur à laquelle la manille se ferme au-dessus du dessus du bumper,
    /// mm. Volontairement indépendant de la SA303-BUMPER-BAR (barre de déport) :
    /// ses perçages ne sont pas à hauteur constante, donc seul le bumper
    /// entre dans le modèle géométrique — la barre n'est qu'un rendu dérivé.
    /// Alias : anciennement `barHeightAboveBumper`, avant de dissocier la
    /// hauteur de manille de la SA303-BUMPER-BAR — un fichier existant avec l'ancien
    /// nom ne doit jamais faire planter la lecture (brief §8).
    #[serde(alias = "barHeightAboveBumper")]
    pub shackle_height_above_bumper: f64,
    /// Décalage maximal du point d'accroche par rapport au centre du bumper
    /// avant qu'une SA303-BUMPER-BAR ne soit nécessaire, mm. Par défaut `depth / 2`,
    /// mais réglable indépendamment (ex. si les points de fixation directs
    /// n'utilisent pas toute la profondeur du bumper).
    /// `#[serde(default)]` : un fichier déjà sur disque sans ce champ ne doit
    /// jamais faire planter la lecture (brief §8, leçon du champ précédent).
    #[serde(default = "default_max_direct_deport_mm")]
    pub max_direct_deport_mm: f64,
    /// Alias : anciennement `compatibleBoxes` (brief §8).
    #[serde(alias = "compatibleBoxes")]
    pub compatible_speakers: Vec<BumperCompatibility>,
}

fn default_max_direct_deport_mm() -> f64 {
    351.0
}

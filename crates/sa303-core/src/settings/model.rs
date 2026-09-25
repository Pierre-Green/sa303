//! Réglages globaux (brief §6, §9) : coefficients de sécurité/dynamique,
//! spécification du sandwich flancs/barre, mapping d'axes outil pour l'export.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinSpec {
    pub diameter: f64,
    pub ultimate: f64,
    pub net_section: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlateSpec {
    pub flank_thickness: f64,
    pub bar_thickness: f64,
    pub ultimate: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisMapping {
    pub tool_x: String,
    pub tool_y: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub safety_factor: f64,
    pub dynamic_factor: f64,
    pub gravity: f64,
    pub share_per_flank: f64,
    pub pin: PinSpec,
    pub plate: PlateSpec,
    pub axis_mapping: AxisMapping,
    /// Tolérance du pull-back autour de la verticale, en degrés : la direction
    /// doit rester dans 180° ± cette valeur (convention §2, 180° = vers le
    /// haut). 10° par défaut, comme Meyer Sound ; absent des anciens fichiers
    /// de réglages, d'où le `serde(default)`.
    #[serde(default = "default_pull_back_tolerance_deg")]
    pub pull_back_tolerance_deg: f64,
}

fn default_pull_back_tolerance_deg() -> f64 {
    crate::pull_back::DEFAULT_PULL_BACK_TOLERANCE_DEG
}

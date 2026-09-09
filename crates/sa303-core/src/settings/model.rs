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
}

//! Une grappe (vol) ou un stack (sol) : la chaîne de splays entre enceintes,
//! plus tout ce qui peut être imposé ou dérivé pour ce compartiment
//! (assiette, bumper, direction de pull-back). Référence sa `SpeakerModel` et
//! son `BumperModel` par id plutôt que par valeur : c'est
//! `super::solver::compute_cluster` qui résout ces références au moment du
//! calcul. Un bumper est obligatoire, en vol comme en stack : le point
//! d'accroche (vol) et le support visuel (stack) en dérivent toujours, il
//! n'existe pas de repli manuel. Un `bumper_model_id` absent ou incompatible
//! avec l'enceinte/le compartiment est une configuration impossible, jamais
//! une valeur par défaut silencieuse (brief §11.6).

use serde::{Deserialize, Serialize};

/// Vol (suspendu) ou stack (posé au sol) — détermine quel côté du joint est
/// le corps libre chargé (brief §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Compartment {
    Flown,
    Stacked,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointSetting {
    pub splay: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    /// Une enceinte par position, du haut vers le bas — une grappe est
    /// hétérogène : chaque position porte son propre modèle, tant que chaque
    /// jonction est déclarée compatible
    /// (`SpeakerModel::compatible_below`). Compte exactement une entrée de
    /// plus que `joints`. La migration depuis l'ancien champ unique
    /// `speakerModelId` est faite à la lecture, côté persistance.
    pub speaker_model_ids: Vec<String>,
    pub compartment: Compartment,
    /// Splays du haut vers le bas : exactement `speaker_model_ids.len() - 1`.
    pub joints: Vec<JointSetting>,
    /// Vol : assiette imposée. Stack : angle de l'enceinte du bas.
    pub imposed_tilt: Option<f64>,
    /// Direction du pull-back automatique (degrés, convention §2 : **180° =
    /// vers le haut**), quand le bumper et sa barre ne suffisent plus. Le
    /// pull-back est un second point de levage qui tire verticalement : la
    /// direction doit rester dans 180° ± `Settings::pull_back_tolerance_deg`.
    /// `None` tant que l'utilisateur n'a pas choisi : le solveur suggère alors
    /// la verticale, ou la borne la plus proche dans la plage utilisable.
    #[serde(default)]
    pub pull_back_angle: Option<f64>,
    /// Bumper utilisé pour dériver le point d'accroche en vol ou comme support en stack.
    pub bumper_model_id: String,
    /// Altitude du **dessous** du bumper au-dessus du sol, mm. En stack, 0 =
    /// posé au sol, et une valeur positive décrit un praticable ; en vol, c'est
    /// la hauteur d'accroche. N'entre dans aucun calcul d'effort : une grappe
    /// pèse le même poids à 2 m qu'à 12 m. Elle sert à situer l'ensemble dans
    /// l'espace, et c'est de là que `ClusterResult::elevation` dérive
    /// l'altitude de chaque point.
    ///
    /// `default` : les grappes déjà enregistrées se relisent sans modification
    /// et repartent du sol (brief §8).
    #[serde(default)]
    pub bumper_height: f64,
}

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

/// Où accrocher en vol : laissé au solveur, ou imposé par l'utilisateur.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RiggingSupport {
    /// Le bumper seul d'abord, puis la barre du montage le plus centré au plus
    /// déporté.
    #[default]
    Auto,
    Bumper,
    /// Barre forcée, même si le bumper seul suffirait (ex. pour écarter deux
    /// points et répartir la charge).
    Bar,
}

/// Choix d'accroche en vol. Jamais un trou précis : le solveur choisit le trou,
/// l'utilisateur ne choisit que la famille (bumper / barre / montage) et le
/// nombre de points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiggingRequest {
    #[serde(default)]
    pub support: RiggingSupport,
    /// 1 ou 2 moteurs. À 2 points, l'assiette est tenue par les longueurs de
    /// chaîne et le solveur choisit les deux trous qui équilibrent au mieux les
    /// charges.
    #[serde(default = "default_rigging_points")]
    pub points: u8,
    /// Montage de barre imposé, indice dans `RiggingView::bar_mounts`. `None` :
    /// au choix du solveur.
    #[serde(default)]
    pub bar_mount_index: Option<usize>,
}

fn default_rigging_points() -> u8 {
    1
}

impl Default for RiggingRequest {
    fn default() -> Self {
        Self {
            support: RiggingSupport::Auto,
            points: 1,
            bar_mount_index: None,
        }
    }
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
    /// Pull-back activé à la main, alors que le bumper et sa barre suffisent
    /// à tenir l'assiette : pour répartir la charge entre le moteur principal
    /// et le pull-back quand les points d'accroche sont faibles. Demande une
    /// assiette imposée. Sans effet quand le pull-back est de toute façon
    /// obligatoire, et en stack.
    #[serde(default)]
    pub pull_back_enabled: bool,
    /// Pull-back manuel seulement : tension voulue dans le pull-back, N,
    /// sur la même base que les efforts affichés (poids × k_dyn). Avec la
    /// direction (`pull_back_angle`) et l'assiette imposée, elle fixe
    /// l'accroche du moteur principal. Ramenée dans
    /// `BumperView::pull_back_tension_range_n`. `None` : la moitié de la
    /// charge au pull-back.
    #[serde(default)]
    pub manual_pull_back_tension_n: Option<f64>,
    /// Vol : famille d'accroche et nombre de points. Sans effet en stack, et
    /// sur un bumper qui ne déclare pas encore ses trous (`BumperModel::rigging`).
    #[serde(default)]
    pub rigging: RiggingRequest,
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

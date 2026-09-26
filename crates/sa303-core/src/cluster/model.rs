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
    pub support: RiggingSupport,
    /// 1 ou 2 moteurs. À 2 points, l'assiette est tenue par les longueurs de
    /// chaîne et le solveur choisit les deux trous qui équilibrent au mieux les
    /// charges.
    pub points: u8,
    /// Montage de barre imposé, indice dans `RiggingView::bar_mounts`. `None` :
    /// au choix du solveur.
    pub bar_mount_index: Option<usize>,
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
    /// Une enceinte par position, du haut vers le bas — une grappe est
    /// hétérogène : chaque position porte son propre modèle, tant que chaque
    /// jonction est déclarée compatible
    /// (`SpeakerModel::compatible_below`). Compte exactement une entrée de
    /// plus que `joints`.
    pub speaker_model_ids: Vec<String>,
    pub compartment: Compartment,
    /// Splays du haut vers le bas : exactement `speaker_model_ids.len() - 1`.
    pub joints: Vec<JointSetting>,
    /// Vol : assiette imposée. Stack : angle de l'enceinte du bas.
    pub imposed_tilt: Option<f64>,
    /// Direction du pull-back (degrés, convention §2 : **180° = vers le
    /// haut**). Le pull-back est un second point de levage qui tire
    /// verticalement : la direction reste dans 180° ±
    /// `Settings::pull_back_tolerance_deg`. `None` : le solveur suggère la
    /// verticale, ou la borne la plus proche dans la plage utilisable.
    pub pull_back_angle: Option<f64>,
    /// Pull-back activé à la main, alors que les trous d'accroche suffisent à
    /// approcher l'assiette : pour répartir la charge entre le moteur principal
    /// et le pull-back. Demande une assiette imposée. Sans effet quand le
    /// pull-back est de toute façon obligatoire, et en stack.
    pub pull_back_enabled: bool,
    /// Tension voulue dans le pull-back, N, sur la même base que les efforts
    /// affichés (poids × k_dyn). Ramenée dans
    /// `BumperView::pull_back_tension_range_n`.
    ///
    /// Réglable que le pull-back soit choisi ou obligatoire : avec la
    /// direction, elle fixe l'équilibre, et le solveur choisit le trou le plus
    /// proche de l'assiette. `None` : la tension qui tient exactement
    /// l'assiette.
    pub manual_pull_back_tension_n: Option<f64>,
    /// Vol : famille d'accroche et nombre de points. Sans effet en stack.
    pub rigging: RiggingRequest,
    /// Bumper utilisé pour dériver le point d'accroche en vol ou comme support en stack.
    pub bumper_model_id: String,
    /// Altitude du **dessous** du bumper au-dessus du sol, mm. En stack, 0 =
    /// posé au sol, et une valeur positive décrit un praticable ; en vol, c'est
    /// la hauteur d'accroche. N'entre dans aucun calcul d'effort : une grappe
    /// pèse le même poids à 2 m qu'à 12 m. Elle sert à situer l'ensemble dans
    /// l'espace, et c'est de là que `ClusterResult::elevation` dérive
    /// l'altitude de chaque point.
    pub bumper_height: f64,
}

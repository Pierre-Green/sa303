//! Description d'une enceinte riggable (brief §3), scindée en deux : la partie
//! mécanique (`SpeakerMechanicalModel` — dimensions, masse, CG, charnière,
//! couronne perçée), seule à entrer dans le calcul de rigging, et la partie
//! acoustique (`SpeakerAcousticsModel`), purement descriptive.
//!
//! Chaque modèle est une enceinte à part entière (SA303-ISOPHASE, SA303-CCA,
//! renfort de grave, line source plus petite, ...) : pas de variantes d'un
//! même châssis, car deux enceintes du même chantier peuvent différer par la
//! masse, le CG ou le perçage. Ce qui les relie, c'est
//! `compatible_below` : la liste de ce qui peut être accroché **directement
//! sous** cette enceinte, par compartiment. Une grappe peut donc mélanger
//! librement des modèles différents tant que chaque jonction est déclarée.
//!
//! `#[serde(flatten)]` sur `mechanical` : le JSON reste plat (mêmes clés
//! `depth`, `height`, `hinge`, ... au premier niveau), donc un fichier déjà
//! enregistré continue de se lire sans modification (brief §8). `acoustics` et
//! `compatible_below` sont `#[serde(default)]` pour la même raison.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hinge {
    pub x: f64,
    pub y: f64,
    pub joint_separation: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crown {
    pub radius: f64,
    pub delta: f64,
    pub anchor_angle: f64,
    pub splay0_angle: f64,
}

/// Nature du front rayonné par le guide d'onde. Ce n'est pas un détail de
/// fiche technique : c'est elle qui décide des critères WST applicables.
/// Angler deux fronts **plans** ouvre entre eux une zone sans énergie, que le
/// critère 5 du papier borne. Un guide à courbure constante rayonne déjà un
/// secteur qui se juxtapose à celui de son voisin : cette zone n'existe pas, et
/// lui appliquer le critère 5 répondrait à une autre question que celle posée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum WaveguideFront {
    /// Front plan (guide isophase, type DOSC).
    #[default]
    Isophase,
    /// Front à courbure constante, rayonnant le secteur `directivity_vertical`.
    ConstantCurvature,
}

/// Niveau au bord du secteur qui donne un raccord plat entre deux guides
/// voisins : les deux contributions s'y somment, soit +6 dB.
fn default_level_at_half_coverage_db() -> f64 {
    -6.0
}

/// Descriptif acoustique d'une enceinte. Aucun calcul de **rigging** n'en
/// dépend — seul `SpeakerMechanicalModel` entre dans la physique des charges —
/// mais les critères WST s'y appuient entièrement : le front, la bouche du
/// guide et son secteur sont des propriétés de l'enceinte, pas des réglages
/// d'outil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerAcousticsModel {
    pub fs: f64,
    pub directivity_horizontal: f64,
    /// Ouverture verticale d'une caisse seule, degrés. Pour un guide à courbure
    /// constante c'est le secteur θ qu'il rayonne, donc le splay maximal
    /// exploitable. 0 pour un front isophase : la ligne n'ouvre alors que par
    /// le splay mécanique.
    pub directivity_vertical: f64,
    /// Nature du front rayonné.
    #[serde(default)]
    pub wg_front: WaveguideFront,
    /// Hauteur de bouche du guide, mm — le `D` des critères WST, dont dérive
    /// l'ARF. Distincte de la hauteur de caisse : c'est la part réellement
    /// occupée par la source. 0 = non renseignée.
    ///
    /// `alias` : ce champ s'appelait `radiatingHeight`, les fichiers déjà
    /// enregistrés continuent donc de se lire (brief §8).
    #[serde(default, alias = "radiatingHeight")]
    pub wg_output_height: f64,
    /// Niveau du guide à la moitié de son secteur (dB, négatif), relevé en
    /// simulation ou en mesure. −6 dB donne un raccord plat entre deux caisses
    /// voisines. Sans objet pour un front isophase.
    #[serde(default = "default_level_at_half_coverage_db")]
    pub wg_level_at_half_coverage_db: f64,
}

// `Default` écrit à la main plutôt que dérivé : le niveau au demi-secteur doit
// valoir −6 dB aussi bien pour un modèle construit en Rust que pour un JSON où
// la clé manque, sinon les deux chemins ne donneraient pas la même enceinte.
impl Default for SpeakerAcousticsModel {
    fn default() -> Self {
        Self {
            fs: 0.0,
            directivity_horizontal: 0.0,
            directivity_vertical: 0.0,
            wg_front: WaveguideFront::default(),
            wg_output_height: 0.0,
            wg_level_at_half_coverage_db: default_level_at_half_coverage_db(),
        }
    }
}

/// Plage de splay recommandée pour une jonction donnée, en degrés. Hors de
/// cette plage, la jonction reste **mécaniquement valable** : elle est
/// seulement signalée comme non optimale acoustiquement (brief : ce n'est
/// jamais une erreur, c'est un conseil de calage).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplayRange {
    pub min_deg: f64,
    pub max_deg: f64,
}

impl SplayRange {
    /// Valeur unique recommandée (ex. « 10° entre ISOPHASE et CCA »).
    pub fn exactly(deg: f64) -> Self {
        Self {
            min_deg: deg,
            max_deg: deg,
        }
    }

    pub fn contains(&self, splay_deg: f64) -> bool {
        // Tolérance : les splays sont des trous percés, pas des réels libres —
        // un 10.0 saisi ne doit jamais rater un 10.0 recommandé pour cause
        // d'arrondi de représentation.
        splay_deg >= self.min_deg - 1e-9 && splay_deg <= self.max_deg + 1e-9
    }
}

/// Ce qui peut être accroché **directement sous** une enceinte donnée, et à
/// quel splay c'est acoustiquement recommandé. La relation est déclarée dans
/// un seul sens (vers le bas) : une jonction est une paire ordonnée
/// haut → bas, donc une seule déclaration par paire — impossible d'avoir deux
/// déclarations qui se contredisent. « Ce qui peut aller au-dessus » se déduit
/// en balayant les modèles qui déclarent celui-ci dans leur `compatible_below`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BelowCompatibility {
    /// Modèle de l'enceinte du dessous.
    pub speaker_model_id: String,
    /// Assemblage autorisé en grappe suspendue.
    pub flown: bool,
    /// Assemblage autorisé en stack.
    pub stacked: bool,
    /// Splay recommandé pour cette jonction. `None` : pas d'avis acoustique,
    /// la jonction n'est jamais signalée comme non optimale.
    #[serde(default)]
    pub recommended_splay: Option<SplayRange>,
}

/// Tout ce qui entre dans le calcul de rigging (brief §3) : c'est à partir de
/// ces seuls champs que `super::geometry::SpeakerGeometry` dérive tous les
/// trous et bras de levier.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerMechanicalModel {
    pub depth: f64,
    pub height: f64,
    pub total_vertical_angle: f64,
    pub mass_kg: f64,
    pub cg: [f64; 2],
    pub hinge: Hinge,
    pub crown: Crown,
    pub splay_grid: Vec<f64>,
    pub frame_hole_splay: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerModel {
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    #[serde(flatten)]
    pub mechanical: SpeakerMechanicalModel,
    #[serde(default)]
    pub acoustics: SpeakerAcousticsModel,
    /// Enceintes accrochables directement sous celle-ci (voir
    /// `BelowCompatibility`). Vide : rien ne peut être accroché dessous, donc
    /// cette enceinte ne peut être qu'en bas de chaîne.
    #[serde(default)]
    pub compatible_below: Vec<BelowCompatibility>,
}

impl SpeakerModel {
    /// Déclaration de jonction vers l'enceinte du dessous, si elle existe.
    pub fn below(&self, lower_speaker_model_id: &str) -> Option<&BelowCompatibility> {
        self.compatible_below
            .iter()
            .find(|c| c.speaker_model_id == lower_speaker_model_id)
    }
}

impl BelowCompatibility {
    /// La jonction est-elle autorisée dans ce compartiment ?
    pub fn allows(&self, flown: bool) -> bool {
        if flown {
            self.flown
        } else {
            self.stacked
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un fichier enregistré avant le renommage doit continuer de se lire, et
    /// de rendre la même enceinte (brief §8 : jamais de perte silencieuse).
    #[test]
    fn a_speaker_saved_before_the_rename_still_loads_its_mouth_height() {
        let legacy = r#"{
            "fs": 60.0,
            "directivityHorizontal": 90.0,
            "directivityVertical": 20.0,
            "radiatingHeight": 470.0
        }"#;
        let acoustics: SpeakerAcousticsModel = serde_json::from_str(legacy).unwrap();
        assert_eq!(acoustics.wg_output_height, 470.0);
        // Champs absents du fichier : le front vaut isophase, et le niveau au
        // demi-secteur le défaut qui donne un raccord plat.
        assert_eq!(acoustics.wg_front, WaveguideFront::Isophase);
        assert_eq!(acoustics.wg_level_at_half_coverage_db, -6.0);
    }

    /// Le défaut construit en Rust et le défaut lu depuis un JSON vide doivent
    /// décrire la même enceinte, sinon les deux chemins divergeraient.
    #[test]
    fn the_rust_default_and_the_json_default_agree() {
        let from_json: SpeakerAcousticsModel = serde_json::from_str(
            "{\"fs\":0.0,\"directivityHorizontal\":0.0,\"directivityVertical\":0.0}",
        )
        .unwrap();
        assert_eq!(from_json, SpeakerAcousticsModel::default());
    }
}

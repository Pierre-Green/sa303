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
    /// Distance verticale centre à centre entre deux caissons au splay 0. Elle
    /// vaut `2*y + entraxe de bielle` : c'est de là qu'est dérivée la longueur
    /// de la bielle avant.
    pub joint_separation: f64,
    /// Recul du trou de charnière depuis la face avant (`e_perp` de la table
    /// 3.9 EN 1993-1-8, perpendiculaire à l'effort). À ne pas confondre avec la
    /// distance au bord dans l'axe de l'effort : les intervertir décale le trou
    /// de 4 mm et fausse toute la couronne (brief §1).
    pub edge_perp: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crown {
    pub radius: f64,
    pub delta: f64,
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

/// Position d'un trou de la paire dans le repère du caisson qui la porte,
/// donnée en polaire depuis `ht` (le trou de charnière avant-haut).
///
/// Polaire et non cartésien parce que c'est ainsi que le perçage est coté :
/// l'ancrage et la couronne sont sur des arcs, et c'est leur rayon qui est la
/// grandeur de fabrication. Le verrou, lui, n'est **plus** sur le cercle de
/// 680 — il est au bout de l'axe de barre, à 710,845 — donc chaque trou porte
/// son propre rayon et il n'existe plus de rayon commun à supposer.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolarHole {
    pub radius: f64,
    pub angle_deg: f64,
}

/// Un trou de la barre, dans le repère de la barre : abscisse depuis le petit
/// bout (côté verrou) et déport latéral, positif vers l'**avant** du caisson.
///
/// Deux coordonnées et non une seule : `up660` est déporté de 19,4 mm de l'axe.
/// Une abscisse seule le placerait sur l'axe et perdrait le bras de levier
/// transversal — donc le moment qu'il introduit sur les splays impairs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BarHole(pub [f64; 2]);

impl BarHole {
    pub fn along(&self) -> f64 {
        self.0[0]
    }
    pub fn lateral(&self) -> f64 {
        self.0[1]
    }
    pub fn as_vec(&self) -> crate::vector::Vec2 {
        crate::vector::Vec2::new(self.0[0], self.0[1])
    }
}

/// Les quatre trous de la barre arrière, repère barre.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarHoles {
    /// Petit bout de la barre. C'est le trou le **plus loin** de la couronne,
    /// donc celui où le moment de flexion est nul.
    pub latch: BarHole,
    /// Premier pion rencontré depuis la couronne : c'est lui qui voit le
    /// moment maximal.
    pub anchor: BarHole,
    /// Trou de couronne des splays **impairs** (couronne intérieure, 660).
    /// Déporté latéralement : l'effort de couronne n'y est pas sur l'axe.
    pub up660: BarHole,
    /// Trou de couronne des splays **pairs** (couronne extérieure, 680), sur
    /// l'axe. C'est lui qui définit l'axe de la barre avec l'ancrage.
    pub up680: BarHole,
}

/// Barre arrière, la pièce qui relie la couronne du caisson du haut à la paire
/// ancrage/verrou du caisson du bas. Goupillée en un point en haut
/// (articulation, moment nul) et en **deux** points en bas (encastrement) :
/// elle travaille donc en flexion, ce qu'aucun élément à deux forces ne fait.
///
/// **Orientation.** Toutes les abscisses sont mesurées depuis le **petit bout**,
/// celui du verrou. La couronne est à l'autre extrémité. L'ordre le long de la
/// barre est donc : verrou, ancrage, épaulement, couronne.
///
/// **L'élargissement est d'un seul côté.** Le bord arrière est une droite sur
/// toute la longueur ; c'est le bord avant qui s'écarte à partir de
/// `step_position` pour passer de `narrow_width` à `wide_width`. Le trou
/// `up660` vit dans cette partie élargie, et son déport latéral est ce qui
/// justifie l'élargissement.
///
/// **Section critique : l'ancrage.** Le moment est nul à la couronne
/// (articulation) et nul au verrou (bout libre au-delà), et il culmine à
/// l'ancrage, premier pion depuis la couronne. Ce n'est plus le verrou comme
/// dans la géométrie précédente : les deux pions ont échangé leur rang le long
/// de la barre.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RearBar {
    pub thickness: f64,
    /// Longueur totale. Pilotée par `latch_offset` :
    /// `length = 443.514 + latch_offset − 85`.
    pub length: f64,
    /// Largeur courante, celle de la partie qui porte la paire — donc celle de
    /// la section critique. C'est le paramètre à balayer pour dimensionner.
    pub narrow_width: f64,
    /// Largeur après l'épaulement, côté couronne.
    pub wide_width: f64,
    /// Longueur de la partie élargie, depuis le **gros** bout.
    pub wide_length: f64,
    /// Abscisse de l'épaulement depuis le petit bout. Redondant avec
    /// `length − wide_length`, et c'est voulu : la cote de fabrication est
    /// donnée des deux côtés sur le plan, un écart entre les deux est une
    /// faute de saisie qu'un contrôle doit attraper.
    pub step_position: f64,
    /// Diamètre de perçage `d0` (goupille + jeu), pas le diamètre de goupille :
    /// c'est lui qui retire de la matière au calcul de section nette.
    pub hole_diameter: f64,
    pub holes: BarHoles,
    /// Limite d'élasticité, MPa.
    pub yield_strength: f64,
    /// Résistance à la rupture, MPa.
    pub ultimate_strength: f64,
}

impl RearBar {
    /// Largeur de la barre à l'abscisse donnée. La transition est franche dans
    /// le modèle : c'est la section juste avant l'épaulement, la plus étroite,
    /// qui est vérifiée.
    pub fn width_at(&self, x: f64) -> f64 {
        if x >= self.step_position {
            self.wide_width
        } else {
            self.narrow_width
        }
    }

    /// Trou de couronne utilisé pour ce splay, selon la parité de la couronne.
    pub fn crown_hole(&self, splay_deg: f64) -> BarHole {
        if super::geometry::is_odd_splay(splay_deg) {
            self.holes.up660
        } else {
            self.holes.up680
        }
    }

    /// Distance du centre d'un trou au bord le plus proche, perpendiculairement
    /// à l'axe. Le bord arrière est à `−narrow_width/2` sur toute la longueur ;
    /// le bord avant est à `+narrow_width/2` avant l'épaulement et s'écarte
    /// ensuite — l'élargissement se faisant d'un seul côté, le demi-axe avant
    /// vaut `wide_width − narrow_width/2`.
    pub fn edge_distance_at(&self, hole: BarHole) -> f64 {
        let back = self.narrow_width / 2.0;
        let front = if hole.along() >= self.step_position {
            self.wide_width - self.narrow_width / 2.0
        } else {
            self.narrow_width / 2.0
        };
        (front - hole.lateral()).min(hole.lateral() + back)
    }

    /// Aire nette : la largeur locale moins le perçage.
    pub fn net_area_at(&self, x: f64, drilled: bool) -> f64 {
        let w = self.width_at(x);
        self.thickness * if drilled { w - self.hole_diameter } else { w }
    }

    /// Module de flexion net à un trou **centré** : le perçage étant sur la
    /// fibre neutre, il se retranche en cube — `t(w³ − d0³)/(6w)`, et **pas**
    /// `t(w − d0)²/6` qui vaudrait pour un trou en fibre extrême et
    /// surestimerait la contrainte d'un facteur 2.
    pub fn section_modulus_at(&self, x: f64, drilled: bool) -> f64 {
        let w = self.width_at(x);
        let d = if drilled { self.hole_diameter } else { 0.0 };
        self.thickness * (w.powi(3) - d.powi(3)) / (6.0 * w)
    }

    /// Module de flexion net à un trou **excentré**. Retirer de la matière hors
    /// de la fibre neutre déplace le centroïde, donc les deux fibres extrêmes
    /// n'ont plus le même module : c'est la plus petite qui gouverne.
    ///
    /// `up660` a un moment nul dans le modèle en vigueur, mais la formule est
    /// là pour que déplacer un trou ne demande pas d'écrire le calcul dans
    /// l'urgence.
    pub fn eccentric_section_modulus(&self, hole: BarHole) -> f64 {
        let w = self.width_at(hole.along());
        let t = self.thickness;
        let d = self.hole_diameter;
        // Demi-largeurs depuis l'axe, asymétriques après l'épaulement.
        let back = self.narrow_width / 2.0;
        let front = if hole.along() >= self.step_position {
            self.wide_width - self.narrow_width / 2.0
        } else {
            self.narrow_width / 2.0
        };
        // Section pleine, repérée depuis l'axe de la partie étroite.
        let a_full = t * w;
        let c_full = (front - back) / 2.0;
        let i_full = t * w.powi(3) / 12.0 + a_full * c_full.powi(2);
        // Le trou, assimilé à une fente de largeur `d` — même convention que
        // le cas centré, pour que les deux formules coïncident à déport nul.
        let a_hole = t * d;
        let c_hole = hole.lateral();
        let i_hole = t * d.powi(3) / 12.0 + a_hole * c_hole.powi(2);

        let a_net = a_full - a_hole;
        let c_net = (a_full * c_full - a_hole * c_hole) / a_net;
        let i_net = (i_full - i_hole) - a_net * c_net.powi(2);
        // Fibres extrêmes depuis le centroïde net : la plus éloignée gouverne.
        let v = (front - c_net).abs().max((c_net + back).abs());
        i_net / v
    }
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
    /// Verrou et ancrage, en polaire depuis `ht`. Ils remplacent les
    /// `crown.anchor_angle` / `crown.latch_angle` de l'ancien modèle, qui
    /// supposaient les deux trous sur le même cercle de 680 — ce n'est plus le
    /// cas du verrou.
    pub latch: PolarHole,
    pub anchor: PolarHole,
    /// Entraxe ancrage-verrou, mm. Paramètre pilote du dessin : c'est lui qui
    /// fixe la longueur de barre et le bras du couple qui reprend le moment.
    pub latch_offset: f64,
    pub splay_grid: Vec<f64>,
    pub frame_hole_splay: f64,
    pub rear_bar: RearBar,
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

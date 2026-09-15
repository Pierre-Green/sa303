//! Géométrie dérivée d'un `SpeakerModel` (brief §3) : trous fixes (charnières,
//! pivot effectif), fonctions de couronne/ancrage paramétrées par le splay,
//! et silhouette (trapèze) de l'enceinte. Ne dépend jamais d'une jonction ou
//! d'une grappe — uniquement des dimensions de l'enceinte elle-même.

use super::model::SpeakerModel;
use crate::vector::{polar, Vec2};
use serde::Serialize;

/// Couronne extérieure sur les splays pairs, intérieure (en retrait de `delta`) sur les impairs.
pub fn is_odd_splay(splay_deg: f64) -> bool {
    (splay_deg.abs().round() as i64).rem_euclid(2) == 1
}

/// Trou de couronne utilisé : extérieur (splay pair) ou intérieur (splay impair).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CrownRow {
    Ext,
    Int,
}

impl CrownRow {
    pub fn of(splay_deg: f64) -> Self {
        if is_odd_splay(splay_deg) {
            CrownRow::Int
        } else {
            CrownRow::Ext
        }
    }
}

/// Géométrie dérivée d'un `SpeakerModel` (brief §3) : trous fixes et fonctions de couronne/ancrage.
/// Écartement des deux caissons à une jonction donnée, mesuré entre les coins
/// avant qui se touchent au splay 0 (brief §5). Décomposé parce que les deux
/// composantes ne se lisent pas de la même façon : c'est le `vertical_mm` qui
/// est l'espacement entre caissons à afficher, tandis que le `front_mm` dit
/// seulement de combien le caisson du bas recule — sous 0.05 mm sur toute la
/// plage line source, donc invisible.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JointOffset {
    /// Composante avant/arrière, positive vers l'arrière (le bas recule).
    pub front_mm: f64,
    /// Composante verticale, négative quand les coins s'écartent.
    pub vertical_mm: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct SpeakerGeometry {
    pub ha: f64,
    pub ht: Vec2,
    pub hb: Vec2,
    /// Entraxe de la bielle avant, entre `hb` du caisson du haut et `ht` de
    /// celui du bas.
    pub bielle_entraxe: f64,
    pub anchor_local: Vec2,
    /// Trou de verrou, second point de la barre arrière dans ce caisson : c'est
    /// lui qui, avec l'ancrage, rend la barre rigide par rapport au caisson du
    /// bas (brief §3).
    pub latch_local: Vec2,
    /// Coin avant-bas de la jonction (l'arête qui porte sur le caisson du
    /// dessous au splay 0), repère enceinte.
    pub front_edge: Vec2,
    crown_radius: f64,
    crown_delta: f64,
    splay0_angle: f64,
}

impl SpeakerGeometry {
    pub fn compute(speaker: &SpeakerModel) -> Self {
        let m = &speaker.mechanical;
        let ha = m.total_vertical_angle / 2.0;
        let ht = Vec2::new(m.hinge.x, m.hinge.y);
        let hb = Vec2::new(m.hinge.x, -m.hinge.y);
        // Entraxe de bielle : `joint_separation` est la distance verticale
        // centre à centre au splay 0, donc l'entraxe est ce qui reste une fois
        // retirées les deux demi-hauteurs de charnière.
        let bielle_entraxe = m.hinge.joint_separation - 2.0 * m.hinge.y;
        // Chaque trou porte son propre rayon : le verrou n'est plus sur le
        // cercle de 680 mm, il est au bout de l'axe de barre, plus loin. Aucune
        // formule ne doit donc supposer un rayon commun aux deux.
        let anchor_local = polar(ht, m.anchor.radius, m.anchor.angle_deg);
        let latch_local = polar(ht, m.latch.radius, m.latch.angle_deg);
        // Recul `edge_perp` depuis la face avant, et demi-entraxe de bielle
        // vers le bas : c'est la construction EN 1993-1-8 du trou de charnière,
        // prise à l'envers (brief §1).
        let front_edge = Vec2::new(
            m.hinge.x - m.hinge.edge_perp,
            -(m.hinge.y + bielle_entraxe / 2.0),
        );
        Self {
            ha,
            ht,
            hb,
            bielle_entraxe,
            anchor_local,
            latch_local,
            front_edge,
            crown_radius: m.crown.radius,
            crown_delta: m.crown.delta,
            splay0_angle: m.crown.splay0_angle,
        }
    }

    /// Goupille basse de la bielle avant, au splay donné, repère du caisson du
    /// **haut** : c'est le point où atterrit le `ht` du caisson du dessous, et
    /// l'origine de toute la couronne (brief §2, §6).
    ///
    /// Ce n'est **pas** un point fixe du caisson : la bielle étant goupillée en
    /// deux points, elle s'incline de `splay/2` (partage égal imposé par
    /// construction) et cette goupille décrit un arc. Tout modèle qui la fige
    /// est obsolète (brief §7).
    pub fn pv_at(&self, splay_deg: f64) -> Vec2 {
        let h = (splay_deg / 2.0).to_radians();
        Vec2::new(
            self.hb.x + self.bielle_entraxe * h.sin(),
            self.hb.y - self.bielle_entraxe * h.cos(),
        )
    }

    /// Inclinaison de la bielle avant, degrés : exactement la moitié du splay.
    pub fn bielle_rotation_deg(splay_deg: f64) -> f64 {
        splay_deg / 2.0
    }

    pub fn crown_radius_at(&self, splay_deg: f64) -> f64 {
        if is_odd_splay(splay_deg) {
            self.crown_radius - self.crown_delta
        } else {
            self.crown_radius
        }
    }

    /// Trou de couronne au splay donné, repère enceinte. Exact, pas approché :
    /// les huit trous ne sont pas sur un arc centré sur un point fixe, puisque
    /// leur centre `pv_at` bouge avec le splay (brief §4, §6).
    pub fn crown(&self, splay_deg: f64) -> Vec2 {
        polar(
            self.pv_at(splay_deg),
            self.crown_radius_at(splay_deg),
            self.ha + self.splay0_angle + splay_deg,
        )
    }

    /// Porte un trou du caisson du **bas** dans le repère du caisson du haut,
    /// au splay donné.
    ///
    /// Le caisson du bas est positionné en faisant coïncider son `ht` avec la
    /// goupille basse de bielle, puis en le tournant du splay entier. Un point
    /// quelconque de ce caisson suit donc `pv_at(k) + (P − ht).rotate(k)`.
    ///
    /// Formulation générale, valable quel que soit le rayon du trou : l'ancienne
    /// écriture polaire supposait ancrage et verrou sur le même cercle de 680,
    /// ce que le verrou ne respecte plus.
    fn carry(&self, local: Vec2, splay_deg: f64) -> Vec2 {
        self.pv_at(splay_deg) + (local - self.ht).rotate(splay_deg.to_radians())
    }

    /// Ancrage de l'enceinte inférieure vu depuis l'enceinte supérieure.
    pub fn anchor_at(&self, splay_deg: f64) -> Vec2 {
        self.carry(self.anchor_local, splay_deg)
    }

    /// Verrou de l'enceinte inférieure vu depuis l'enceinte supérieure.
    pub fn latch_at(&self, splay_deg: f64) -> Vec2 {
        self.carry(self.latch_local, splay_deg)
    }

    /// Écartement des coins avant au splay donné (brief §5). L'algorithme suit
    /// le coin avant-haut du caisson du bas — parti de `ht`, tourné de `splay`
    /// autour de la goupille basse de bielle — et le compare à l'arête sur
    /// laquelle il portait au splay 0.
    pub fn joint_offset(&self, splay_deg: f64) -> JointOffset {
        let pb = self.pv_at(splay_deg);
        // Vecteur goupille -> coin avant-haut, figé dans le caisson du bas :
        // il tourne donc du splay entier, pas de sa moitié.
        let c = Vec2::new(self.front_edge.x, -self.front_edge.y) - self.ht;
        let corner = pb + c.rotate(splay_deg.to_radians());
        JointOffset {
            front_mm: corner.x - self.front_edge.x,
            vertical_mm: corner.y - self.front_edge.y,
        }
    }

    /// Bras de levier géométrique de la barre arrière, entre la goupille basse
    /// de bielle et l'axe couronne-ancrage. Constant avec le splay (les deux
    /// trous sont radiaux depuis `pv_at`, d'écart angulaire fixe) ; ne sert
    /// plus qu'au recoupement de perçage, plus à la statique — la barre n'est
    /// plus un élément à deux forces (brief §7).
    pub fn lever(&self, splay_deg: f64) -> f64 {
        let a = self.anchor_at(splay_deg);
        let c = self.crown(splay_deg);
        let u = (c - a).normalize();
        (a - self.pv_at(splay_deg)).cross(u).abs()
    }

    /// Recoupement trigonométrique du bras de levier, pour contrôle de perçage.
    ///
    /// Couronne et ancrage sont tous deux radiaux depuis `pv_at`, mais sur des
    /// rayons différents : l'angle entre eux se lit donc sur la géométrie plutôt
    /// que sur une somme d'angles déclarés — c'est ce qui rend ce recoupement
    /// indépendant du chemin qui a servi à placer les trous.
    pub fn lever_check(&self, splay_deg: f64) -> f64 {
        let pv = self.pv_at(splay_deg);
        let a = (self.anchor_at(splay_deg) - pv).norm();
        let b = (self.crown(splay_deg) - pv).norm();
        let theta = ((self.anchor_at(splay_deg) - pv).cross(self.crown(splay_deg) - pv))
            .atan2((self.anchor_at(splay_deg) - pv).dot(self.crown(splay_deg) - pv));
        a * b * theta.sin() / (a * a + b * b - 2.0 * a * b * theta.cos()).sqrt()
    }
}

/// Écart relevé entre la barre déclarée et la géométrie de jonction qu'elle est
/// censée desservir, ou entre une distance au bord et son minimum réglementaire.
/// Signalé plutôt que tu : une barre qui ne tombe pas sur ses trous se monte à
/// la masse, elle ne se signale pas toute seule au montage.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarWarning {
    pub what: String,
    /// Valeur attendue par la géométrie de jonction (ou le minimum requis).
    pub expected_mm: f64,
    /// Valeur déclarée sur la barre.
    pub declared_mm: f64,
    pub delta_mm: f64,
}

/// Incohérence rédhibitoire : la barre déclarée ne dessert aucune des deux
/// couronnes, ou la géométrie de jonction n'est pas constante sur une couronne.
/// Ce n'est pas un avertissement — il n'existe pas de barre droite qui monte.
#[derive(Clone, Debug)]
pub struct BarInconsistency {
    pub reason: String,
}

/// Tolérance de concordance barre/jonction. Au-delà, la goupille n'entre pas :
/// le jeu de perçage (`d0` − diamètre goupille) ne vaut que quelques centièmes.
pub const BAR_FIT_TOLERANCE_MM: f64 = 0.05;

/// Au-delà de cet écart, ce n'est plus un défaut de cotation mais une barre qui
/// n'a rien à faire sur cette jonction.
const BAR_FIT_HARD_LIMIT_MM: f64 = 2.0;

/// Écart au-delà duquel les trois trous ne sont plus alignés : une barre droite
/// ne peut alors pas passer par les trois, quelle que soit sa cotation.
pub const BAR_AXIS_TOLERANCE_MM: f64 = 0.01;

/// Minimum EN 1993-1-8 pour les distances au bord, exprimé en diamètres de perçage.
const MIN_EDGE_DISTANCE_IN_D0: f64 = 1.2;

/// Contrôle de la barre arrière contre la géométrie de jonction qu'elle dessert
/// (brief §1). Deux choses différentes y sont vérifiées :
///
/// - que les entraxes couronne-ancrage et couronne-verrou sont **constants** sur
///   tous les splays d'une même couronne. S'ils ne l'étaient pas, aucune barre
///   rigide ne pourrait desservir cette couronne, quelle que soit sa cotation ;
/// - que les trous **déclarés** sur la barre tombent bien sur ces entraxes.
///
/// Le premier point est rédhibitoire, le second aussi au-delà de 2 mm ; en
/// dessous il ressort en avertissement, parce qu'un écart de quelques dixièmes
/// est une question de cotation à trancher, pas une barre à jeter.
pub fn check_rear_bar(speaker: &SpeakerModel) -> Result<Vec<BarWarning>, BarInconsistency> {
    let m = &speaker.mechanical;
    let bar = &m.rear_bar;
    let geo = SpeakerGeometry::compute(speaker);
    let mut warnings = Vec::new();

    let mut hard = |what: &str, expected: f64, declared: f64| -> Result<(), BarInconsistency> {
        let delta = declared - expected;
        if delta.abs() > BAR_FIT_HARD_LIMIT_MM {
            return Err(BarInconsistency {
                reason: format!(
                    "{what} : déclaré {declared:.3} mm contre {expected:.3} mm sur la \
                     jonction, écart {delta:.3} mm"
                ),
            });
        }
        if delta.abs() > BAR_FIT_TOLERANCE_MM {
            warnings.push(BarWarning {
                what: what.to_string(),
                expected_mm: expected,
                declared_mm: declared,
                delta_mm: delta,
            });
        }
        Ok(())
    };

    // --- 1. Cohérence interne du dessin -------------------------------------
    // Le profil de largeur doit être croissant en abscisse et couvrir la barre :
    // une ligne brisée qui revient en arrière ne décrit aucune pièce.
    for w in bar.width_profile.windows(2) {
        if w[1][0] < w[0][0] {
            return Err(BarInconsistency {
                reason: format!(
                    "profil de largeur non monotone : abscisse {} après {}",
                    w[1][0], w[0][0]
                ),
            });
        }
    }
    hard(
        "profil de largeur couvrant toute la barre",
        bar.length,
        bar.width_profile.last().map(|p| p[0]).unwrap_or(0.0),
    )?;

    // --- 2. La paire, dans le repère caisson --------------------------------
    // L'entraxe déclaré doit être celui que les deux polaires produisent. Les
    // deux trous n'étant plus sur le même cercle, c'est la seule façon de le
    // vérifier — aucune différence d'angles ne le donne.
    let pair_span = (geo.anchor_local - geo.latch_local).norm();
    hard("entraxe ancrage-verrou", m.latch_offset, pair_span)?;
    hard(
        "entraxe ancrage-verrou, repère barre",
        pair_span,
        bar.holes.anchor.along() - bar.holes.latch.along(),
    )?;

    // --- 3. L'axe de la barre -----------------------------------------------
    // Le verrou doit être sur la droite ancrage -> couronne extérieure : c'est
    // la définition même de l'axe, et une barre droite ne se monte pas
    // autrement. Vu depuis le caisson qui porte la paire, la couronne est à une
    // place fixe — indépendante du splay, puisque le caisson du bas tourne avec
    // elle.
    let crown_from_pair = |radius: f64| polar(geo.ht, radius, geo.ha + geo.splay0_angle);
    let up680_local = crown_from_pair(geo.crown_radius);
    let axis = (up680_local - geo.anchor_local).normalize();
    // Normale « vers l'avant » du caisson, celle qui donne son signe au déport
    // latéral. L'axe monte vers la couronne ; le tourner d'un quart de tour
    // direct pointe vers −x, donc vers l'avant (repère caisson : x va vers
    // l'arrière). Nommée plutôt que laissée à un ordre de produit vectoriel :
    // l'inverser mettrait `up660` du mauvais côté de la barre sans rien casser
    // d'autre.
    let front_normal = Vec2::new(-axis.y, axis.x);
    let off_axis = (geo.latch_local - geo.anchor_local).dot(front_normal).abs();
    if off_axis > BAR_AXIS_TOLERANCE_MM {
        return Err(BarInconsistency {
            reason: format!(
                "le verrou est à {off_axis:.4} mm de la droite ancrage-couronne : \
                 aucune barre droite ne passe par les trois trous"
            ),
        });
    }

    // --- 4. Les trous de couronne, repère barre -----------------------------
    // Chaque couronne a son entraxe et son déport. `up680` est sur l'axe,
    // `up660` en est écarté : c'est ce déport qui met l'effort de couronne hors
    // de l'axe sur les splays impairs, donc qui crée du moment là où une
    // abscisse seule n'en verrait aucun.
    for (label, radius, hole) in [
        ("extérieure", geo.crown_radius, bar.holes.up680),
        (
            "intérieure",
            geo.crown_radius - geo.crown_delta,
            bar.holes.up660,
        ),
    ] {
        let local = crown_from_pair(radius);
        let from_anchor = local - geo.anchor_local;
        hard(
            &format!("couronne {label} : entraxe depuis l'ancrage"),
            from_anchor.norm(),
            (hole.as_vec() - bar.holes.anchor.as_vec()).norm(),
        )?;
        // Déport latéral, compté positif vers l'avant du caisson comme la
        // convention du repère barre l'impose.
        hard(
            &format!("couronne {label} : déport latéral"),
            from_anchor.dot(front_normal),
            hole.lateral(),
        )?;
    }

    // --- 5. Distances au bord (EN 1993-1-8) ---------------------------------
    let min_edge = MIN_EDGE_DISTANCE_IN_D0 * bar.hole_diameter;
    let mut edge = |what: &str, value: f64| {
        if value < min_edge - 1e-9 {
            warnings.push(BarWarning {
                what: what.to_string(),
                expected_mm: min_edge,
                declared_mm: value,
                delta_mm: value - min_edge,
            });
        }
    };
    // e1 : le long de l'axe, depuis chaque bout.
    edge("e1 bout verrou", bar.holes.latch.along());
    edge("e1 bout couronne", bar.length - bar.holes.up680.along());
    // e2 : perpendiculairement, vers le bord le plus proche.
    edge("e2 verrou", bar.edge_distance_at(bar.holes.latch));
    edge("e2 ancrage", bar.edge_distance_at(bar.holes.anchor));
    edge("e2 up660", bar.edge_distance_at(bar.holes.up660));
    edge("e2 up680", bar.edge_distance_at(bar.holes.up680));

    Ok(warnings)
}

/// Silhouette de l'enceinte (trapèze), repère enceinte. Ne dépend que de ses
/// dimensions, jamais de la jonction ou de la grappe.
pub fn speaker_outline(speaker: &SpeakerModel) -> [Vec2; 4] {
    let m = &speaker.mechanical;
    let half_h = m.height / 2.0;
    let half_d = m.depth / 2.0;
    let taper = (m.total_vertical_angle / 2.0).to_radians().tan() * m.depth;
    [
        Vec2::new(-half_d, half_h),
        Vec2::new(half_d, half_h - taper),
        Vec2::new(half_d, -half_h + taper),
        Vec2::new(-half_d, -half_h),
    ]
}

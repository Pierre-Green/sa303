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
    anchor_angle: f64,
    latch_angle: f64,
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
        let anchor_local = polar(ht, m.crown.radius, -(ha + m.crown.anchor_angle));
        let latch_local = polar(ht, m.crown.radius, -(ha + m.crown.latch_angle));
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
            anchor_angle: m.crown.anchor_angle,
            latch_angle: m.crown.latch_angle,
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

    /// Ancrage de l'enceinte inférieure vue depuis l'enceinte supérieure, au splay donné.
    /// Piège (brief §3) : l'angle est `-(ha + anchor_angle) + s`, pas `ha - anchor_angle + s`.
    pub fn anchor_at(&self, splay_deg: f64) -> Vec2 {
        polar(
            self.pv_at(splay_deg),
            self.crown_radius,
            -(self.ha + self.anchor_angle) + splay_deg,
        )
    }

    /// Verrou de l'enceinte inférieure vu depuis l'enceinte supérieure.
    pub fn latch_at(&self, splay_deg: f64) -> Vec2 {
        polar(
            self.pv_at(splay_deg),
            self.crown_radius,
            -(self.ha + self.latch_angle) + splay_deg,
        )
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

    /// Recoupement trigonométrique du bras de levier (brief §3), pour contrôle.
    pub fn lever_check(&self, splay_deg: f64) -> f64 {
        let a = self.crown_radius;
        let b = self.crown_radius_at(splay_deg);
        let theta = (2.0 * self.ha + self.anchor_angle + self.splay0_angle).to_radians();
        a * b * theta.sin() / (a * a + b * b - 2.0 * a * b * theta.cos()).sqrt()
    }
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

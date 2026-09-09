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
#[derive(Clone, Copy, Debug)]
pub struct SpeakerGeometry {
    pub ha: f64,
    pub ht: Vec2,
    pub hb: Vec2,
    pub pv: Vec2,
    pub bielle_entraxe: f64,
    pub anchor_local: Vec2,
    crown_radius: f64,
    crown_delta: f64,
    anchor_angle: f64,
    splay0_angle: f64,
}

impl SpeakerGeometry {
    pub fn compute(speaker: &SpeakerModel) -> Self {
        let m = &speaker.mechanical;
        let ha = m.total_vertical_angle / 2.0;
        let ht = Vec2::new(m.hinge.x, m.hinge.y);
        let hb = Vec2::new(m.hinge.x, -m.hinge.y);
        let pv = Vec2::new(m.hinge.x, m.hinge.y - m.hinge.joint_separation);
        let bielle_entraxe = (hb - pv).norm();
        let anchor_local = polar(ht, m.crown.radius, -(ha + m.crown.anchor_angle));
        Self {
            ha,
            ht,
            hb,
            pv,
            bielle_entraxe,
            anchor_local,
            crown_radius: m.crown.radius,
            crown_delta: m.crown.delta,
            anchor_angle: m.crown.anchor_angle,
            splay0_angle: m.crown.splay0_angle,
        }
    }

    pub fn crown_radius_at(&self, splay_deg: f64) -> f64 {
        if is_odd_splay(splay_deg) {
            self.crown_radius - self.crown_delta
        } else {
            self.crown_radius
        }
    }

    /// Trou de couronne au splay donné, repère enceinte.
    pub fn crown(&self, splay_deg: f64) -> Vec2 {
        polar(
            self.pv,
            self.crown_radius_at(splay_deg),
            self.ha + self.splay0_angle + splay_deg,
        )
    }

    /// Ancrage de l'enceinte inférieure vue depuis l'enceinte supérieure, au splay donné.
    /// Piège (brief §3) : l'angle est `-(ha + anchor_angle) + s`, pas `ha - anchor_angle + s`.
    pub fn anchor_at(&self, splay_deg: f64) -> Vec2 {
        polar(
            self.pv,
            self.crown_radius,
            -(self.ha + self.anchor_angle) + splay_deg,
        )
    }

    /// Bras de levier de la barre orientation au splay donné.
    pub fn lever(&self, splay_deg: f64) -> f64 {
        let a = self.anchor_at(splay_deg);
        let c = self.crown(splay_deg);
        let u = (c - a).normalize();
        (a - self.pv).cross(u).abs()
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

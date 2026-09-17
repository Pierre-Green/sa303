//! Répartition d'un effort de barre sur la paire de goupilles qui l'encastre.
//!
//! La même règle sert à deux endroits — la barre d'une jonction, et celle qui
//! boulonne le bumper sur l'enceinte de référence — parce que c'est la même
//! liaison : une barre rigide reprise par deux goupilles. La garder en un seul
//! exemplaire évite que les deux se mettent à diverger.

use crate::vector::Vec2;

/// Répartit `force`, appliquée en `apply_at`, sur les deux goupilles `anchor`
/// et `latch`.
///
/// Répartition élastique à raideurs égales : chaque goupille prend la moitié de
/// la résultante, plus ou moins le couple qui équilibre le moment de `force`
/// autour du barycentre de la paire. Renvoie aussi ce moment (N·mm), qui est
/// **exactement** celui produisant le couple rendu — pas une reconstruction
/// `|V| × bras`, qui perdrait le bras transversal.
pub(super) fn split_over_pair(
    anchor: Vec2,
    latch: Vec2,
    apply_at: Vec2,
    force: Vec2,
) -> (Vec2, Vec2, f64) {
    let ab = anchor - latch;
    let d = ab.norm();
    let g_point = (anchor + latch) * 0.5;
    let m_g = (apply_at - g_point).cross(force);
    // perp(ab) tourné d'un quart de tour : (x, y) -> (−y, x).
    let perp = Vec2::new(-ab.y, ab.x) * (1.0 / d);
    let p = perp * (m_g / d);
    (force * 0.5 + p, force * 0.5 - p, m_g)
}

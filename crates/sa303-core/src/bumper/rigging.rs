//! Points d'accroche **réels** du bumper et de sa barre de déport : des trous
//! de manille percés, en nombre fini. Le solveur choisit parmi eux ; il ne
//! place plus jamais une accroche à un x continu.
//!
//! Tout est coté en coordonnées locales de la pièce, comme sur le plan :
//!
//! * **bumper** : origine au centre du dessous du bumper (face posée sur
//!   l'enceinte), x vers l'arrière — même sens que le repère enceinte —, y vers
//!   le haut ;
//! * **barre** : origine au centre de la barre, x le long de la barre, y vers le
//!   haut, barre dans son sens « normal ». L'origine en y est libre : la barre
//!   est placée par ses pattes, pas par son origine.
//!
//! Un montage de barre, c'est ses deux pattes goupillées dans deux trous de
//! liaison du bumper, dans un sens ou dans l'autre (miroir x → −x). On ne
//! saisit pas les positions de montage : on les **déduit** en cherchant les
//! paires de trous dont l'entraxe retombe sur celui des pattes. Changer la
//! barre, c'est changer le JSON, rien d'autre.

use serde::{Deserialize, Serialize};

use crate::vector::Vec2;

/// Charge maximale d'utilisation d'une manille 3,25 t classique, kg.
pub const DEFAULT_SHACKLE_WLL_KG: f64 = 3250.0;

fn default_wll_kg() -> f64 {
    DEFAULT_SHACKLE_WLL_KG
}

/// Écart toléré entre l'entraxe des pattes de la barre et celui de deux trous
/// de liaison du bumper, mm : au-delà, la goupille ne passe pas.
const LINK_MATCH_TOLERANCE_MM: f64 = 0.5;

/// Perçage d'accroche du bumper, repère bumper (voir l'en-tête du module).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperRigging {
    /// Trous de manille intégrés au bumper, `[x, y]` mm.
    pub shackle_holes: Vec<[f64; 2]>,
    /// Trous où se goupillent les pattes de la barre de déport, `[x, y]` mm.
    pub bar_link_holes: Vec<[f64; 2]>,
    /// Charge maximale par trou de manille, kg.
    #[serde(default = "default_wll_kg")]
    pub wll_kg: f64,
}

/// Géométrie de la barre de déport, repère barre (voir l'en-tête du module).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperBarGeometry {
    /// Trous d'accroche, `[x, y]` mm, dans l'ordre de la barre.
    pub pickup_holes: Vec<[f64; 2]>,
    /// Les deux pattes inférieures qui se goupillent au bumper, `[x, y]` mm.
    pub link_pins: [[f64; 2]; 2],
    /// Charge maximale par trou d'accroche, kg.
    #[serde(default = "default_wll_kg")]
    pub wll_kg: f64,
}

/// Une façon de monter la barre sur le bumper. Le passage repère barre →
/// repère bumper est `p_bumper = (±x, y) + translation`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarMount {
    /// Barre retournée (miroir x → −x) par rapport à son sens normal.
    pub flipped: bool,
    pub translation: Vec2,
    /// Abscisse du centre de la barre dans le repère bumper, mm : la cote
    /// « 258 / 288 » du rigger, signée (positif vers l'arrière).
    pub center_x_mm: f64,
}

impl BarMount {
    pub fn to_bumper(&self, p: [f64; 2]) -> Vec2 {
        let x = if self.flipped { -p[0] } else { p[0] };
        Vec2::new(x, p[1]) + self.translation
    }
}

fn v(p: [f64; 2]) -> Vec2 {
    Vec2::new(p[0], p[1])
}

/// Tous les montages de barre physiquement possibles : chaque sens de barre,
/// chaque paire ordonnée de trous de liaison dont le vecteur est celui des
/// pattes (la barre reste parallèle au bumper). Triés du plus centré au plus
/// déporté, l'avant avant l'arrière à cote égale — c'est l'ordre de préférence
/// du solveur.
pub fn bar_mounts(bumper: &BumperRigging, bar: &BumperBarGeometry) -> Vec<BarMount> {
    let mut mounts = Vec::new();
    for flipped in [false, true] {
        let pin = |i: usize| {
            let p = bar.link_pins[i];
            Vec2::new(if flipped { -p[0] } else { p[0] }, p[1])
        };
        let (pin_a, pin_b) = (pin(0), pin(1));
        for (i, &hole_a) in bumper.bar_link_holes.iter().enumerate() {
            for (j, &hole_b) in bumper.bar_link_holes.iter().enumerate() {
                if i == j {
                    continue;
                }
                let mismatch = (v(hole_b) - v(hole_a)) - (pin_b - pin_a);
                if mismatch.norm() > LINK_MATCH_TOLERANCE_MM {
                    continue;
                }
                let translation = v(hole_a) - pin_a;
                mounts.push(BarMount {
                    flipped,
                    translation,
                    center_x_mm: translation.x,
                });
            }
        }
    }
    mounts.sort_by(|a, b| {
        a.center_x_mm
            .abs()
            .total_cmp(&b.center_x_mm.abs())
            .then(a.center_x_mm.total_cmp(&b.center_x_mm))
    });
    mounts
}

/// Silhouette **schématique** de la barre, repère barre : une plaque dont le
/// bord haut suit l'arc des trous et le bord bas est droit, et deux pattes qui
/// descendent à la verticale jusqu'à leur goupille. Pour le dessin seulement :
/// aucune de ces marges n'entre dans un calcul.
pub fn bar_outline_local(bar: &BumperBarGeometry) -> Vec<[f64; 2]> {
    const ABOVE_HOLES_MM: f64 = 25.0;
    const BELOW_LOWEST_HOLE_MM: f64 = 45.0;
    const END_MARGIN_MM: f64 = 35.0;
    const LEG_HALF_WIDTH_MM: f64 = 27.5;
    const BELOW_PIN_MM: f64 = 30.0;

    let mut holes = bar.pickup_holes.clone();
    holes.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let (Some(&first), Some(&last)) = (holes.first(), holes.last()) else {
        return Vec::new();
    };
    let lowest = holes.iter().map(|h| h[1]).fold(f64::INFINITY, f64::min);
    let bottom = lowest - BELOW_LOWEST_HOLE_MM;
    let (left, right) = (first[0] - END_MARGIN_MM, last[0] + END_MARGIN_MM);

    let mut outline = vec![[left, bottom], [left, first[1] + ABOVE_HOLES_MM]];
    outline.extend(holes.iter().map(|h| [h[0], h[1] + ABOVE_HOLES_MM]));
    outline.push([right, last[1] + ABOVE_HOLES_MM]);
    outline.push([right, bottom]);
    // Bord bas, de droite à gauche, en descendant dans chaque patte au passage.
    let mut pins = bar.link_pins.to_vec();
    pins.sort_by(|a, b| b[0].total_cmp(&a[0]));
    for [x, y] in pins {
        let foot = y - BELOW_PIN_MM;
        outline.extend([
            [x + LEG_HALF_WIDTH_MM, bottom],
            [x + LEG_HALF_WIDTH_MM, foot],
            [x - LEG_HALF_WIDTH_MM, foot],
            [x - LEG_HALF_WIDTH_MM, bottom],
        ]);
    }
    outline
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Données provisoires de la SA303 : les quatre montages attendus par le
    /// rigger (centre de barre à ±258 et ±288 mm) doivent sortir des seuls
    /// perçages.
    #[test]
    fn provisional_sa303_bar_yields_the_four_known_mounts() {
        let bumper = BumperRigging {
            shackle_holes: vec![],
            bar_link_holes: vec![[-210.0, 160.0], [-180.0, 160.0], [180.0, 160.0], [210.0, 160.0]],
            wll_kg: DEFAULT_SHACKLE_WLL_KG,
        };
        let bar = BumperBarGeometry {
            pickup_holes: vec![],
            link_pins: [[-468.0, 0.0], [-78.0, 0.0]],
            wll_kg: DEFAULT_SHACKLE_WLL_KG,
        };
        let centers: Vec<f64> = bar_mounts(&bumper, &bar).iter().map(|m| m.center_x_mm).collect();
        assert_eq!(centers, vec![-258.0, 258.0, -288.0, 288.0]);
    }
}

//! Vecteur 2D partagé par tous les modules métier (`speaker`, `bumper`,
//! `cluster`, `tie`) et convention angulaire du brief (§2 : origine au centre
//! de l'enceinte, x -> arrière, y -> haut ; angle de sortie 0° vers le bas,
//! sens horaire, 90° vers l'avant). Le calcul vectoriel non trivial
//! (rotation, produit vectoriel, normalisation sûre) est délégué à
//! `glam::DVec2` ; le type reste néanmoins le nôtre, pas un simple alias,
//! pour garder la forme JSON `{x, y}` attendue par le front, indépendante du
//! choix de bibliothèque de calcul interne.

use glam::DVec2;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn glam(self) -> DVec2 {
        DVec2::new(self.x, self.y)
    }

    fn from_glam(v: DVec2) -> Self {
        Vec2::new(v.x, v.y)
    }

    pub fn dot(self, o: Vec2) -> f64 {
        self.glam().dot(o.glam())
    }

    pub fn cross(self, o: Vec2) -> f64 {
        self.glam().perp_dot(o.glam())
    }

    pub fn norm(self) -> f64 {
        self.glam().length()
    }

    pub fn normalize(self) -> Vec2 {
        Vec2::from_glam(self.glam().normalize_or_zero())
    }

    /// R(phi) * v : repère enceinte -> repère global. `phi` en radians.
    pub fn rotate(self, phi: f64) -> Vec2 {
        Vec2::from_glam(self.glam().rotate_angle(phi))
    }

    /// R(phi)^T * v : repère global -> repère enceinte. `phi` en radians.
    /// Transposée d'une rotation = rotation d'angle opposé (matrice orthogonale).
    pub fn rotate_transpose(self, phi: f64) -> Vec2 {
        self.rotate(-phi)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

/// Point à distance `r` et angle `angle_deg` (degrés) autour de `center`.
pub fn polar(center: Vec2, r: f64, angle_deg: f64) -> Vec2 {
    let a = angle_deg.to_radians();
    Vec2::new(center.x + r * a.cos(), center.y + r * a.sin())
}

/// Convention angulaire de sortie (brief §2) : 0° vers le bas, horaire, 90° vers l'avant.
pub fn dir_from_angle(theta_deg: f64) -> Vec2 {
    let t = theta_deg.to_radians();
    Vec2::new(-t.sin(), -t.cos())
}

/// Angle dans la convention de sortie, dans [0, 360).
pub fn angle_of(v: Vec2) -> f64 {
    let a = (-v.x).atan2(-v.y).to_degrees();
    ((a % 360.0) + 360.0) % 360.0
}

/// Intersection d'un rayon (origine + direction, `dir` non nul) avec le
/// segment `[a, b]`. `true` si le rayon croise le segment strictement devant
/// son origine (`t > 0`), peu importe à quelle distance.
fn ray_hits_segment(origin: Vec2, dir: Vec2, a: Vec2, b: Vec2) -> bool {
    let e = b - a;
    let rxs = dir.cross(e);
    if rxs.abs() < 1e-12 {
        return false; // rayon parallèle au segment : pas de croisement franc
    }
    let ao = a - origin;
    let t = ao.cross(e) / rxs;
    let u = ao.cross(dir) / rxs;
    t > 1e-6 && (-1e-9..=1.0 + 1e-9).contains(&u)
}

/// Collision rayon/polygone : sert à vérifier qu'une tirette ne traverse
/// aucune enceinte de la grappe avant d'atteindre son ancrage (brief,
/// correction utilisateur — "juste un vecteur avec un calcul de collision").
/// `true` si le rayon (origine + direction) croise le contour fermé `poly`
/// (ses arêtes prises dans l'ordre, y compris la dernière -> première).
/// Générique, sans dépendance à la géométrie de l'enceinte : marche pour
/// n'importe quel contour à 4 sommets.
pub fn ray_hits_polygon(origin: Vec2, dir: Vec2, poly: &[Vec2; 4]) -> bool {
    (0..poly.len()).any(|i| {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        ray_hits_segment(origin, dir, a, b)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(cx: f64, cy: f64, half: f64) -> [Vec2; 4] {
        [
            Vec2::new(cx - half, cy + half),
            Vec2::new(cx + half, cy + half),
            Vec2::new(cx + half, cy - half),
            Vec2::new(cx - half, cy - half),
        ]
    }

    #[test]
    fn ray_hits_polygon_directly_ahead() {
        let poly = square(0.0, 10.0, 2.0);
        assert!(ray_hits_polygon(Vec2::ZERO, Vec2::new(0.0, 1.0), &poly));
    }

    #[test]
    fn ray_misses_polygon_to_the_side() {
        let poly = square(10.0, 10.0, 2.0);
        assert!(!ray_hits_polygon(Vec2::ZERO, Vec2::new(0.0, 1.0), &poly));
    }

    #[test]
    fn ray_does_not_hit_what_is_behind_its_origin() {
        let poly = square(0.0, -10.0, 2.0);
        assert!(!ray_hits_polygon(Vec2::ZERO, Vec2::new(0.0, 1.0), &poly));
    }

    #[test]
    fn ray_hits_polygon_at_an_angle() {
        let poly = square(10.0, 10.0, 3.0);
        // Direction (1,1) normalisée : passe par (10,10), doit toucher le carré.
        assert!(ray_hits_polygon(Vec2::ZERO, Vec2::new(1.0, 1.0), &poly));
    }

    #[test]
    fn ray_grazing_just_past_the_corner_misses() {
        let poly = square(10.0, 10.0, 2.0);
        // Le carré occupe x∈[8,12] pour y∈[8,12] ; sur ce même intervalle de
        // y, cette direction reste à x∈[16,24] — strictement à côté.
        assert!(!ray_hits_polygon(Vec2::ZERO, Vec2::new(20.0, 10.0), &poly));
    }
}

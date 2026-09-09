//! Tirette basse : tension et direction. Un câble ne peut que tirer, jamais
//! pousser, ce qui délimite une plage de directions physiquement valables
//! (`tie_valid_angle_range_deg`) pour un point d'accroche et un point de
//! levage donnés — l'utilisateur choisit son propre point d'ancrage réel
//! dedans (`tie_tension` accepte n'importe quel angle et laisse l'appelant
//! vérifier le signe) ; `optimal_tie_direction`/`tie_default_angle_deg` ne
//! servent qu'à suggérer un point de départ, jamais à décider à sa place.

use crate::cluster::SpeakerInstance;
use crate::vector::{angle_of, dir_from_angle, Vec2};

/// Tirette basse déjà résolue en force globale, prête à être ajoutée au corps libre.
#[derive(Clone, Copy, Debug)]
pub struct TieForce {
    /// Point d'accroche sur l'enceinte du bas, repère enceinte.
    pub point_local: Vec2,
    /// Force globale (déjà mise à l'échelle par la tension).
    pub force: Vec2,
}

/// Moment du poids de la grappe autour du point de levage, et positions
/// utiles — partagé par `tie_tension`, `optimal_tie_direction` et
/// `tie_valid_angle_range_deg`, qui en dérivent chacun une chose différente.
struct TieMoment {
    pk: Vec2,
    q: Vec2,
    mw: f64,
}

fn tie_moment(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    tie_point_local: Vec2,
) -> TieMoment {
    let pk = speakers[0].o + pickup_local.rotate(speakers[0].phi);
    let last = speakers[speakers.len() - 1];
    let q = last.o + tie_point_local.rotate(last.phi);
    let mw = (cg_global - pk).cross(Vec2::new(0.0, -weight_n));
    TieMoment { pk, q, mw }
}

/// Tension nécessaire pour tenir l'assiette avec une tirette tirant dans la
/// direction `tie_angle_deg` (convention §2) depuis `tie_point_local`. Peut
/// sortir négative : un câble ne peut que tirer, donc une tension négative
/// signale que cette direction précise est physiquement intenable (il
/// faudrait pousser) — à l'appelant de le vérifier, ce n'est pas la
/// responsabilité de cette fonction de choisir une direction à la place.
pub fn tie_tension(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    tie_point_local: Vec2,
    tie_angle_deg: f64,
) -> f64 {
    let m = tie_moment(speakers, weight_n, cg_global, pickup_local, tie_point_local);
    let u = dir_from_angle(tie_angle_deg);
    let d = (m.q - m.pk).cross(u);
    if d.abs() < 1e-9 {
        0.0
    } else {
        -m.mw / d
    }
}

/// Direction (unitaire) et tension d'une tirette qui minimise l'effort parmi
/// toutes les directions physiquement valables (tension positive — un câble
/// ne peut que tirer, jamais pousser) : la perpendiculaire à l'axe
/// accroche-tirette, du côté qui tire. Sert de valeur par défaut quand
/// l'utilisateur n'a pas encore choisi d'angle dans la plage affichée
/// (`tie_valid_angle_range_deg`).
pub fn optimal_tie_direction(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    tie_point_local: Vec2,
) -> (Vec2, f64) {
    let m = tie_moment(speakers, weight_n, cg_global, pickup_local, tie_point_local);
    let qp = m.q - m.pk;
    let perp = qp.normalize().rotate(std::f64::consts::FRAC_PI_2);
    let d = qp.cross(perp);
    if d.abs() < 1e-9 {
        return (perp, 0.0);
    }
    let t = -m.mw / d;
    if t >= 0.0 {
        (perp, t)
    } else {
        (-perp, -t)
    }
}

/// Plage (bornes exclues) des directions de traction physiquement valables,
/// en degrés (convention §2) : exactement un demi-cercle, centré sur la
/// direction optimale, au-delà duquel la tension deviendrait négative
/// (pousser au lieu de tirer). L'utilisateur choisit son propre point
/// d'ancrage réel dans cette plage — ce n'est pas à l'algorithme de décider
/// à sa place, seulement de délimiter ce qui est tenable.
pub fn tie_valid_angle_range_deg(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    tie_point_local: Vec2,
) -> (f64, f64) {
    let (dir, _) =
        optimal_tie_direction(speakers, weight_n, cg_global, pickup_local, tie_point_local);
    let center = angle_of(dir);
    let mut lower = center - 90.0;
    if lower < 0.0 {
        lower += 360.0;
    }
    (lower, lower + 180.0)
}

/// Suggestion affichée tant que l'utilisateur n'a pas choisi sa propre
/// direction : pas forcément celle qui minimise la tension, mais celle qui
/// se rapproche le plus de 180° (vers l'arrière, la direction la plus
/// courante en pratique) sans sortir de la plage valable — un point de
/// départ réaliste, jamais un choix imposé.
pub fn tie_default_angle_deg(lower: f64, upper: f64) -> f64 {
    let contains = |angle: f64| {
        (angle >= lower && angle <= upper) || (angle + 360.0 >= lower && angle + 360.0 <= upper)
    };
    if contains(180.0) {
        return 180.0;
    }
    let circ_dist = |a: f64, b: f64| {
        let d = (a - b).rem_euclid(360.0);
        d.min(360.0 - d)
    };
    if circ_dist(lower, 180.0) <= circ_dist(upper, 180.0) {
        lower
    } else {
        upper
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une enceinte de test réduite à ce dont la tirette a besoin : position,
    /// inclinaison, CG. La silhouette n'entre pas dans le calcul de tirette.
    fn instance(cg: Vec2) -> SpeakerInstance {
        SpeakerInstance {
            o: Vec2::ZERO,
            phi: 0.0,
            cg,
            outline: [Vec2::ZERO; 4],
        }
    }

    #[test]
    fn optimal_tie_direction_is_perpendicular_positive_and_in_equilibrium() {
        let speakers = [instance(Vec2::new(3.0, 0.0))];
        let weight_n = 10.0;
        let cg_global = speakers[0].cg;
        let pickup_local = Vec2::new(0.0, 10.0);
        let tie_point_local = Vec2::new(5.0, -10.0);

        let (dir, tension) = optimal_tie_direction(
            &speakers,
            weight_n,
            cg_global,
            pickup_local,
            tie_point_local,
        );

        assert!(tension >= 0.0, "un câble ne peut que tirer");
        assert!((dir.norm() - 1.0).abs() < 1e-9, "direction non unitaire");

        let pk = pickup_local; // speakers[0].o = 0, phi = 0
        let q = tie_point_local;
        let qp = q - pk;
        assert!(
            qp.dot(dir).abs() < 1e-9,
            "la direction optimale doit être perpendiculaire à l'axe accroche-tirette"
        );

        // Invariant qui ne peut pas mentir (brief §7) : moment du poids +
        // moment de la tirette autour du point de levage doit s'annuler.
        let weight_moment = (speakers[0].cg - pk).cross(Vec2::new(0.0, -weight_n));
        let tie_moment = (q - pk).cross(dir * tension);
        assert!(
            (weight_moment + tie_moment).abs() < 1e-6,
            "résidu de moment trop grand : {}",
            weight_moment + tie_moment
        );
    }

    #[test]
    fn optimal_tie_direction_picks_the_positive_tension_side() {
        // Retourner la perpendiculaire opposée doit donner l'exacte tension
        // négative de celle retenue : le solveur choisit toujours le côté qui
        // tire, jamais celui qui pousserait.
        let speakers = [instance(Vec2::new(-3.0, 0.0))];
        let (dir, tension) = optimal_tie_direction(
            &speakers,
            10.0,
            speakers[0].cg,
            Vec2::new(0.0, 10.0),
            Vec2::new(5.0, -10.0),
        );
        assert!(tension >= 0.0);
        // Le côté opposé (cg symétrique en x) doit donner la direction opposée.
        let speakers_mirrored = [instance(Vec2::new(3.0, 0.0))];
        let (dir_mirrored, _) = optimal_tie_direction(
            &speakers_mirrored,
            10.0,
            speakers_mirrored[0].cg,
            Vec2::new(0.0, 10.0),
            Vec2::new(5.0, -10.0),
        );
        assert!((dir.x + dir_mirrored.x).abs() < 1e-9);
        assert!((dir.y + dir_mirrored.y).abs() < 1e-9);
    }

    #[test]
    fn tie_default_angle_picks_180_when_in_range() {
        assert_eq!(tie_default_angle_deg(120.0, 300.0), 180.0);
    }

    #[test]
    fn tie_default_angle_picks_nearest_bound_when_180_is_outside() {
        // Plage 190°..370° (soit 190°..10°) : ne contient pas 180°, la borne
        // la plus proche est 190° (10° d'écart, contre 170° pour l'autre côté).
        assert_eq!(tie_default_angle_deg(190.0, 370.0), 190.0);
        // Plage 350°..530° (soit 350°..170°) : ne contient pas 180° non plus
        // (elle s'arrête à 170°) ; la borne la plus proche est 530° (=170°,
        // 10° d'écart, contre 170° pour l'autre côté).
        assert_eq!(tie_default_angle_deg(350.0, 530.0), 530.0);
    }
}

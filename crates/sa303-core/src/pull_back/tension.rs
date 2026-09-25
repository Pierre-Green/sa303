//! Pull-back : tension et direction. Le pull-back est un **second point de
//! levage** (moteur ou palan) accroché au trou de couronne 0° de l'enceinte du
//! bas, qui tire **verticalement vers le haut** (pratique Meyer Sound, « Pull
//! Back Rigging Operating Instructions », PN 05.083.008.01 : rester à ±10° de
//! la verticale). Il porte une partie de la grappe par en dessous, ce qui met
//! une partie de la chaîne en **compression** et décharge la manille
//! principale (`supportForceN` = W·k − T). C'est tout son intérêt : il
//! n'existe pas de pull-back tirant vers le bas.
//!
//! Convention d'angle (§2, `dir_from_angle`) : 0° vers le bas, horaire, 90°
//! vers l'avant, **180° vers le haut**, 270° vers l'arrière. La direction est
//! donc restreinte à 180° ± `Settings::pull_back_tolerance_deg`
//! (`pull_back_usable_range_deg`), et un câble ne pouvant que tirer, seule la
//! partie de cette fenêtre où la tension reste positive est retenue.

use crate::cluster::SpeakerInstance;
use crate::vector::{angle_of, dir_from_angle, Vec2};

/// Pull-back déjà résolu en force globale, prêt à être ajouté au corps libre.
#[derive(Clone, Copy, Debug)]
pub struct PullBackForce {
    /// Point d'accroche sur l'enceinte du bas, repère enceinte.
    pub point_local: Vec2,
    /// Force globale (déjà mise à l'échelle par la tension).
    pub force: Vec2,
}

/// Moment du poids de la grappe autour du point de levage, et positions
/// utiles — partagé par `pull_back_tension`, `optimal_pull_back_direction` et
/// `pull_back_valid_angle_range_deg`, qui en dérivent chacun une chose différente.
struct PullBackMoment {
    pk: Vec2,
    q: Vec2,
    mw: f64,
}

fn pull_back_moment(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    pull_back_point_local: Vec2,
) -> PullBackMoment {
    let pk = speakers[0].o + pickup_local.rotate(speakers[0].phi);
    let last = speakers[speakers.len() - 1];
    let q = last.o + pull_back_point_local.rotate(last.phi);
    let mw = (cg_global - pk).cross(Vec2::new(0.0, -weight_n));
    PullBackMoment { pk, q, mw }
}

/// Tension nécessaire pour tenir l'assiette avec un pull-back tirant dans la
/// direction `pull_back_angle_deg` (convention §2) depuis `pull_back_point_local`. Peut
/// sortir négative : un câble ne peut que tirer, donc une tension négative
/// signale que cette direction précise est physiquement intenable (il
/// faudrait pousser) — à l'appelant de le vérifier, ce n'est pas la
/// responsabilité de cette fonction de choisir une direction à la place.
pub fn pull_back_tension(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    pull_back_point_local: Vec2,
    pull_back_angle_deg: f64,
) -> f64 {
    let m = pull_back_moment(speakers, weight_n, cg_global, pickup_local, pull_back_point_local);
    let u = dir_from_angle(pull_back_angle_deg);
    let d = (m.q - m.pk).cross(u);
    if d.abs() < 1e-9 {
        0.0
    } else {
        -m.mw / d
    }
}

/// Direction (unitaire) et tension d'un pull-back qui minimise l'effort parmi
/// toutes les directions physiquement valables (tension positive — un câble
/// ne peut que tirer, jamais pousser) : la perpendiculaire à l'axe
/// accroche–pull-back, du côté qui tire. Sert de valeur par défaut quand
/// l'utilisateur n'a pas encore choisi d'angle dans la plage affichée
/// (`pull_back_valid_angle_range_deg`).
pub fn optimal_pull_back_direction(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    pull_back_point_local: Vec2,
) -> (Vec2, f64) {
    let m = pull_back_moment(speakers, weight_n, cg_global, pickup_local, pull_back_point_local);
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
pub fn pull_back_valid_angle_range_deg(
    speakers: &[SpeakerInstance],
    weight_n: f64,
    cg_global: Vec2,
    pickup_local: Vec2,
    pull_back_point_local: Vec2,
) -> (f64, f64) {
    let (dir, _) =
        optimal_pull_back_direction(speakers, weight_n, cg_global, pickup_local, pull_back_point_local);
    let center = angle_of(dir);
    let mut lower = center - 90.0;
    if lower < 0.0 {
        lower += 360.0;
    }
    (lower, lower + 180.0)
}

/// Direction nominale du pull-back : **180° = verticalement vers le haut**
/// (convention §2).
pub const PULL_BACK_VERTICAL_DEG: f64 = 180.0;

/// Tolérance par défaut autour de la verticale, en degrés : ±10° de Meyer
/// Sound. Réglable par `Settings::pull_back_tolerance_deg`.
pub const DEFAULT_PULL_BACK_TOLERANCE_DEG: f64 = 10.0;

/// La tolérance ramenée à ce qui garde le câble montant : [0°, 89°].
fn clamp_tolerance(tolerance_deg: f64) -> f64 {
    if tolerance_deg.is_finite() {
        tolerance_deg.clamp(0.0, 89.0)
    } else {
        DEFAULT_PULL_BACK_TOLERANCE_DEG
    }
}

/// Fenêtre verticale 180° ± `tolerance_deg`, en degrés.
pub fn pull_back_window_deg(tolerance_deg: f64) -> (f64, f64) {
    let tol = clamp_tolerance(tolerance_deg);
    (PULL_BACK_VERTICAL_DEG - tol, PULL_BACK_VERTICAL_DEG + tol)
}

/// `true` si `angle_deg` (modulo 360°) est dans la fenêtre verticale
/// 180° ± `tolerance_deg`.
pub fn is_within_pull_back_window(angle_deg: f64, tolerance_deg: f64) -> bool {
    let tol = clamp_tolerance(tolerance_deg);
    (angle_deg.rem_euclid(360.0) - PULL_BACK_VERTICAL_DEG).abs() <= tol + 1e-9
}

/// Marge, en degrés, gardée à l'intérieur d'une borne de plage qui vient de
/// la statique : sur cette borne, le bras de levier du câble est nul et la
/// tension infinie (`pull_back_tension` y renverrait même 0). La plage annoncée
/// s'arrête donc un dixième de degré avant.
pub const PULL_BACK_STATIC_MARGIN_DEG: f64 = 0.1;

/// Plage des directions de pull-back réellement utilisables, en degrés dans
/// [0°, 360°) : l'intersection de la fenêtre verticale 180° ± `tolerance_deg`
/// et du demi-cercle statique `valid_range` (tension positive), resserrée de
/// `PULL_BACK_STATIC_MARGIN_DEG` du côté des bornes statiques. `None` si
/// elles ne se recouvrent pas : à cette assiette, aucun pull-back vertical ne
/// tire, il faudrait pousser (ou tirer vers le bas).
pub fn pull_back_usable_range_deg(valid_range: (f64, f64), tolerance_deg: f64) -> Option<(f64, f64)> {
    let tol = clamp_tolerance(tolerance_deg);
    let (w_lo, w_hi) = (PULL_BACK_VERTICAL_DEG - tol, PULL_BACK_VERTICAL_DEG + tol);
    let lo = valid_range.0.rem_euclid(360.0);
    let m = PULL_BACK_STATIC_MARGIN_DEG;
    // Le demi-cercle peut chevaucher 0° : on essaie ses deux représentants.
    [lo - 360.0, lo, lo + 360.0].into_iter().find_map(|l| {
        let (a, b) = (w_lo.max(l + m), w_hi.min(l + 180.0 - m));
        (b - a > 1e-9).then_some((a, b))
    })
}

/// Ramène un angle saisi dans la plage utilisable : modulo 360°, puis borné.
/// C'est ce que fait aussi le champ de saisie ; le solveur le refait pour
/// une grappe enregistrée dont la plage a bougé avec l'assiette.
pub fn clamp_to_pull_back_range(angle_deg: f64, range: (f64, f64)) -> f64 {
    angle_deg.rem_euclid(360.0).clamp(range.0, range.1)
}

/// Suggestion tant que l'utilisateur n'a pas choisi : 180° (verticale
/// exacte) si c'est dans la plage, sinon la borne la plus proche de 180°.
pub fn pull_back_default_angle_deg(range: (f64, f64)) -> f64 {
    PULL_BACK_VERTICAL_DEG.clamp(range.0, range.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une enceinte de test réduite à ce dont le pull-back a besoin : position,
    /// inclinaison, CG. La silhouette n'entre pas dans le calcul de pull-back.
    fn instance(cg: Vec2) -> SpeakerInstance {
        SpeakerInstance {
            o: Vec2::ZERO,
            phi: 0.0,
            cg,
            outline: [Vec2::ZERO; 4],
        }
    }

    #[test]
    fn optimal_pull_back_direction_is_perpendicular_positive_and_in_equilibrium() {
        let speakers = [instance(Vec2::new(3.0, 0.0))];
        let weight_n = 10.0;
        let cg_global = speakers[0].cg;
        let pickup_local = Vec2::new(0.0, 10.0);
        let pull_back_point_local = Vec2::new(5.0, -10.0);

        let (dir, tension) = optimal_pull_back_direction(
            &speakers,
            weight_n,
            cg_global,
            pickup_local,
            pull_back_point_local,
        );

        assert!(tension >= 0.0, "un câble ne peut que tirer");
        assert!((dir.norm() - 1.0).abs() < 1e-9, "direction non unitaire");

        let pk = pickup_local; // speakers[0].o = 0, phi = 0
        let q = pull_back_point_local;
        let qp = q - pk;
        assert!(
            qp.dot(dir).abs() < 1e-9,
            "la direction optimale doit être perpendiculaire à l'axe accroche–pull-back"
        );

        // Invariant qui ne peut pas mentir (brief §7) : moment du poids +
        // moment du pull-back autour du point de levage doit s'annuler.
        let weight_moment = (speakers[0].cg - pk).cross(Vec2::new(0.0, -weight_n));
        let pull_back_moment = (q - pk).cross(dir * tension);
        assert!(
            (weight_moment + pull_back_moment).abs() < 1e-6,
            "résidu de moment trop grand : {}",
            weight_moment + pull_back_moment
        );
    }

    #[test]
    fn optimal_pull_back_direction_picks_the_positive_tension_side() {
        // Retourner la perpendiculaire opposée doit donner l'exacte tension
        // négative de celle retenue : le solveur choisit toujours le côté qui
        // tire, jamais celui qui pousserait.
        let speakers = [instance(Vec2::new(-3.0, 0.0))];
        let (dir, tension) = optimal_pull_back_direction(
            &speakers,
            10.0,
            speakers[0].cg,
            Vec2::new(0.0, 10.0),
            Vec2::new(5.0, -10.0),
        );
        assert!(tension >= 0.0);
        // Le côté opposé (cg symétrique en x) doit donner la direction opposée.
        let speakers_mirrored = [instance(Vec2::new(3.0, 0.0))];
        let (dir_mirrored, _) = optimal_pull_back_direction(
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
    fn pull_back_nominal_direction_points_straight_up() {
        let d = dir_from_angle(PULL_BACK_VERTICAL_DEG);
        assert!(d.x.abs() < 1e-12 && (d.y - 1.0).abs() < 1e-12, "180° = +y = vers le haut");
    }

    #[test]
    fn pull_back_range_is_the_vertical_window_when_fully_valid() {
        assert_eq!(pull_back_usable_range_deg((120.0, 300.0), 10.0), Some((170.0, 190.0)));
        // Demi-cercle exprimé au-delà de 360° : même résultat.
        assert_eq!(pull_back_usable_range_deg((480.0, 660.0), 10.0), Some((170.0, 190.0)));
    }

    #[test]
    fn pull_back_range_is_clipped_by_the_static_half_circle() {
        // Tension positive seulement au-delà de 185° : la fenêtre est rognée.
        assert_eq!(pull_back_usable_range_deg((185.0, 365.0), 10.0), Some((185.1, 190.0)));
        // Tension positive seulement en deçà de 176°.
        let (a, b) = pull_back_usable_range_deg((356.0, 536.0), 10.0).unwrap();
        assert!(a == 170.0 && (b - 175.9).abs() < 1e-9, "{a}..{b}");
    }

    #[test]
    fn no_pull_back_when_only_downward_directions_pull() {
        // Demi-cercle centré sur 0° (vers le bas) : rien de vertical ne tire.
        assert_eq!(pull_back_usable_range_deg((270.0, 450.0), 10.0), None);
        assert_eq!(pull_back_usable_range_deg((190.0, 370.0), 10.0), None);
    }

    #[test]
    fn pull_back_tolerance_is_configurable() {
        assert_eq!(pull_back_usable_range_deg((90.0, 270.0), 25.0), Some((155.0, 205.0)));
        assert!(is_within_pull_back_window(165.0, 15.0));
        assert!(!is_within_pull_back_window(165.0, 10.0));
        assert!(is_within_pull_back_window(540.0, 0.0));
        assert!(!is_within_pull_back_window(300.0, 10.0));
    }

    #[test]
    fn a_typed_angle_is_brought_back_into_the_range() {
        let r = (170.0, 190.0);
        assert_eq!(clamp_to_pull_back_range(300.0, r), 190.0);
        assert_eq!(clamp_to_pull_back_range(0.0, r), 170.0);
        assert_eq!(clamp_to_pull_back_range(540.0, r), 180.0);
        assert_eq!(clamp_to_pull_back_range(-175.0, r), 185.0);
        assert_eq!(clamp_to_pull_back_range(183.0, r), 183.0);
    }

    #[test]
    fn pull_back_default_prefers_the_exact_vertical() {
        assert_eq!(pull_back_default_angle_deg((170.0, 190.0)), 180.0);
        assert_eq!(pull_back_default_angle_deg((185.0, 190.0)), 185.0);
        assert_eq!(pull_back_default_angle_deg((170.0, 176.0)), 176.0);
    }
}

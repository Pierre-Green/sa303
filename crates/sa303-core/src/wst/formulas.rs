//! Formules WST brutes (Urban, Heil, Bauman, *Wavefront Sculpture
//! Technology*, AES 5488, §3 à §6.2). Toutes en SI : longueurs en mètres,
//! angles en radians, fréquences en Hz — la conversion depuis/vers les unités
//! d'affichage (mm, degrés, kHz) est faite par `super::report`, pour qu'aucune
//! formule n'ait à s'en soucier.
//!
//! Le papier écrit ses formules avec l'approximation `λ = 1/(3F)` (F en kHz),
//! soit c ≈ 333,33 m/s. Ici la célérité est toujours explicite : passer
//! `SPEED_OF_SOUND_PAPER` retrouve exactement les chiffres publiés, passer
//! 343 m/s donne les valeurs réelles (~3 % plus hautes en fréquence).

/// Célérité usuelle à 20 °C.
pub const SPEED_OF_SOUND_DEFAULT: f64 = 343.0;
/// Célérité implicite du papier (`λ = 1/(3F)`, F en kHz) : à utiliser pour
/// recouper ses tableaux.
pub const SPEED_OF_SOUND_PAPER: f64 = 1000.0 / 3.0;

pub fn wavelength_m(speed_of_sound: f64, frequency_hz: f64) -> f64 {
    speed_of_sound / frequency_hz
}

// --- Critère 1 : Active Radiating Factor ------------------------------------

/// `ARF = D / STEP` : part de la hauteur du pas réellement occupée par la
/// source. Un ARF de 1 serait une ligne continue sans trou.
pub fn arf(radiating_height_m: f64, step_m: f64) -> f64 {
    if step_m <= 0.0 {
        return 0.0;
    }
    radiating_height_m / step_m
}

/// ARF minimal pour que le premier lobe secondaire reste à −13,5 dB, forme
/// exacte du papier : `0,82 · (1 + 1/(4,73·(N+1)))`.
pub fn arf_min(speaker_count: usize) -> f64 {
    0.82 * (1.0 + 1.0 / (4.73 * (speaker_count as f64 + 1.0)))
}

/// Niveau du lobe secondaire pour un grand N : `20·log10(ARF / (1 − ARF))`.
/// Renvoie `None` à ARF ≥ 1 (ligne continue : plus de lobe de réseau).
pub fn side_lobe_attenuation_db(arf: f64) -> Option<f64> {
    if arf <= 0.0 || arf >= 1.0 {
        return None;
    }
    Some(20.0 * (arf / (1.0 - arf)).log10())
}

/// Inverse de `side_lobe_attenuation_db` : ARF nécessaire pour une atténuation
/// visée.
pub fn arf_for_attenuation(attenuation_db: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(-attenuation_db / 20.0))
}

/// Perte de niveau sur l'axe due à la surface rayonnante manquante (dB, négatif).
pub fn axial_loss_db(arf: f64) -> f64 {
    if arf <= 0.0 {
        return f64::NEG_INFINITY;
    }
    20.0 * arf.log10()
}

// --- Critère 2 : pas devant rester sous λ/2 ---------------------------------

/// Fréquence au-delà de laquelle le pas dépasse λ/2 : `c / (2·STEP)`. En
/// dessous, aucun lobe de réseau ne peut exister, quel que soit l'ARF.
pub fn frequency_limit_half_wavelength(speed_of_sound: f64, step_m: f64) -> f64 {
    speed_of_sound / (2.0 * step_m)
}

/// Angle du lobe de réseau en champ lointain, `asin(λ / STEP)`. `None` tant
/// que λ > STEP : le lobe n'existe pas encore.
pub fn grating_lobe_angle_rad(wavelength_m: f64, step_m: f64) -> Option<f64> {
    asin_if_valid(wavelength_m / step_m)
}

/// Premier creux, `asin(λ / (2·STEP))`.
pub fn first_dip_angle_rad(wavelength_m: f64, step_m: f64) -> Option<f64> {
    asin_if_valid(wavelength_m / (2.0 * step_m))
}

// --- Critère 3 : planéité du front ------------------------------------------

/// Déviation maximale admissible d'un front plan à cette fréquence : `λ / 4`.
pub fn max_wavefront_deviation_m(speed_of_sound: f64, frequency_hz: f64) -> f64 {
    wavelength_m(speed_of_sound, frequency_hz) / 4.0
}

/// Fréquence jusqu'à laquelle une déviation mesurée reste isophase :
/// `c / (4·s)`.
pub fn isophase_frequency_limit(speed_of_sound: f64, deviation_m: f64) -> Option<f64> {
    if deviation_m <= 0.0 {
        return None;
    }
    Some(speed_of_sound / (4.0 * deviation_m))
}

// --- Champ proche / champ lointain (ligne plate) ----------------------------

/// En dessous de `c / H`, la ligne n'a pas de champ proche du tout.
pub fn no_near_field_below_hz(speed_of_sound: f64, line_height_m: f64) -> f64 {
    speed_of_sound / line_height_m
}

/// Frontière champ proche → lointain, forme complète du papier :
/// `(H²/(2λ))·sqrt(1 − (λ/H)²)`. `None` quand λ ≥ H (pas de champ proche).
pub fn near_field_boundary_m(line_height_m: f64, wavelength_m: f64) -> Option<f64> {
    let ratio = wavelength_m / line_height_m;
    if ratio >= 1.0 {
        return None;
    }
    Some((line_height_m.powi(2) / (2.0 * wavelength_m)) * (1.0 - ratio.powi(2)).sqrt())
}

/// Version Fresnel de la même frontière (~50 % plus proche) : `H² / (4λ)`. La
/// vraie transition est progressive entre les deux.
pub fn near_field_boundary_fresnel_m(line_height_m: f64, wavelength_m: f64) -> f64 {
    line_height_m.powi(2) / (4.0 * wavelength_m)
}

/// Premier creux vertical en champ lointain d'une ligne plate : `asin(λ / H)`.
pub fn line_first_dip_angle_rad(wavelength_m: f64, line_height_m: f64) -> Option<f64> {
    asin_if_valid(wavelength_m / line_height_m)
}

// --- Critère 5 : angle maximal entre deux caisses ---------------------------
//
// Le papier écrit ces quatre formules avec le produit `ARF · STEP`. Dans la
// dérivation du §6.2 ce produit ne vaut jamais autre chose que `D` : la
// demi-ouverture d'un élément y est `φ = λ/D`, et l'ARF géométrique n'apparaît
// nulle part. On prend donc `D` comme entrée directe — la **hauteur de bouche
// acoustique**, celle que voit le rayonnement, qui n'est pas exactement la
// bouche physique à cause de la diffraction de bride. Écrire `ARF · STEP`
// laisserait régler l'ARF et le pas indépendamment, donc produire une fréquence
// limite ne correspondant à aucune géométrie réelle.

/// Angle maximal admissible entre deux caisses adjacentes (rad) :
/// `2λ/D − STEP/d`. Négatif ou nul = aucun angle ne convient à cette
/// fréquence pour cet auditeur (caisse trop grande pour ce premier rang).
pub fn max_splay_rad(
    speed_of_sound: f64,
    frequency_hz: f64,
    acoustic_mouth_height_m: f64,
    step_m: f64,
    distance_m: f64,
) -> f64 {
    let lambda = wavelength_m(speed_of_sound, frequency_hz);
    2.0 * lambda / acoustic_mouth_height_m - step_m / distance_m
}

/// Fréquence maximale tenable à un angle et une distance donnés :
/// `2c / (D·(α + STEP/d))`. `distance_m` infini = auditeur à l'infini.
pub fn max_frequency_hz(
    speed_of_sound: f64,
    splay_rad: f64,
    acoustic_mouth_height_m: f64,
    step_m: f64,
    distance_m: f64,
) -> Option<f64> {
    let denominator = acoustic_mouth_height_m * (splay_rad + step_m / distance_m);
    if denominator <= 0.0 {
        return None;
    }
    Some(2.0 * speed_of_sound / denominator)
}

/// Distance minimale d'écoute à laquelle cet angle reste tenable à cette
/// fréquence : `STEP / (2λ/D − α)`. `None` si l'angle dépasse déjà la limite à
/// distance infinie.
pub fn min_distance_m(
    speed_of_sound: f64,
    frequency_hz: f64,
    splay_rad: f64,
    acoustic_mouth_height_m: f64,
    step_m: f64,
) -> Option<f64> {
    let lambda = wavelength_m(speed_of_sound, frequency_hz);
    let denominator = 2.0 * lambda / acoustic_mouth_height_m - splay_rad;
    if denominator <= 0.0 {
        return None;
    }
    Some(step_m / denominator)
}

/// Pas maximal pour qu'un angle strictement positif reste possible à cette
/// fréquence et cette distance : `2·λ·d / D`, la racine de `α_max = 0`.
///
/// À bouche acoustique fixée la relation est linéaire en `STEP`, là où la forme
/// `sqrt(2λd/ARF)` du papier suppose l'ARF constant — c'est-à-dire une bouche
/// qui s'agrandirait avec le pas. C'est la bouche qui est la donnée matérielle,
/// pas le rapport.
pub fn max_step_m(
    speed_of_sound: f64,
    frequency_hz: f64,
    acoustic_mouth_height_m: f64,
    distance_m: f64,
) -> f64 {
    let lambda = wavelength_m(speed_of_sound, frequency_hz);
    2.0 * lambda * distance_m / acoustic_mouth_height_m
}

// --- Critère 4 : courbure variable ------------------------------------------

/// Rayon de courbure local d'une portion à angle constant : `STEP / α`.
pub fn curvature_radius_m(step_m: f64, splay_rad: f64) -> Option<f64> {
    if splay_rad <= 0.0 {
        return None;
    }
    Some(step_m / splay_rad)
}

/// Niveau d'une ligne courbée relativement à la même ligne plate, en champ
/// proche : `−10·log10(1 + α·d/STEP)`.
pub fn curved_line_relative_level_db(splay_rad: f64, distance_m: f64, step_m: f64) -> f64 {
    -10.0 * (1.0 + splay_rad * distance_m / step_m).log10()
}

/// En dessous de cet angle la ligne se comporte comme plate : la forme courbée
/// n'est valable que pour `α > 4·STEP·λ / H²` (forme papier `4·STEP/(3F·H²)`,
/// réécrite avec λ explicite).
pub fn curved_model_min_splay_rad(step_m: f64, wavelength_m: f64, line_height_m: f64) -> f64 {
    4.0 * step_m * wavelength_m / line_height_m.powi(2)
}

// --- Géométrie CCA (angle constant) -----------------------------------------

/// Flèche d'un front plan sur la corde d'une caisse : `R·(1 − cos(α/2))`.
pub fn cca_sagitta_m(radius_m: f64, splay_rad: f64) -> f64 {
    radius_m * (1.0 - (splay_rad / 2.0).cos())
}

/// Retard de trajet au bord de bouche par rapport au centre, pour dessiner le
/// profil d'un guide courbé : `sqrt(R² + y²) − R`.
pub fn guide_path_delay_m(radius_m: f64, y_m: f64) -> f64 {
    (radius_m.powi(2) + y_m.powi(2)).sqrt() - radius_m
}

/// Angle de raccord entre une caisse isophase et une caisse à guide courbé.
///
/// La condition est la **tangence des deux fronts à la jonction** : chaque
/// front arrive au raccord incliné de la moitié de son propre secteur, donc
/// l'angle mécanique qui les aligne est `(θ_iso + θ_courbe) / 2`. La forme
/// `θ_courbe / 2` n'en est que le cas particulier d'un guide isophase
/// parfaitement plan (`θ_iso = 0`) — dès qu'il rayonne un secteur, même
/// petit, elle sous-estime le raccord.
pub fn transition_splay_rad(isophase_sector_rad: f64, curved_sector_rad: f64) -> f64 {
    (isophase_sector_rad + curved_sector_rad) / 2.0
}

/// Secteur rayonné par une bouche `D` dont le front a le rayon `R` : `D / R`.
/// C'est l'angle sous lequel la bouche est vue depuis le centre de courbure —
/// donc l'ouverture géométrique du front, pas une directivité à −6 dB.
/// `None` pour un front plan (rayon absent ou nul).
pub fn wavefront_sector_rad(mouth_height_m: f64, radius_m: Option<f64>) -> Option<f64> {
    let radius_m = radius_m?;
    if radius_m <= 0.0 || mouth_height_m <= 0.0 {
        return None;
    }
    Some(mouth_height_m / radius_m)
}

/// Écart au plan d'un front de rayon `R` sur une bouche de hauteur `D` :
/// `s = (D/2)² / (2R)`. C'est la vraie mesure de « à quel point le front est
/// plan » — à comparer à λ/4 via [`isophase_frequency_limit`]. `None` pour un
/// front plan.
pub fn wavefront_flatness_deviation_m(mouth_height_m: f64, radius_m: Option<f64>) -> Option<f64> {
    let radius_m = radius_m?;
    if radius_m <= 0.0 || mouth_height_m <= 0.0 {
        return None;
    }
    Some((mouth_height_m / 2.0).powi(2) / (2.0 * radius_m))
}

/// Premier creux d'interférence dans la zone où deux secteurs voisins se
/// recouvrent : `c / (2·STEP·sin(recouvrement/2))`. Deux sources distantes de
/// STEP vues sous un demi-angle de recouvrement s'y annulent pour la première
/// fois. `None` sans recouvrement (trou angulaire) : il n'y a alors aucune zone
/// commune où quoi que ce soit puisse interférer.
pub fn overlap_notch_frequency_hz(
    speed_of_sound: f64,
    step_m: f64,
    overlap_rad: f64,
) -> Option<f64> {
    if overlap_rad <= 0.0 || step_m <= 0.0 {
        return None;
    }
    let denominator = 2.0 * step_m * (overlap_rad / 2.0).sin();
    (denominator > 0.0).then(|| speed_of_sound / denominator)
}

fn asin_if_valid(ratio: f64) -> Option<f64> {
    if !(0.0..=1.0).contains(&ratio) {
        return None;
    }
    Some(ratio.asin())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Jeu du papier : ARF 0,8 / STEP 0,55 m / c = 333,33, soit une bouche de
    /// 0,44 m — c'est elle qui pilote désormais le critère 5.
    const ARF: f64 = 0.8;
    const STEP: f64 = 0.55;
    const MOUTH: f64 = ARF * STEP;

    fn f_max_khz(splay_deg: f64, distance_m: f64) -> f64 {
        max_frequency_hz(
            SPEED_OF_SOUND_PAPER,
            splay_deg.to_radians(),
            MOUTH,
            STEP,
            distance_m,
        )
        .unwrap()
            / 1000.0
    }

    #[test]
    fn criterion5_matches_the_published_frequency_vectors() {
        // Vecteurs de la spec (convention papier), arrondis à 0,1 kHz.
        assert!(
            (f_max_khz(5.0, 25.0) - 13.9).abs() < 0.05,
            "{}",
            f_max_khz(5.0, 25.0)
        );
        assert!((f_max_khz(5.0, 35.0) - 14.7).abs() < 0.05);
        assert!((f_max_khz(5.0, f64::INFINITY) - 17.4).abs() < 0.05);
        assert!((f_max_khz(4.0, 25.0) - 16.5).abs() < 0.05);
        assert!((f_max_khz(4.0, 10.0) - 12.1).abs() < 0.05);
        assert!((f_max_khz(20.0, 25.0) - 4.1).abs() < 0.05);
    }

    #[test]
    fn criterion5_matches_the_published_angle_vectors() {
        let at = |distance_m: f64| {
            max_splay_rad(SPEED_OF_SOUND_PAPER, 16_000.0, MOUTH, STEP, distance_m).to_degrees()
        };
        assert!((at(20.0) - 3.9).abs() < 0.05, "{}", at(20.0));
        assert!((at(10.0) - 2.3).abs() < 0.05, "{}", at(10.0));
    }

    #[test]
    fn criterion5_inverses_are_consistent_with_each_other() {
        // `d_min` et `f_max` sont deux lectures de la même équation : partir
        // d'un angle et d'une distance, puis revenir, doit boucler.
        let splay = 5f64.to_radians();
        let distance = 25.0;
        let f = max_frequency_hz(SPEED_OF_SOUND_PAPER, splay, MOUTH, STEP, distance).unwrap();
        let back = min_distance_m(SPEED_OF_SOUND_PAPER, f, splay, MOUTH, STEP).unwrap();
        assert!(
            (back - distance).abs() < 1e-6,
            "attendu {distance}, obtenu {back}"
        );

        let alpha_back = max_splay_rad(SPEED_OF_SOUND_PAPER, f, MOUTH, STEP, distance);
        assert!((alpha_back - splay).abs() < 1e-12);
    }

    #[test]
    fn a_splay_beyond_the_infinite_distance_limit_has_no_valid_distance() {
        // 30° à 16 kHz dépasse la limite même à l'infini : aucune distance ne
        // rattrape, la fonction doit le dire au lieu de rendre un nombre.
        assert!(min_distance_m(
            SPEED_OF_SOUND_PAPER,
            16_000.0,
            30f64.to_radians(),
            MOUTH,
            STEP
        )
        .is_none());
    }

    #[test]
    fn max_step_is_the_pitch_that_zeroes_the_max_splay() {
        let step = max_step_m(SPEED_OF_SOUND_PAPER, 16_000.0, MOUTH, 20.0);
        let splay = max_splay_rad(SPEED_OF_SOUND_PAPER, 16_000.0, MOUTH, step, 20.0);
        assert!(
            splay.abs() < 1e-12,
            "α_max devrait s'annuler, obtenu {splay}"
        );
    }

    #[test]
    fn side_lobe_attenuation_matches_the_published_table() {
        let db = |arf: f64| side_lobe_attenuation_db(arf).unwrap();
        assert!((db(0.80) - 12.0).abs() < 0.05);
        assert!((db(0.82) - 13.2).abs() < 0.05);
        assert!((db(0.84) - 14.4).abs() < 0.05);
        assert!((db(0.90) - 19.1).abs() < 0.05);
        assert!((db(0.76) - 10.0).abs() < 0.05);
        // Et l'inverse retombe sur ses pieds.
        assert!((arf_for_attenuation(db(0.84)) - 0.84).abs() < 1e-9);
        // Une ligne continue n'a plus de lobe de réseau à annoncer.
        assert!(side_lobe_attenuation_db(1.0).is_none());
    }

    #[test]
    fn axial_loss_matches_the_published_value() {
        assert!((axial_loss_db(0.84) + 1.5).abs() < 0.05);
    }

    #[test]
    fn wavefront_deviation_matches_the_published_values() {
        let mm = |f| max_wavefront_deviation_m(SPEED_OF_SOUND_DEFAULT, f) * 1000.0;
        assert!((mm(16_000.0) - 5.4).abs() < 0.05, "{}", mm(16_000.0));
        assert!((mm(20_000.0) - 4.3).abs() < 0.05, "{}", mm(20_000.0));
        // Inverse : la déviation admissible à 16 kHz reste isophase jusqu'à 16 kHz.
        let s = max_wavefront_deviation_m(SPEED_OF_SOUND_DEFAULT, 16_000.0);
        let f = isophase_frequency_limit(SPEED_OF_SOUND_DEFAULT, s).unwrap();
        assert!((f - 16_000.0).abs() < 1e-6);
    }

    #[test]
    fn grating_lobe_only_exists_above_the_half_wavelength_limit() {
        let step = 0.558;
        let f_c2 = frequency_limit_half_wavelength(SPEED_OF_SOUND_DEFAULT, step);
        // Juste en dessous : λ > STEP, aucun lobe de réseau.
        let below = wavelength_m(SPEED_OF_SOUND_DEFAULT, f_c2 * 0.5);
        assert!(grating_lobe_angle_rad(below, step).is_none());
        // Bien au-dessus : le lobe existe et rentre vers l'axe quand f monte.
        let high = wavelength_m(SPEED_OF_SOUND_DEFAULT, f_c2 * 4.0);
        let higher = wavelength_m(SPEED_OF_SOUND_DEFAULT, f_c2 * 8.0);
        let a = grating_lobe_angle_rad(high, step).unwrap();
        let b = grating_lobe_angle_rad(higher, step).unwrap();
        assert!(
            b < a,
            "le lobe doit se rapprocher de l'axe en montant en fréquence"
        );
    }

    #[test]
    fn near_field_boundary_is_shorter_in_the_fresnel_form() {
        let h = 3.35;
        let lambda = wavelength_m(SPEED_OF_SOUND_DEFAULT, 2_000.0);
        let exact = near_field_boundary_m(h, lambda).unwrap();
        let fresnel = near_field_boundary_fresnel_m(h, lambda);
        assert!(fresnel < exact, "Fresnel doit être la borne la plus proche");
        // λ ≥ H : plus de champ proche du tout.
        assert!(near_field_boundary_m(h, h * 1.01).is_none());
    }

    #[test]
    fn cca_geometry_holds_together() {
        let step = 0.558;
        let splay = 5f64.to_radians();
        let radius = curvature_radius_m(step, splay).unwrap();
        assert!((radius - step / splay).abs() < 1e-12);
        // La flèche croît avec l'angle, et un guide plus ouvert se raccorde à
        // un splay plus grand.
        let s_small = cca_sagitta_m(radius, splay);
        let s_big = cca_sagitta_m(curvature_radius_m(step, splay * 2.0).unwrap(), splay * 2.0);
        assert!(s_big > s_small);
        // Au centre de la bouche, aucun retard.
        assert!(guide_path_delay_m(radius, 0.0).abs() < 1e-12);
        assert!(guide_path_delay_m(radius, 0.2) > 0.0);
    }

    #[test]
    fn transition_splay_is_the_tangency_of_both_wavefronts() {
        // SA303 : un guide isophase qui rayonne encore 1,3° raccorde un guide de
        // 21,2° à 11,25°, pas aux 10,6° qu'aurait donnés θ/2.
        let splay = transition_splay_rad(1.3f64.to_radians(), 21.2f64.to_radians()).to_degrees();
        assert!((splay - 11.25).abs() < 1e-9, "obtenu {splay}");
        // Un guide isophase parfaitement plan retombe sur l'ancienne forme θ/2.
        let flat = transition_splay_rad(0.0, 20f64.to_radians()).to_degrees();
        assert!((flat - 10.0).abs() < 1e-9);
    }

    #[test]
    fn a_wavefront_sector_and_its_flatness_come_from_the_same_radius() {
        // Guide courbé SA303 : D identifié 385 mm sur un rayon de 1,04 m. Seul
        // leur rapport est bien déterminé, et il vaut 21,2°.
        let sector = wavefront_sector_rad(0.385, Some(1.04))
            .unwrap()
            .to_degrees();
        assert!((sector - 21.2).abs() < 0.05, "obtenu {sector}");

        // Guide isophase SA303 : bouche physique 464 mm, rayon 22,6 m. L'écart
        // au plan est millimétrique, donc la limite isophase sort de la bande
        // audio — c'est bien un front plan.
        let s = wavefront_flatness_deviation_m(0.464, Some(22.6)).unwrap();
        assert!((s * 1000.0 - 1.19).abs() < 0.01, "{} mm", s * 1000.0);
        let f = isophase_frequency_limit(SPEED_OF_SOUND_DEFAULT, s).unwrap();
        assert!(f > 20_000.0, "obtenu {f} Hz");

        // Pas de rayon = front plan : ni secteur, ni écart au plan.
        assert!(wavefront_sector_rad(0.464, None).is_none());
        assert!(wavefront_flatness_deviation_m(0.464, None).is_none());
    }

    #[test]
    fn the_overlap_notch_needs_an_actual_overlap() {
        // Guide de 21,2° exploité à 15° : 6,2° de recouvrement, premier creux
        // vers 5,7 kHz.
        let f = overlap_notch_frequency_hz(SPEED_OF_SOUND_DEFAULT, 0.5524, 6.2f64.to_radians())
            .unwrap();
        assert!((f - 5_740.0).abs() < 20.0, "obtenu {f} Hz");
        // Le creux monte quand le recouvrement se referme : à recouvrement nul
        // ou négatif (trou angulaire), il n'y a plus de zone commune du tout.
        let tighter =
            overlap_notch_frequency_hz(SPEED_OF_SOUND_DEFAULT, 0.5524, 3.0f64.to_radians())
                .unwrap();
        assert!(tighter > f);
        assert!(overlap_notch_frequency_hz(SPEED_OF_SOUND_DEFAULT, 0.5524, 0.0).is_none());
        assert!(overlap_notch_frequency_hz(SPEED_OF_SOUND_DEFAULT, 0.5524, -0.05).is_none());
    }

    /// La bouche acoustique est plus grande que la bouche physique (diffraction
    /// de bride) : elle donne donc une fréquence limite **plus basse**. Confondre
    /// les deux, c'est se croire meilleur qu'on n'est.
    #[test]
    fn a_larger_acoustic_mouth_lowers_the_criterion5_limit() {
        let step = 0.5524;
        let at = |mouth: f64| {
            max_frequency_hz(SPEED_OF_SOUND_DEFAULT, 5f64.to_radians(), mouth, step, 25.0).unwrap()
        };
        assert!(at(0.485) < at(0.464));
    }

    #[test]
    fn curved_line_loses_three_db_when_curvature_equals_the_step() {
        // α·d/STEP = 1 → −3 dB, valeur de référence de la spec.
        let step = 0.55;
        let db = curved_line_relative_level_db(step / 25.0, 25.0, step);
        assert!((db + 3.0).abs() < 0.02, "obtenu {db}");
    }
}

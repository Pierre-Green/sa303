//! Sandwich flancs/barre en cisaillement double (brief §6) : la goupille
//! d'assemblage travaille en cisaillement double entre les deux flancs, et
//! matit soit la barre mobile, soit les flancs eux-mêmes selon le mode le
//! plus contraignant — jamais une valeur unique, toujours les trois calculées
//! et le pire retenu.

use crate::settings::Settings;

/// Assemblage sandwich : deux flancs encadrant une barre mobile, goupille en cisaillement double.
#[derive(Clone, Copy, Debug)]
pub struct SandwichSpec {
    pub pin_diameter_mm: f64,
    pub flank_thickness_mm: f64,
    pub bar_thickness_mm: f64,
    pub pin_ultimate_mpa: f64,
    pub plate_ultimate_mpa: f64,
    /// Fraction de section résistante restante (alésage du plongeur).
    pub pin_net_section: f64,
    pub safety_factor: f64,
}

impl SandwichSpec {
    pub fn from_settings(s: &Settings) -> Self {
        SandwichSpec {
            pin_diameter_mm: s.pin.diameter,
            flank_thickness_mm: s.plate.flank_thickness,
            bar_thickness_mm: s.plate.bar_thickness,
            pin_ultimate_mpa: s.pin.ultimate,
            plate_ultimate_mpa: s.plate.ultimate,
            pin_net_section: s.pin.net_section,
            safety_factor: s.safety_factor,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UtilizationBreakdown {
    pub shear: f64,
    pub bar_bearing: f64,
    pub flank_bearing: f64,
}

impl UtilizationBreakdown {
    pub fn max(&self) -> f64 {
        self.shear.max(self.bar_bearing).max(self.flank_bearing)
    }
}

pub fn utilization_breakdown(force_n: f64, spec: &SandwichSpec) -> UtilizationBreakdown {
    let area = std::f64::consts::PI * spec.pin_diameter_mm.powi(2) / 4.0 * spec.pin_net_section;
    let shear_admissible = 0.6 * spec.pin_ultimate_mpa / spec.safety_factor;
    let plate_admissible = spec.plate_ultimate_mpa / spec.safety_factor;

    let shear = (force_n / (2.0 * area)) / shear_admissible;
    let bar_bearing = (force_n / (spec.pin_diameter_mm * spec.bar_thickness_mm)) / plate_admissible;
    let flank_bearing =
        (force_n / 2.0 / (spec.pin_diameter_mm * spec.flank_thickness_mm)) / plate_admissible;

    UtilizationBreakdown {
        shear,
        bar_bearing,
        flank_bearing,
    }
}

pub fn utilization(force_n: f64, spec: &SandwichSpec) -> f64 {
    utilization_breakdown(force_n, spec).max()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sandwich par défaut du brief §6 : goupille Ø12 17-4 PH H1075, flancs 4 mm,
    /// barre 10 mm, Rm goupille 1000 MPa, Rm tôle S355 510 MPa, coefficient 4:1.
    fn default_spec() -> SandwichSpec {
        SandwichSpec {
            pin_diameter_mm: 12.0,
            flank_thickness_mm: 4.0,
            bar_thickness_mm: 10.0,
            pin_ultimate_mpa: 1000.0,
            plate_ultimate_mpa: 510.0,
            pin_net_section: 0.86,
            safety_factor: 4.0,
        }
    }

    #[test]
    fn breakdown_matches_hand_computed_ratios_at_1000n() {
        let b = utilization_breakdown(1000.0, &default_spec());
        // Matage de barre et de flanc sont des fractions exactes, calculables à la main.
        assert!((b.bar_bearing - 1000.0 / 15300.0).abs() < 1e-9);
        assert!((b.flank_bearing - 500.0 / 6120.0).abs() < 1e-9);
        // Cisaillement double : recalculé indépendamment de la fonction testée.
        let area = std::f64::consts::PI * 12f64.powi(2) / 4.0 * 0.86;
        let expected_shear = (1000.0 / (2.0 * area)) / (0.6 * 1000.0 / 4.0);
        assert!((b.shear - expected_shear).abs() < 1e-9);
    }

    #[test]
    fn max_is_the_governing_mode() {
        // Avec la géométrie par défaut, le matage de flanc (4 mm, effort réparti sur
        // un seul flanc côté trou) est toujours le plus contraignant des trois.
        let b = utilization_breakdown(1000.0, &default_spec());
        assert!(b.flank_bearing > b.bar_bearing);
        assert!(b.flank_bearing > b.shear);
        assert_eq!(utilization(1000.0, &default_spec()), b.flank_bearing);
    }

    #[test]
    fn utilization_scales_linearly_with_force() {
        let s = default_spec();
        let u1 = utilization(1000.0, &s);
        let u2 = utilization(3000.0, &s);
        assert!((u2 - 3.0 * u1).abs() < 1e-9);
    }

    #[test]
    fn safety_factor_4_1_matches_100_percent_at_reduced_admissible() {
        // Coefficient de sécurité 4:1 sur la structure (brief §6) : au seuil F tel
        // que le matage de flanc vaille exactement Rm_tole/4, le taux doit être 1.0.
        let s = default_spec();
        let f_at_100pct =
            2.0 * s.pin_diameter_mm * s.flank_thickness_mm * s.plate_ultimate_mpa / s.safety_factor;
        let u = utilization(f_at_100pct, &s);
        assert!((u - 1.0).abs() < 1e-9);
    }

    #[test]
    fn utilization_increases_with_a_tighter_safety_factor() {
        // Un coefficient plus élevé (plus conservateur, ex. le 10:1 des accessoires
        // certifiés vs le 4:1 de la structure) réduit l'admissible et augmente donc
        // le taux de travail pour un même effort.
        let mut s = default_spec();
        let u4 = utilization(1000.0, &s);
        s.safety_factor = 8.0;
        let u8 = utilization(1000.0, &s);
        assert!(u8 > u4);
        assert!((u8 - 2.0 * u4).abs() < 1e-9);
    }
}

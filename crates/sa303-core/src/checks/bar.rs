//! Barre arrière : flexion composée balayée en continu, et arrachement de bord
//! aux perçages.
//!
//! **Pourquoi un balayage et non une liste de sections.** Tant que la barre
//! avait une largeur constante par morceaux, regarder les trous et la marche
//! suffisait : entre deux discontinuités, le moment croît mais la section ne
//! bouge pas, donc le maximum de contrainte tombe forcément sur une des deux
//! bornes. Le profil v3 a une **pente** de 40 à 70 mm entre les abscisses 30 et
//! 100 : la largeur et le moment y varient tous les deux, et rien ne garantit
//! que leur rapport culmine à une extrémité. La contrainte est donc calculée
//! millimètre par millimètre, et le maximum sort du balayage.
//!
//! **Le diagramme de moment, en deux morceaux.**
//!
//! - **Côté paire** (verrou → ancrage) : la barre ne voit que l'effort du pion
//!   de verrou. Le moment part de zéro au verrou — il ne reste rien derrière
//!   lui — et croît jusqu'à l'ancrage.
//! - **Côté console** (ancrage → couronne) : elle ne voit que l'effort de
//!   couronne. Le moment part de zéro à la couronne — articulation simple — et
//!   croît en redescendant vers l'ancrage.
//!
//! Les deux expressions doivent donner **la même valeur à l'ancrage**. Ce n'est
//! pas une commodité : c'est l'équilibre de la barre, donc le contrôle que la
//! paire a été correctement résolue en amont. Un écart y signalerait une erreur
//! dans la répartition élastique, pas dans ce module.

use crate::speaker::{BarHole, CrownRow, RearBar};
use crate::vector::Vec2;

/// Pas de balayage. 1 mm sur 430 mm de barre : assez fin pour que le maximum ne
/// se cache pas entre deux points (la contrainte varie de moins de 1 % sur un
/// millimètre), assez grossier pour rester gratuit.
pub const SWEEP_STEP_MM: f64 = 1.0;

/// Pas d'échantillonnage du profil rendu à l'export. Plus large que le balayage :
/// le profil sert à tracer une courbe, pas à trouver un maximum.
pub const PROFILE_SAMPLE_MM: f64 = 5.0;

/// Tolérance de raccord des deux branches du diagramme à l'ancrage.
pub const MOMENT_CONTINUITY_TOLERANCE_NMM: f64 = 1e-6;

/// Une section de la barre, à une abscisse donnée.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarSectionCheck {
    /// Abscisse depuis le petit bout (côté verrou), mm.
    pub at_mm: f64,
    /// Nom du trou si la section est percée, sinon `None`.
    pub hole: Option<String>,
    pub width_mm: f64,
    pub drilled: bool,
    /// Décalage du perçage par rapport à l'axe des trous, mm.
    pub hole_offset_mm: f64,
    /// Moment de flexion, N·mm (par flanc).
    pub moment_nmm: f64,
    /// Effort normal, N (par flanc).
    pub axial_n: f64,
    pub section_modulus_mm3: f64,
    pub area_mm2: f64,
    pub stress_mpa: f64,
    /// Rapport à `R_m / sf` — le critère, cohérent avec `checks::sandwich`.
    pub utilization: f64,
    /// Rapport à `f_y`, indicatif : ce n'est pas le critère retenu.
    pub yield_ratio: f64,
}

/// Un point du profil σ(a), pour tracer la courbe à l'export.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarProfilePoint {
    pub at_mm: f64,
    pub width_mm: f64,
    pub moment_nmm: f64,
    pub stress_mpa: f64,
    pub utilization: f64,
}

/// Arrachement de bord à un perçage : la goupille chasse devant elle le bloc de
/// matière qui la sépare du bord libre, cisaillé sur ses deux flancs.
///
/// Distinct du matage (`checks::sandwich`), qui est un écrasement local et ne
/// regarde pas où est le bord. Ici c'est la **distance au bord** qui gouverne, et
/// seule compte la composante dirigée vers ce bord : un effort parallèle au bord
/// ne chasse rien devant lui.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarTearOutCheck {
    pub location: String,
    pub edge_distance_mm: f64,
    pub force_toward_edge_n: f64,
    pub shear_area_mm2: f64,
    pub stress_mpa: f64,
    pub utilization: f64,
}

/// Écart géométrique relevé sur la barre montée. Signalé, jamais tu : une barre
/// qui dépasse derrière le caisson ne se manifeste qu'au moment où elle touche.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarGeometryWarning {
    pub what: String,
    pub expected_mm: f64,
    pub actual_mm: f64,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarCheck {
    /// La section la plus sollicitée du balayage.
    pub critical: BarSectionCheck,
    /// σ(a) échantillonné, pour tracer.
    pub profile: Vec<BarProfilePoint>,
    pub tear_out: Vec<BarTearOutCheck>,
    pub worst_tear_out: usize,
    pub warnings: Vec<BarGeometryWarning>,
    /// Écart entre les deux branches du diagramme à l'ancrage. Doit être nul :
    /// c'est le contrôle que la paire est bien résolue.
    pub moment_continuity_nmm: f64,
}

impl BarCheck {
    pub fn worst_tear_out(&self) -> &BarTearOutCheck {
        &self.tear_out[self.worst_tear_out]
    }

    /// Le pire des deux modes : flexion composée et arrachement de bord.
    pub fn utilization(&self) -> f64 {
        self.critical
            .utilization
            .max(self.worst_tear_out().utilization)
    }
}

/// Efforts appliqués à la barre, dans son propre repère, **avec leur point
/// d'application**. Tout est par flanc.
///
/// Les points viennent de la géométrie réellement calculée sur le caisson, pas
/// de la cotation déclarée de la barre. Les deux ne coïncident qu'à la
/// tolérance d'ajustement près (0,05 mm), et mélanger les sources laisserait un
/// résidu de moment de l'ordre du N·mm à l'ancrage — pas une faute de physique,
/// mais assez pour rendre le contrôle de raccord inexploitable.
pub struct BarLoads {
    /// Effort de couronne, appliqué au trou `up660` ou `up680` selon la parité.
    pub crown: Vec2,
    pub crown_at: Vec2,
    /// Effort que le pion de verrou applique à la barre.
    pub latch: Vec2,
    pub latch_at: Vec2,
    pub anchor_at: Vec2,
}

/// Vérifie la barre par balayage continu.
///
/// `safety_factor` est celui de `checks::sandwich`, volontairement : deux
/// coefficients différents sur la même jonction donneraient deux taux de travail
/// incomparables sur la même ligne de tableau.
pub fn check_bar(
    bar: &RearBar,
    crown_row: CrownRow,
    loads: &BarLoads,
    safety_factor: f64,
    rear_face_x: Option<f64>,
    bar_rear_edge_max_x: Option<f64>,
) -> BarCheck {
    let admissible = bar.ultimate_strength / safety_factor;
    let crown = loads.crown_at;
    let anchor = loads.anchor_at;
    let latch = loads.latch_at;

    // Moment et effort normal à une abscisse, selon le côté de l'ancrage où
    // elle tombe. Produit vectoriel complet et non `|V| × bras` : `up660` est
    // déporté de 19,4 mm, et ce bras transversal compte.
    let at = |a: f64| -> (f64, f64) {
        let p = Vec2::new(a, 0.0);
        if a <= anchor.x {
            // Côté paire : seul le verrou charge ce tronçon.
            ((p - latch).cross(loads.latch), loads.latch.x)
        } else {
            // Côté console : seule la couronne charge ce tronçon.
            //
            // Signe opposé : un moment de flexion lu depuis la gauche d'une
            // coupe et depuis sa droite sortent en sens contraires du produit
            // vectoriel brut. Sans cette inversion le diagramme sauterait de
            // signe à l'ancrage — sans que la contrainte s'en aperçoive, elle
            // qui prend la valeur absolue, mais le contrôle de raccord, lui,
            // ne verrait plus rien.
            (-(p - crown).cross(loads.crown), -loads.crown.x)
        }
    };

    // Les quatre trous, pour savoir si une abscisse balayée est percée.
    let holes: [(&str, BarHole); 4] = [
        ("verrou", bar.holes.latch),
        ("ancrage", bar.holes.anchor),
        ("couronne 660", bar.holes.up660),
        ("couronne 680", bar.holes.up680),
    ];
    let hole_at = |a: f64| holes.iter().find(|(_, h)| (h.along() - a).abs() < 0.5);

    let section = |a: f64| -> BarSectionCheck {
        let (m, n) = at(a);
        let width = bar.width_at(a);
        let (name, drilled, offset, w, area) = match hole_at(a) {
            Some((name, h)) => (
                Some((*name).to_string()),
                true,
                h.lateral(),
                bar.net_modulus_at(*h),
                bar.net_area_at(a),
            ),
            None => (None, false, 0.0, bar.modulus_at(a), bar.area_at(a)),
        };
        let stress = m.abs() / w + n.abs() / area;
        BarSectionCheck {
            at_mm: a,
            hole: name,
            width_mm: width,
            drilled,
            hole_offset_mm: offset,
            moment_nmm: m.abs(),
            axial_n: n,
            section_modulus_mm3: w,
            area_mm2: area,
            stress_mpa: stress,
            utilization: stress / admissible,
            yield_ratio: stress / bar.yield_strength,
        }
    };

    // --- Balayage -----------------------------------------------------------
    let start = latch.x;
    let end = crown.x;
    let mut a = start;
    let mut critical = section(start);
    let mut profile = Vec::new();
    let mut next_sample = start;
    while a <= end + 1e-9 {
        let s = section(a);
        if s.utilization > critical.utilization {
            critical = s.clone();
        }
        if a >= next_sample - 1e-9 {
            profile.push(BarProfilePoint {
                at_mm: s.at_mm,
                width_mm: s.width_mm,
                moment_nmm: s.moment_nmm,
                stress_mpa: s.stress_mpa,
                utilization: s.utilization,
            });
            next_sample += PROFILE_SAMPLE_MM;
        }
        a += SWEEP_STEP_MM;
    }
    // Les quatre trous sont regardés explicitement en plus du pas de 1 mm :
    // leur abscisse n'est pas ronde, et c'est là que la section est nette.
    for (_, h) in holes {
        if h.along() >= start - 1e-9 && h.along() <= end + 1e-9 {
            let s = section(h.along());
            if s.utilization > critical.utilization {
                critical = s;
            }
        }
    }

    // --- Raccord des deux branches à l'ancrage ------------------------------
    let m_from_pair = (anchor - latch).cross(loads.latch);
    let m_from_cantilever = -(anchor - crown).cross(loads.crown);
    let moment_continuity_nmm = m_from_pair - m_from_cantilever;

    // --- Arrachement de bord ------------------------------------------------
    let shear_admissible = 0.6 * bar.ultimate_strength / safety_factor;
    let tear_out: Vec<BarTearOutCheck> = [
        ("verrou", bar.holes.latch, loads.latch),
        ("ancrage", bar.holes.anchor, -(loads.latch + loads.crown)),
        ("couronne", bar.crown_hole(crown_row), loads.crown),
    ]
    .into_iter()
    .map(|(name, hole, force)| {
        let front = bar.front_edge_at(hole.along());
        let back = bar.rear_edge_offset;
        let e2 = bar.edge_distance_at(hole);
        let toward_front = (front - hole.lateral()) <= (hole.lateral() + back);
        let component = if toward_front { force.y } else { -force.y };
        let area = 2.0 * (e2 - bar.hole_diameter / 2.0).max(0.0) * bar.thickness;
        let stress = if area > 0.0 {
            component.abs() / area
        } else {
            f64::INFINITY
        };
        BarTearOutCheck {
            location: name.to_string(),
            edge_distance_mm: e2,
            force_toward_edge_n: component.abs(),
            shear_area_mm2: area,
            stress_mpa: stress,
            utilization: stress / shear_admissible,
        }
    })
    .collect();
    let worst_tear_out = tear_out
        .iter()
        .enumerate()
        .max_by(|(_, x), (_, y)| x.utilization.total_cmp(&y.utilization))
        .map(|(i, _)| i)
        .unwrap_or(0);

    // --- Contrôles géométriques ---------------------------------------------
    let mut warnings = Vec::new();
    let min_edge = 1.2 * bar.hole_diameter;
    for (name, h) in holes {
        // e₂ : perpendiculairement, vers le bord le plus proche.
        let e2 = bar.edge_distance_at(h);
        if e2 < min_edge - 1e-9 {
            warnings.push(BarGeometryWarning {
                what: format!("e₂ {name}"),
                expected_mm: min_edge,
                actual_mm: e2,
            });
        }
    }
    // e₁ : le long de l'axe, depuis chaque bout.
    for (name, e1) in [
        ("e₁ bout verrou", bar.holes.latch.along()),
        ("e₁ bout couronne", bar.length - bar.holes.up680.along()),
    ] {
        if e1 < min_edge - 1e-9 {
            warnings.push(BarGeometryWarning {
                what: name.to_string(),
                expected_mm: min_edge,
                actual_mm: e1,
            });
        }
    }
    // Le bord arrière de la barre doit rester devant la face arrière du
    // caisson. Le maximum est au **petit bout** et non à l'ancrage : le bord
    // arrière est parallèle à l'axe de barre, qui s'incline vers l'avant en
    // montant, donc son abscisse recule à mesure qu'on monte.
    if let (Some(face), Some(edge)) = (rear_face_x, bar_rear_edge_max_x) {
        if edge > face {
            warnings.push(BarGeometryWarning {
                what: "bord arrière de barre au-delà de la face arrière".into(),
                expected_mm: face,
                actual_mm: edge,
            });
        }
    }

    BarCheck {
        critical,
        profile,
        tear_out,
        worst_tear_out,
        warnings,
        moment_continuity_nmm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speaker::BarHoles;

    fn sa303_bar() -> RearBar {
        RearBar {
            thickness: 10.0,
            length: 460.014,
            width_profile: vec![[0.0, 40.0], [30.0, 40.0], [100.0, 70.0], [460.014, 70.0]],
            rear_edge_offset: 20.0,
            hole_diameter: 12.08,
            holes: BarHoles {
                latch: BarHole([16.0, 0.0]),
                anchor: BarHole([116.0, 0.0]),
                up660: BarHole([440.175, 19.406]),
                up680: BarHole([445.014, 0.0]),
            },
            mass_kg: 2.32,
            yield_strength: 355.0,
            ultimate_strength: 510.0,
        }
    }

    /// Le profil de largeur, aux abscisses de contrôle du dessin.
    #[test]
    fn the_width_profile_interpolates_the_slope() {
        let bar = sa303_bar();
        for (x, expected) in [
            (16.0, 40.0),
            (30.0, 40.0),
            (50.0, 48.571_428_6),
            (70.0, 57.142_857_1),
            (90.0, 65.714_285_7),
            (100.0, 70.0),
            (300.0, 70.0),
            (460.0, 70.0),
        ] {
            let w = bar.width_at(x);
            assert!(
                (w - expected).abs() < 1e-6,
                "w({x}) = {w}, attendu {expected}"
            );
        }
        // Le bord avant suit, le bord arrière ne bouge pas.
        assert!((bar.front_edge_at(16.0) - 20.0).abs() < 1e-9);
        assert!((bar.front_edge_at(100.0) - 50.0).abs() < 1e-9);
    }

    /// Là où la section est **symétrique** autour de l'axe des trous — largeur
    /// 40, donc 20 de chaque côté — le module net doit retomber sur la formule
    /// de la fente sur fibre neutre. C'est le cas au verrou.
    #[test]
    fn the_net_modulus_agrees_with_the_centred_formula_on_a_symmetric_section() {
        let bar = sa303_bar();
        let h = bar.holes.latch;
        let w = bar.width_at(h.along());
        assert_eq!(w, 40.0);
        let centred = bar.thickness * (w.powi(3) - bar.hole_diameter.powi(3)) / (6.0 * w);
        let got = bar.net_modulus_at(h);
        assert!((got - centred).abs() < 1e-9, "{got} contre {centred}");
    }

    /// Mais dès que la barre s'élargit, elle le fait **d'un seul côté** : l'axe
    /// des trous reste à 20 mm du bord arrière, donc il s'écarte de la fibre
    /// neutre à mesure que le bord avant s'éloigne.
    ///
    /// À l'ancrage — largeur 70 — le trou est à 18,1 mm du centroïde. La
    /// formule centrée y donnerait 8 125 mm³ au lieu de 6 597 : elle
    /// **sous-estimerait la contrainte de 19 %** à la section critique. C'est
    /// la raison d'être du calcul excentré, et elle n'existait pas sur le
    /// profil précédent où les trous étaient au milieu d'une largeur de 40.
    #[test]
    fn the_widening_puts_the_holes_off_the_neutral_axis() {
        let bar = sa303_bar();
        let h = bar.holes.anchor;
        let w = bar.width_at(h.along());
        assert_eq!(w, 70.0);
        let centred = bar.thickness * (w.powi(3) - bar.hole_diameter.powi(3)) / (6.0 * w);
        let exact = bar.net_modulus_at(h);
        assert!((exact - 6596.5).abs() < 0.5, "W_net = {exact}");
        assert!((centred - 8124.7).abs() < 0.5, "centré = {centred}");
        assert!(
            exact < centred * 0.85,
            "l'excentration doit coûter nettement : {exact} contre {centred}"
        );
    }
}

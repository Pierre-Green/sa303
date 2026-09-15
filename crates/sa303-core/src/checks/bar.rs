//! Barre arrière : flexion composée, et arrachement de bord aux perçages.
//!
//! C'est le mode que le modèle à deux forces ne pouvait pas voir : tant que la
//! barre était supposée bi-articulée, elle ne portait qu'un effort normal et sa
//! section n'avait qu'à passer la traction. Goupillée en deux points sur le
//! caisson du bas, elle fléchit.
//!
//! **Diagramme de moment.** La barre n'a que deux liaisons : la goupille de
//! couronne (articulation, moment nul) et la paire ancrage/verrou. L'ordre le
//! long de la barre est verrou, ancrage, épaulement, couronne. Entre la couronne
//! et l'ancrage, la barre ne voit qu'un seul effort : le moment y croît depuis
//! zéro. Au-delà de l'ancrage, la réaction de la paire le fait redescendre, et
//! il est **nul au verrou** — il ne reste derrière lui que 14,5 mm de barre où
//! rien ne s'applique.
//!
//! Le maximum est donc à l'**ancrage**. C'est l'inverse de la géométrie
//! précédente, où le verrou était le premier pion depuis la couronne : les deux
//! ont échangé leur rang.
//!
//! **Les sections regardées.** L'ancrage (étroite, percée, trou centré, moment
//! maximal), l'épaulement (étroite, brute), et les deux trous de couronne
//! (partie élargie, percés, moment nul — seule la section nette sous traction et
//! l'arrachement les concernent). Le verrou est vérifié aussi, pour que son
//! moment nul soit visible plutôt que sous-entendu.

use crate::speaker::{BarHole, RearBar};
use crate::vector::Vec2;

/// Une section vérifiée, avec ce qui la sollicite et ce qu'elle vaut.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarSectionCheck {
    /// Nom lisible de la section, pour dire *où* ça travaille.
    pub location: String,
    /// Abscisse depuis le petit bout (côté verrou), mm.
    pub at_mm: f64,
    /// Largeur de la section. Rendue explicitement pour qu'un balayage
    /// 40 / 50 / 60 / 70 se lise dans le résultat sans recouper le modèle.
    pub width_mm: f64,
    pub drilled: bool,
    /// Décalage du perçage par rapport à l'axe de la partie étroite, mm.
    pub hole_offset_mm: f64,
    /// Moment de flexion à cette section, N·mm (par flanc).
    pub moment_nmm: f64,
    /// Effort normal, N (par flanc). Constant le long de la barre.
    pub axial_n: f64,
    /// Module de flexion net retenu, mm³.
    pub section_modulus_mm3: f64,
    /// Contrainte de flexion composée, MPa.
    pub stress_mpa: f64,
    /// Rapport à `R_m / sf` — le critère, cohérent avec `checks::sandwich`.
    pub utilization: f64,
    /// Rapport à `f_y`, à titre indicatif : au-delà de 1, la barre a plastifié
    /// même si elle est loin de rompre. Ce n'est pas le critère retenu.
    pub yield_ratio: f64,
}

/// Arrachement de bord à un perçage : la goupille chasse devant elle le bloc de
/// matière qui la sépare du bord libre, cisaillé sur ses deux flancs.
///
/// Distinct du matage (`checks::sandwich`), qui est un écrasement local et ne
/// regarde pas où est le bord. Ici c'est la **distance au bord** qui gouverne,
/// et seule compte la composante de l'effort qui pousse vers ce bord : un effort
/// parallèle au bord ne chasse rien devant lui.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarTearOutCheck {
    pub location: String,
    /// Distance du centre du trou au bord le plus proche, mm.
    pub edge_distance_mm: f64,
    /// Composante de l'effort dirigée vers ce bord, N (par flanc).
    pub force_toward_edge_n: f64,
    /// Aire cisaillée : deux plans de `(e₂ − d₀/2) × t`.
    pub shear_area_mm2: f64,
    pub stress_mpa: f64,
    pub utilization: f64,
}

/// Résultat de la vérification : toutes les sections, l'arrachement de chaque
/// trou, et la pire des deux familles.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarCheck {
    pub sections: Vec<BarSectionCheck>,
    /// Index de la section la plus sollicitée dans `sections`.
    pub worst: usize,
    pub tear_out: Vec<BarTearOutCheck>,
    /// Index du trou le plus exposé à l'arrachement.
    pub worst_tear_out: usize,
    /// Largeur de la section critique. C'est le paramètre de dimensionnement de
    /// la barre : le rendre ici permet de balayer 40 / 50 / 60 / 70 et de lire
    /// dans le résultat ce qui a servi, sans recouper le modèle.
    pub critical_width_mm: f64,
}

impl BarCheck {
    pub fn worst_section(&self) -> &BarSectionCheck {
        &self.sections[self.worst]
    }

    pub fn worst_tear_out(&self) -> &BarTearOutCheck {
        &self.tear_out[self.worst_tear_out]
    }

    /// Le pire des deux modes : flexion composée et arrachement de bord.
    pub fn utilization(&self) -> f64 {
        self.worst_section()
            .utilization
            .max(self.worst_tear_out().utilization)
    }
}

/// Vérifie la barre sous l'effort de couronne déjà décomposé dans son repère.
///
/// `shear_n` et `axial_n` sont **par flanc**, comme tout ce que rend
/// `compute_joint` : la barre est elle-même doublée, un exemplaire par flanc.
///
/// `safety_factor` est celui de `checks::sandwich`, volontairement : deux
/// coefficients différents sur la même jonction donneraient deux taux de
/// travail incomparables sur la même ligne de tableau.
pub fn check_bar(
    bar: &RearBar,
    splay_deg: f64,
    shear_n: f64,
    axial_n: f64,
    safety_factor: f64,
) -> BarCheck {
    let admissible = bar.ultimate_strength / safety_factor;
    let crown = bar.crown_hole(splay_deg);
    let anchor = bar.holes.anchor;
    let latch = bar.holes.latch;

    // Effort vu par la barre, repère barre. `axial_n` est compté positif en
    // compression, donc dirigé vers les abscisses décroissantes : le remettre
    // sur l'axe demande le signe opposé.
    let f = Vec2::new(-axial_n, shear_n);

    // Moment exact en une section, vu depuis le côté couronne. Valable entre la
    // couronne et l'ancrage : c'est le seul tronçon où la barre ne voit qu'un
    // effort. Produit vectoriel complet et non `|V| × abscisse` — sur les splays
    // impairs la couronne est déportée de 19,4 mm, et ce bras-là compte.
    let moment_from_crown = |p: Vec2| (p - crown.as_vec()).cross(f);

    let mut sections = Vec::with_capacity(5);
    let mut push = |location: &str, hole: Option<BarHole>, at: f64, moment: f64| {
        let width = bar.width_at(at);
        let (drilled, offset, w_net, a_net) = match hole {
            Some(h) if h.lateral() == 0.0 => (
                true,
                0.0,
                bar.section_modulus_at(at, true),
                bar.net_area_at(at, true),
            ),
            // Trou déporté : retirer de la matière hors de la fibre neutre
            // déplace le centroïde, donc les deux fibres extrêmes n'ont plus le
            // même module. Le moment y est nul dans le modèle en vigueur, mais
            // la formule est là pour que déplacer un trou ne demande pas
            // d'écrire le calcul dans l'urgence.
            Some(h) => (
                true,
                h.lateral(),
                bar.eccentric_section_modulus(h),
                bar.net_area_at(at, true),
            ),
            None => (
                false,
                0.0,
                bar.section_modulus_at(at, false),
                bar.net_area_at(at, false),
            ),
        };
        let stress = moment.abs() / w_net + axial_n.abs() / a_net;
        sections.push(BarSectionCheck {
            location: location.to_string(),
            at_mm: at,
            width_mm: width,
            drilled,
            hole_offset_mm: offset,
            moment_nmm: moment.abs(),
            axial_n,
            section_modulus_mm3: w_net,
            stress_mpa: stress,
            utilization: stress / admissible,
            yield_ratio: stress / bar.yield_strength,
        });
    };

    // Ancrage : la section critique.
    push(
        "ancrage",
        Some(anchor),
        anchor.along(),
        moment_from_crown(anchor.as_vec()),
    );
    // Épaulement : encore dans la largeur étroite, non percé. Le moment y est
    // pris sur le tronçon couronne-ancrage, donc en extrapolant depuis la
    // couronne — ce qui le majore, et c'est le sens conservatif.
    if bar.step_position > anchor.along() && bar.step_position < crown.along() {
        let p = Vec2::new(bar.step_position, 0.0);
        push("épaulement", None, bar.step_position, moment_from_crown(p));
    }
    // Verrou : moment nul, seule la traction le traverse. Vérifié quand même
    // pour que le zéro soit visible plutôt que sous-entendu.
    push("verrou", Some(latch), latch.along(), 0.0);
    // Trous de couronne : articulation, donc moment nul. Ils ne sont là que pour
    // la section nette sous traction et, surtout, pour l'arrachement.
    push(
        "couronne 680",
        Some(bar.holes.up680),
        bar.holes.up680.along(),
        0.0,
    );
    push(
        "couronne 660",
        Some(bar.holes.up660),
        bar.holes.up660.along(),
        0.0,
    );

    let worst = sections
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.utilization.total_cmp(&b.utilization))
        .map(|(i, _)| i)
        .unwrap_or(0);

    // --- Arrachement de bord ------------------------------------------------
    //
    // Deux plans de cisaillement de `(e₂ − d₀/2) × t`, comparés à `0,6·Rm/sf`.
    //
    // Les efforts sur les pions de la paire ne sont pas connus ici — ils sortent
    // de `compute_joint`, qui a la répartition élastique. On prend donc la
    // moitié de la résultante à chaque pion, soit la part directe : c'est une
    // minoration sur le pion chargé par le couple, et le taux exact est rendu à
    // côté par `utilization_anchor` / `utilization_latch`. Ce contrôle-ci
    // répond à une autre question : la barre a-t-elle assez de matière derrière
    // ses trous.
    let shear_admissible = 0.6 * bar.ultimate_strength / safety_factor;
    let tear_out: Vec<BarTearOutCheck> = [
        ("ancrage", anchor, f * 0.5),
        ("verrou", latch, f * 0.5),
        ("couronne", crown, f),
    ]
    .into_iter()
    .map(|(name, hole, force)| {
        let e2 = bar.edge_distance_at(hole);
        // Le bord le plus proche est devant ou derrière selon le côté où le
        // trou est décalé ; la composante utile est celle qui va vers lui.
        let back = bar.narrow_width / 2.0;
        let front = if hole.along() >= bar.step_position {
            bar.wide_width - bar.narrow_width / 2.0
        } else {
            bar.narrow_width / 2.0
        };
        let toward_front = (front - hole.lateral()) <= (hole.lateral() + back);
        let component = if toward_front { force.y } else { -force.y };
        let toward_edge = component.abs();
        let area = 2.0 * (e2 - bar.hole_diameter / 2.0).max(0.0) * bar.thickness;
        let stress = if area > 0.0 {
            toward_edge / area
        } else {
            f64::INFINITY
        };
        BarTearOutCheck {
            location: name.to_string(),
            edge_distance_mm: e2,
            force_toward_edge_n: toward_edge,
            shear_area_mm2: area,
            stress_mpa: stress,
            utilization: stress / shear_admissible,
        }
    })
    .collect();

    let worst_tear_out = tear_out
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.utilization.total_cmp(&b.utilization))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let critical_width_mm = sections[worst].width_mm;
    BarCheck {
        sections,
        worst,
        tear_out,
        worst_tear_out,
        critical_width_mm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speaker::BarHoles;

    fn sa303_bar() -> RearBar {
        RearBar {
            thickness: 10.0,
            length: 458.514,
            narrow_width: 40.0,
            wide_width: 55.0,
            wide_length: 78.514,
            step_position: 380.0,
            hole_diameter: 12.08,
            holes: BarHoles {
                latch: BarHole([14.5, 0.0]),
                anchor: BarHole([114.5, 0.0]),
                up660: BarHole([438.675, 19.406]),
                up680: BarHole([443.514, 0.0]),
            },
            yield_strength: 355.0,
            ultimate_strength: 510.0,
        }
    }

    fn section<'a>(check: &'a BarCheck, name: &str) -> &'a BarSectionCheck {
        check
            .sections
            .iter()
            .find(|s| s.location == name)
            .unwrap_or_else(|| panic!("section {name} absente"))
    }

    /// Le module de flexion net traite le trou centré comme une fente sur la
    /// fibre neutre : `t(w³ − d0³)/(6w)`. Recoupé à la main : 325 N·m sur la
    /// section 40×10 percée Ø12,08 donnent 125,3 MPa. La formule
    /// `t(w − d0)²/6`, qui vaudrait pour un trou en fibre extrême, en donnerait
    /// 249 — soit un facteur 2 sur le dimensionnement de la barre.
    #[test]
    fn the_net_section_modulus_matches_the_hand_check() {
        let bar = sa303_bar();
        let w = bar.section_modulus_at(bar.holes.anchor.along(), true);
        assert!((w - 2593.217).abs() < 1e-3, "W_net = {w}");
        assert!((325_000.0 / w - 125.33).abs() < 0.01);
    }

    /// La section critique est l'ancrage, et le moment est nul au verrou comme
    /// aux trous de couronne : c'est l'inversion par rapport à la géométrie
    /// précédente, où le verrou était le premier pion depuis la couronne.
    #[test]
    fn the_anchor_is_the_critical_section_and_the_latch_carries_no_moment() {
        let check = check_bar(&sa303_bar(), 0.0, 1000.0, 0.0, 4.0);
        assert_eq!(section(&check, "verrou").moment_nmm, 0.0);
        assert_eq!(section(&check, "couronne 680").moment_nmm, 0.0);
        assert_eq!(section(&check, "couronne 660").moment_nmm, 0.0);
        assert_eq!(check.worst_section().location, "ancrage");
        // Bras ancrage -> couronne extérieure = 329,014 mm.
        assert!((section(&check, "ancrage").moment_nmm - 329_014.0).abs() < 1.0);
    }

    /// Loi d'échelle : la contrainte est linéaire en effort. Garde-fou contre
    /// une unité qui se perdrait (N·m contre N·mm) — l'erreur donnerait un
    /// facteur 1000, invisible sur un ratio.
    #[test]
    fn a_kilonewton_of_shear_gives_the_hand_computed_stress() {
        let bar = sa303_bar();
        let check = check_bar(&bar, 0.0, 1000.0, 0.0, 4.0);
        let anchor = check.worst_section();
        let expected = 329_014.0 / bar.section_modulus_at(bar.holes.anchor.along(), true);
        assert!(
            (anchor.stress_mpa - expected).abs() < 0.01,
            "{} contre {expected}",
            anchor.stress_mpa
        );
        // ~127 MPa sur la section 40×10 percée, pour 1 kN de transverse.
        assert!(
            (anchor.stress_mpa - 126.9).abs() < 0.5,
            "{}",
            anchor.stress_mpa
        );
    }

    /// `up660` est déporté : son module net doit être celui d'une section à trou
    /// excentré, plus faible que la formule centrée qui suppose le trou sur la
    /// fibre neutre.
    #[test]
    fn the_offset_crown_hole_uses_the_eccentric_section_modulus() {
        let check = check_bar(&sa303_bar(), 1.0, 1000.0, 0.0, 4.0);
        let up660 = section(&check, "couronne 660");
        assert_eq!(up660.hole_offset_mm, 19.406);
        assert_eq!(up660.width_mm, 55.0);
        let centred = sa303_bar().section_modulus_at(up660.at_mm, true);
        assert!(
            up660.section_modulus_mm3 < centred,
            "excentré {} devrait être sous le centré {centred}",
            up660.section_modulus_mm3
        );
    }

    /// À déport nul, la formule excentrée doit retomber sur la formule centrée :
    /// deux écritures du même cas, elles ne peuvent pas diverger.
    #[test]
    fn the_eccentric_formula_agrees_with_the_centred_one_at_zero_offset() {
        let bar = sa303_bar();
        let centred = bar.section_modulus_at(bar.holes.anchor.along(), true);
        let eccentric = bar.eccentric_section_modulus(bar.holes.anchor);
        assert!(
            (centred - eccentric).abs() < 1e-9,
            "{centred} contre {eccentric}"
        );
    }

    /// L'arrachement de bord est un mode distinct du matage : c'est la distance
    /// au bord qui gouverne, pas l'épaisseur seule.
    #[test]
    fn edge_tear_out_is_driven_by_the_edge_distance() {
        let bar = sa303_bar();
        let wide = check_bar(&bar, 0.0, 2000.0, 0.0, 4.0);
        let mut narrow_bar = bar;
        narrow_bar.narrow_width = 30.0; // bord deux fois plus proche
        let narrow = check_bar(&narrow_bar, 0.0, 2000.0, 0.0, 4.0);
        assert!(
            narrow.worst_tear_out().utilization > wide.worst_tear_out().utilization,
            "rapprocher le bord doit aggraver l'arrachement"
        );
    }

    /// Un effort purement axial est parallèle aux bords longs : il ne chasse
    /// rien devant lui, donc aucun arrachement.
    #[test]
    fn a_purely_axial_load_tears_out_nothing() {
        let check = check_bar(&sa303_bar(), 0.0, 0.0, 5000.0, 4.0);
        for t in &check.tear_out {
            assert_eq!(t.force_toward_edge_n, 0.0, "{}", t.location);
            assert_eq!(t.utilization, 0.0);
        }
    }

    /// La largeur critique est rendue pour qu'un balayage se lise dans le
    /// résultat, et élargir doit soulager.
    #[test]
    fn the_critical_width_is_reported_and_widening_relieves_the_bar() {
        let stress = |w: f64| {
            let mut bar = sa303_bar();
            bar.narrow_width = w;
            let check = check_bar(&bar, 0.0, 1000.0, 0.0, 4.0);
            assert_eq!(check.critical_width_mm, w);
            check.worst_section().stress_mpa
        };
        assert!(stress(70.0) < stress(60.0));
        assert!(stress(60.0) < stress(50.0));
        assert!(stress(50.0) < stress(40.0));
    }

    #[test]
    fn utilization_is_the_ratio_to_ultimate_over_the_safety_factor() {
        let bar = sa303_bar();
        let check = check_bar(&bar, 0.0, 1000.0, 0.0, 4.0);
        let w = check.worst_section();
        assert!((w.utilization - w.stress_mpa / (510.0 / 4.0)).abs() < 1e-9);
        assert!((w.yield_ratio - w.stress_mpa / 355.0).abs() < 1e-9);
    }
}

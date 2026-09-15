//! Barre arrière en flexion composée. C'est le mode que le modèle à deux
//! forces ne pouvait pas voir : tant que la barre était supposée bi-articulée,
//! elle ne portait qu'un effort normal et sa section n'avait qu'à passer la
//! traction. Goupillée en deux points sur le caisson du bas, elle fléchit.
//!
//! Diagramme de moment. La barre n'a que deux liaisons : la goupille de
//! couronne (articulation, moment nul) et la paire ancrage/verrou. Entre le
//! bout couronne et le premier trou de la paire, elle ne voit donc qu'un seul
//! effort, et son moment croît linéairement depuis zéro. Au-delà du premier
//! trou, la réaction de la paire le fait redescendre. Trois sections méritent
//! d'être regardées :
//!
//! - les deux goupilles de la paire, percées, dans la largeur étroite ;
//! - la transition large -> étroit, non percée, mais où la section se réduit
//!   d'un coup alors que le moment, lui, continue de croître.
//!
//! La transition n'est pas une précaution de principe : elle tombe à mi-chemin,
//! là où le moment vaut déjà les deux tiers de son maximum sur une section qui
//! vient de perdre 37 % de sa largeur.

use crate::speaker::{BarHole, BarHoles, RearBar};

/// Une section vérifiée, avec ce qui la sollicite et ce qu'elle vaut.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarSectionCheck {
    /// Nom lisible de la section, pour dire *où* ça travaille.
    pub location: String,
    /// Abscisse depuis l'extrémité couronne, mm.
    pub at_mm: f64,
    pub width_mm: f64,
    pub drilled: bool,
    /// Moment de flexion à cette section, N·mm (par flanc).
    pub moment_nmm: f64,
    /// Effort normal, N (par flanc). Constant le long de la barre.
    pub axial_n: f64,
    /// Contrainte de flexion composée, MPa.
    pub stress_mpa: f64,
    /// Rapport à `R_m / sf` — le critère, cohérent avec `checks::sandwich`.
    pub utilization: f64,
    /// Rapport à `f_y`, à titre indicatif : au-delà de 1, la barre a plastifié
    /// même si elle est loin de rompre. Ce n'est pas le critère retenu.
    pub yield_ratio: f64,
}

/// Résultat de la vérification : toutes les sections, et la pire.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarCheck {
    pub sections: Vec<BarSectionCheck>,
    /// Index de la section la plus sollicitée dans `sections`.
    pub worst: usize,
}

impl BarCheck {
    pub fn worst_section(&self) -> &BarSectionCheck {
        &self.sections[self.worst]
    }

    pub fn utilization(&self) -> f64 {
        self.worst_section().utilization
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
    let crown_at = bar.crown_hole(splay_deg).along();
    let admissible = bar.ultimate_strength / safety_factor;

    // Moment à une abscisse, tant qu'on reste avant le premier trou de la
    // paire : au-delà le diagramme redescend, et prendre la valeur linéaire
    // serait faux dans le sens non conservatif seulement après l'ancrage — on
    // s'arrête donc au premier trou.
    let moment_at = |x: f64| (shear_n * (x - crown_at)).abs();

    let mut sections = Vec::with_capacity(3);
    let mut push = |location: &str, at: f64, drilled: bool, moment: f64| {
        let width = bar.width_at(at);
        let w_net = bar.section_modulus_at(at, drilled);
        let a_net = bar.net_area_at(at, drilled);
        let stress = moment / w_net + axial_n.abs() / a_net;
        sections.push(BarSectionCheck {
            location: location.to_string(),
            at_mm: at,
            width_mm: width,
            drilled,
            moment_nmm: moment,
            axial_n,
            stress_mpa: stress,
            utilization: stress / admissible,
            yield_ratio: stress / bar.yield_strength,
        });
    };

    // Transition de largeur : section brute, mais rétrécie. Sans intérêt si la
    // barre est d'une seule largeur, ou si la transition tombe après la paire.
    let first_pair = bar.holes.latch.along().min(bar.holes.anchor.along());
    if bar.step_position > crown_at.min(first_pair) && bar.step_position < crown_at.max(first_pair)
    {
        push(
            "transition large → étroit",
            bar.step_position,
            false,
            moment_at(bar.step_position),
        );
    }
    // Premier trou de la paire : le sommet du diagramme.
    let (first_name, second_name) = if bar.holes.latch.along() <= bar.holes.anchor.along() {
        ("verrou", "ancrage")
    } else {
        ("ancrage", "verrou")
    };
    let second_pair = bar.holes.latch.along().max(bar.holes.anchor.along());
    push(first_name, first_pair, true, moment_at(first_pair));
    // Second trou de la paire : au-delà, la barre se termine sur quelques
    // millimètres qui ne portent rien. Le moment y est donc nul et cette
    // section ne travaille qu'en effort normal — l'extrapoler linéairement
    // depuis la couronne donnerait le moment le plus élevé du lot, à l'endroit
    // précis où il n'y en a pas.
    push(second_name, second_pair, true, 0.0);

    let worst = sections
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.utilization.total_cmp(&b.utilization))
        .map(|(i, _)| i)
        .unwrap_or(0);

    BarCheck { sections, worst }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Le module de flexion net traite le trou comme une fente sur la fibre
    /// neutre : `t(w³ − d0³)/(6w)`. Recoupé à la main : 325 N·m sur la section
    /// 40×10 percée Ø12,08 donnent 125,3 MPa. La formule `t(w − d0)²/6`, qui
    /// vaudrait pour un trou en fibre extrême, en donnerait 249 — soit un
    /// facteur 2 sur le dimensionnement de la barre.
    #[test]
    fn the_net_section_modulus_matches_the_hand_check() {
        let bar = sa303_bar();
        let w = bar.section_modulus_at(bar.holes.latch.along(), true);
        assert!((w - 2593.217).abs() < 1e-3, "W_net = {w}");
        assert!((325_000.0 / w - 125.33).abs() < 0.01);
    }

    /// Loi d'échelle : la contrainte est linéaire en effort, donc un transverse
    /// de 1 kN par flanc doit donner exactement le produit du bras par le
    /// module. Sert de garde-fou contre une unité qui se perdrait (N·m contre

    /// La transition doit être regardée : elle perd 37 % de largeur là où le
    /// moment vaut déjà les deux tiers de son maximum. Elle ne gouverne pas sur
    /// cette barre-ci, mais l'écart n'est pas d'un ordre de grandeur — une

    /// Le moment est nul à la couronne : une barre dont la paire serait au

    #[test]
    fn utilization_is_the_ratio_to_ultimate_over_the_safety_factor() {
        let bar = sa303_bar();
        let check = check_bar(&bar, 0.0, 1000.0, 0.0, 4.0);
        let w = check.worst_section();
        assert!((w.utilization - w.stress_mpa / (510.0 / 4.0)).abs() < 1e-9);
        assert!((w.yield_ratio - w.stress_mpa / 355.0).abs() < 1e-9);
    }
}

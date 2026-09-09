//! Sélection des pires cas parmi les clusters, en amont de l'agrégat (brief
//! §9) : bloc A par chemin de charge (`select_block_a`), bloc B enveloppe par
//! splay (`select_block_b`), doublons signalés entre les deux.

use super::joint::JointResult;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadCase {
    pub cluster_name: String,
    /// Numéro du joint, 1-indexé, depuis le haut de la grappe.
    pub joint_number: usize,
    pub result: JointResult,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockACase {
    pub labels: Vec<&'static str>,
    pub case_index: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockBCase {
    pub splay_deg: f64,
    pub case_index: usize,
    /// Déjà présent au bloc A : même grappe, même joint, donc même chargement.
    pub duplicate: bool,
}

fn max_by(
    cases: &[LoadCase],
    filter: impl Fn(&LoadCase) -> bool,
    key: impl Fn(&LoadCase) -> f64,
) -> Option<usize> {
    cases
        .iter()
        .enumerate()
        .filter(|(_, c)| filter(c))
        .max_by(|(_, a), (_, b)| key(a).partial_cmp(&key(b)).unwrap())
        .map(|(i, _)| i)
}

/// Bloc A : un cas par chemin de charge distinct (brief §9), fusionnés si le même
/// joint gagne plusieurs critères.
pub fn select_block_a(cases: &[LoadCase]) -> Vec<BlockACase> {
    let candidates: [(&'static str, Option<usize>); 6] = [
        (
            "effort orientation max",
            max_by(cases, |_| true, |c| c.result.mag_orientation()),
        ),
        (
            "effort pivot max",
            max_by(cases, |_| true, |c| c.result.mag_pivot()),
        ),
        (
            "charnière inversée",
            max_by(cases, |c| c.result.hinge_reversed, |c| c.result.mag_pivot()),
        ),
        (
            "compression",
            max_by(
                cases,
                |c| !c.result.traction,
                |c| c.result.mag_orientation(),
            ),
        ),
        (
            "deux zones chargées",
            max_by(
                cases,
                |_| true,
                |c| c.result.mag_orientation().min(c.result.mag_pivot()),
            ),
        ),
        (
            "inclinaison extrême",
            max_by(
                cases,
                |_| true,
                |c| {
                    c.result.inclination_deg.abs()
                        * c.result.mag_orientation().max(c.result.mag_pivot())
                },
            ),
        ),
    ];

    let mut out: Vec<BlockACase> = Vec::new();
    for (label, idx) in candidates {
        if let Some(idx) = idx {
            if let Some(existing) = out.iter_mut().find(|b| b.case_index == idx) {
                existing.labels.push(label);
            } else {
                out.push(BlockACase {
                    labels: vec![label],
                    case_index: idx,
                });
            }
        }
    }
    out
}

/// Bloc B : pire cas par réglage de splay, un par trou percé. Marque les doublons
/// déjà retenus au bloc A.
pub fn select_block_b(cases: &[LoadCase], block_a: &[BlockACase]) -> Vec<BlockBCase> {
    use std::collections::BTreeMap;
    // clé = splay * 10, arrondi, pour trier/dédupliquer sans souci de flottants.
    let mut best: BTreeMap<i64, usize> = BTreeMap::new();
    for (i, c) in cases.iter().enumerate() {
        let key = (c.result.splay_deg * 10.0).round() as i64;
        let is_better = match best.get(&key) {
            None => true,
            Some(&bi) => {
                let cur = cases[bi]
                    .result
                    .mag_orientation()
                    .max(cases[bi].result.mag_pivot());
                let cand = c.result.mag_orientation().max(c.result.mag_pivot());
                cand > cur
            }
        };
        if is_better {
            best.insert(key, i);
        }
    }
    let a_indices: std::collections::HashSet<usize> =
        block_a.iter().map(|b| b.case_index).collect();

    let mut out: Vec<BlockBCase> = best
        .into_iter()
        .map(|(k, i)| BlockBCase {
            splay_deg: k as f64 / 10.0,
            case_index: i,
            duplicate: a_indices.contains(&i),
        })
        .collect();
    out.sort_by(|a, b| a.splay_deg.partial_cmp(&b.splay_deg).unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::model::Compartment;
    use crate::speaker::CrownRow;
    use crate::vector::Vec2;

    /// Un `JointResult` synthétique : seuls les champs pertinents pour la sélection
    /// varient, le reste est un remplissage neutre.
    fn fake_joint(
        mag_o: f64,
        mag_p: f64,
        hinge_reversed: bool,
        traction: bool,
        splay_deg: f64,
        inclination_deg: f64,
    ) -> JointResult {
        JointResult {
            joint_index: 0,
            compartment: Compartment::Flown,
            free_body_count: 1,
            loaded_flank: 0,
            inclination_deg,
            splay_deg,
            row: CrownRow::of(splay_deg),
            crown_radius: 680.0,
            lever_mm: 660.0,
            loaded_orientation_hole: Vec2::ZERO,
            loaded_pivot_hole: Vec2::ZERO,
            constrained_hinge_hole: Vec2::ZERO,
            constrained_crown_splay: None,
            constrained_crown_hole: None,
            constrained_crown_row: None,
            constrained_crown_radius: None,
            constrained_is_frame: false,
            f_orientation: Vec2::new(mag_o, 0.0),
            f_pivot: Vec2::new(mag_p, 0.0),
            gravity_local: Vec2::new(0.0, -1.0),
            f_orientation_n: mag_o,
            f_orientation_angle_deg: 0.0,
            f_pivot_n: mag_p,
            f_pivot_angle_deg: 0.0,
            loaded_orientation_hole_global: Vec2::ZERO,
            loaded_pivot_hole_global: Vec2::ZERO,
            f_orientation_global: Vec2::new(mag_o, 0.0),
            f_pivot_global: Vec2::new(mag_p, 0.0),
            traction,
            hinge_reversed,
            bumper_moment_nm: 0.0,
            residual_n: 0.0,
            recommended_splay_range_deg: None,
            acoustically_optimal: true,
        }
    }

    fn load_case(joint_number: usize, result: JointResult) -> LoadCase {
        LoadCase {
            cluster_name: "Test".into(),
            joint_number,
            result,
        }
    }

    #[test]
    fn block_a_merges_labels_when_one_case_wins_multiple_criteria() {
        // case 0 : plus gros effort orientation, rien d'autre.
        // case 1 : plus gros effort pivot, en compression, charnière inversée,
        // et gagne aussi "deux zones chargées" et "inclinaison extrême" : cinq
        // critères sur un seul joint, une seule ligne attendue avec cinq labels.
        let cases = vec![
            load_case(1, fake_joint(100.0, 10.0, false, true, 1.0, -5.0)),
            load_case(2, fake_joint(50.0, 200.0, true, false, 2.0, 10.0)),
        ];
        let block_a = select_block_a(&cases);
        assert_eq!(block_a.len(), 2);

        let case0 = block_a.iter().find(|b| b.case_index == 0).unwrap();
        assert_eq!(case0.labels, vec!["effort orientation max"]);

        let case1 = block_a.iter().find(|b| b.case_index == 1).unwrap();
        assert_eq!(
            case1.labels,
            vec![
                "effort pivot max",
                "charnière inversée",
                "compression",
                "deux zones chargées",
                "inclinaison extrême",
            ]
        );
    }

    #[test]
    fn block_a_omits_criteria_with_no_matching_case() {
        // Aucun cas en compression ni charnière inversée dans ce jeu : ces deux
        // critères ne doivent produire aucune entrée fantôme (brief : "aucun cas
        // en compression" est un signal à part entière, pas une valeur par défaut).
        let cases = vec![
            load_case(1, fake_joint(100.0, 10.0, false, true, 1.0, -5.0)),
            load_case(2, fake_joint(50.0, 200.0, false, true, 2.0, 10.0)),
        ];
        let block_a = select_block_a(&cases);
        let case1 = block_a.iter().find(|b| b.case_index == 1).unwrap();
        assert!(!case1.labels.contains(&"compression"));
        assert!(!case1.labels.contains(&"charnière inversée"));
    }

    #[test]
    fn block_a_is_empty_for_empty_input() {
        assert!(select_block_a(&[]).is_empty());
    }

    #[test]
    fn block_b_envelope_picks_max_per_splay_and_sorts_ascending() {
        let cases = vec![
            load_case(1, fake_joint(100.0, 10.0, false, true, 1.0, 0.0)), // idx 0, max=100
            load_case(2, fake_joint(50.0, 40.0, false, true, 1.0, 0.0)), // idx 1, max=50, même splay
            load_case(1, fake_joint(30.0, 30.0, false, true, 2.0, 0.0)), // idx 2, max=30
        ];
        let block_a = select_block_a(&cases);
        let block_b = select_block_b(&cases, &block_a);

        assert_eq!(block_b.len(), 2, "un trou percé par splay distinct");

        let e1 = block_b
            .iter()
            .find(|b| (b.splay_deg - 1.0).abs() < 1e-9)
            .unwrap();
        assert_eq!(
            e1.case_index, 0,
            "idx0 (max 100) bat idx1 (max 50) au splay 1°"
        );

        let e2 = block_b
            .iter()
            .find(|b| (b.splay_deg - 2.0).abs() < 1e-9)
            .unwrap();
        assert_eq!(e2.case_index, 2);

        assert!(block_b[0].splay_deg < block_b[1].splay_deg);
    }

    #[test]
    fn block_b_flags_cases_already_retained_in_block_a() {
        let cases = vec![
            load_case(1, fake_joint(100.0, 10.0, false, true, 1.0, -5.0)), // idx0 : gagne tous les critères applicables
            load_case(2, fake_joint(10.0, 5.0, false, true, 2.0, 0.0)), // idx1 : strictement dominé, n'apparaît nulle part
        ];
        let block_a = select_block_a(&cases);
        assert!(block_a.iter().any(|b| b.case_index == 0));

        let block_b = select_block_b(&cases, &block_a);
        let e1 = block_b
            .iter()
            .find(|b| (b.splay_deg - 1.0).abs() < 1e-9)
            .unwrap();
        assert_eq!(e1.case_index, 0);
        assert!(
            e1.duplicate,
            "même grappe, même joint qu'au bloc A : inutile de le monter deux fois"
        );

        let e2 = block_b
            .iter()
            .find(|b| (b.splay_deg - 2.0).abs() < 1e-9)
            .unwrap();
        assert!(!e2.duplicate);
    }
}

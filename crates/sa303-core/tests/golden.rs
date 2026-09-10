//! Tests non négociables du brief §7 : invariants sur un jeu représentatif,
//! puis valeurs de référence (golden test).

use sa303_core::bumper::{
    BumperBarCompatibility, BumperBarModel, BumperCompatibility, BumperModel,
};
use sa303_core::cluster::{Cluster, Compartment, JointResult, JointSetting};
use sa303_core::settings::{AxisMapping, PinSpec, PlateSpec, Settings};
use sa303_core::speaker::{
    BelowCompatibility, Crown, Hinge, SpeakerMechanicalModel, SpeakerModel, SplayRange,
};
use sa303_core::vector::angle_of;
use sa303_core::{compute_aggregate, compute_cluster};

fn default_speaker() -> SpeakerModel {
    SpeakerModel {
        id: "sa303".into(),
        name: "SA303".into(),
        schema_version: 1,
        mechanical: SpeakerMechanicalModel {
            depth: 700.0,
            height: 550.0,
            total_vertical_angle: 20.0,
            mass_kg: 83.695,
            cg: [10.84, 11.91],
            hinge: Hinge {
                x: -338.43,
                y: 257.13,
                joint_separation: 552.384,
            },
            crown: Crown {
                radius: 680.0,
                delta: 20.0,
                anchor_angle: 2.5,
                splay0_angle: 5.0,
            },
            splay_grid: vec![
                0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0, 15.0, 20.0,
            ],
            frame_hole_splay: 0.0,
        },
        acoustics: Default::default(),
        // Grappe homogène : l'enceinte s'empile sous elle-même sans
        // recommandation acoustique, donc jamais de jonction signalée.
        compatible_below: vec![BelowCompatibility {
            speaker_model_id: "sa303".into(),
            flown: true,
            stacked: true,
            recommended_splay: None,
        }],
    }
}

fn default_settings() -> Settings {
    Settings {
        safety_factor: 4.0,
        dynamic_factor: 1.3,
        gravity: 9.80665,
        share_per_flank: 0.5,
        pin: PinSpec {
            diameter: 12.0,
            ultimate: 1000.0,
            net_section: 0.86,
        },
        plate: PlateSpec {
            flank_thickness: 4.0,
            bar_thickness: 10.0,
            ultimate: 510.0,
        },
        axis_mapping: AxisMapping {
            tool_x: "X".into(),
            tool_y: "-Y".into(),
        },
    }
}

fn joints(splays: &[f64]) -> Vec<JointSetting> {
    splays.iter().map(|&s| JointSetting { splay: s }).collect()
}

/// Un bumper est obligatoire (brief §11.6), en vol comme en stack : `bumper_id`
/// doit référencer un bumper compatible avec l'enceinte utilisée.
fn flown_cluster(
    name: &str,
    splays: &[f64],
    imposed_tilt: Option<f64>,
    bumper_id: &str,
) -> Cluster {
    Cluster {
        id: name.into(),
        name: name.into(),
        schema_version: 1,
        speaker_model_ids: vec!["sa303".into(); splays.len() + 1],
        compartment: Compartment::Flown,
        joints: joints(splays),
        imposed_tilt,
        tie_angle: None,
        bumper_model_id: bumper_id.into(),
        // Références figées : au sol, pour que l'altitude n'introduise aucune
        // variable dans des valeurs vérifiées à la main.
        bumper_height: 0.0,
    }
}

fn stack_cluster(name: &str, splays: &[f64], bottom_angle_deg: f64, bumper_id: &str) -> Cluster {
    Cluster {
        id: name.into(),
        name: name.into(),
        schema_version: 1,
        speaker_model_ids: vec!["sa303".into(); splays.len() + 1],
        compartment: Compartment::Stacked,
        joints: joints(splays),
        imposed_tilt: Some(bottom_angle_deg),
        tie_angle: None,
        bumper_model_id: bumper_id.into(),
        // Références figées : au sol, pour que l'altitude n'introduise aucune
        // variable dans des valeurs vérifiées à la main.
        bumper_height: 0.0,
    }
}

/// Jeu représentatif : les grappes à embarquer en seed (brief §8).
fn representative_clusters(bumper_id: &str) -> Vec<Cluster> {
    vec![
        flown_cluster("Droite 12", &[0.0; 11], None, bumper_id),
        flown_cluster(
            "Banane douce 12",
            &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 4.0, 5.0, 7.0],
            None,
            bumper_id,
        ),
        flown_cluster(
            "Grosse banane 12",
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0],
            None,
            bumper_id,
        ),
        flown_cluster(
            "J-array 14",
            &[
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 10.0, 15.0,
            ],
            None,
            bumper_id,
        ),
        flown_cluster(
            "Long splay 8",
            &[5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0],
            None,
            bumper_id,
        ),
        flown_cluster(
            "Assiette -6",
            &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 10.0, 12.0],
            Some(-6.0),
            bumper_id,
        ),
        stack_cluster("Stack classique 3", &[0.0, 20.0], 40.0, bumper_id),
        stack_cluster("Stack 4 boites", &[0.0, 0.0, 20.0], 40.0, bumper_id),
        stack_cluster("Stack peu incliné", &[0.0, 10.0], 20.0, bumper_id),
        stack_cluster("Stack droit 3", &[0.0, 0.0], 0.0, bumper_id),
    ]
}

fn default_bumper() -> BumperModel {
    BumperModel {
        id: "sa303-bumper".into(),
        name: "SA303-BUMPER".into(),
        schema_version: 1,
        depth: 702.0,
        height: 100.0,
        shackle_height_above_bumper: 40.0,
        max_direct_deport_mm: 702.0 / 2.0,
        compatible_speakers: vec![BumperCompatibility {
            speaker_model_id: "sa303".into(),
            flown: true,
            stacked: true,
        }],
    }
}

#[test]
fn invariant_equilibrium_residual_is_negligible() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    for cluster in representative_clusters(&bumper.id) {
        let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
            .expect("configuration possible");
        for j in &result.joints {
            let rext_norm = (j.f_orientation + j.f_pivot).norm().max(1.0);
            assert!(
                j.residual_n < 1e-6 * rext_norm.max(j.f_pivot.norm().max(j.f_orientation.norm())),
                "{}: résidu {} trop grand au joint {}",
                cluster.name,
                j.residual_n,
                j.joint_index + 1
            );
        }
    }
}

#[test]
fn invariant_lever_matches_trigonometric_recoupement() {
    use sa303_core::speaker::SpeakerGeometry;
    let sm = default_speaker();
    let geo = SpeakerGeometry::compute(&sm);
    for &s in &sm.mechanical.splay_grid {
        let lever = geo.lever(s);
        let check = geo.lever_check(s);
        assert!(
            (lever - check).abs() < 1e-3,
            "splay {s}: lever {lever} vs recoupement {check}"
        );
    }
}

#[test]
fn invariant_gravity_direction_matches_flank_inclination() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    for cluster in representative_clusters(&bumper.id) {
        let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
            .expect("configuration possible");
        for j in &result.joints {
            let a = angle_of(j.gravity_local);
            let expected = ((j.inclination_deg % 360.0) + 360.0) % 360.0;
            let diff = (a - expected).abs();
            let diff = diff.min(360.0 - diff);
            assert!(
                diff < 1e-6,
                "{}: gravité {a}° != inclinaison {expected}° au joint {}",
                cluster.name,
                j.joint_index + 1
            );
        }
    }
}

#[test]
fn invariant_monotonic_decrease_on_straight_cluster() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let cluster = flown_cluster("Droite 12", &[0.0; 11], None, &bumper.id);
    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    for pair in result.joints.windows(2) {
        assert!(
            pair[1].mag_orientation() < pair[0].mag_orientation(),
            "mO non décroissant entre joints {} et {}",
            pair[0].joint_index + 1,
            pair[1].joint_index + 1
        );
        assert!(
            pair[1].mag_pivot() < pair[0].mag_pivot(),
            "mP non décroissant entre joints {} et {}",
            pair[0].joint_index + 1,
            pair[1].joint_index + 1
        );
    }
}

fn angle_close(v: sa303_core::vector::Vec2, expected_deg: f64, tol_deg: f64) {
    let a = angle_of(v);
    let diff = (a - expected_deg).abs();
    let diff = diff.min(360.0 - diff);
    assert!(diff <= tol_deg, "attendu {expected_deg}°, obtenu {a}°");
}

fn check_case(j: &JointResult, force_n: f64, dir_deg: f64, tol_n: f64, tol_deg: f64) -> f64 {
    let mag = j.f_orientation.norm();
    assert!(
        (mag - force_n).abs() <= tol_n,
        "F attendu {force_n} N, obtenu {mag} N"
    );
    angle_close(j.f_orientation, dir_deg, tol_deg);
    mag
}

/// Bumper dédié aux valeurs de référence ci-dessous : sa hauteur d'accroche
/// (275 + 100 + 105 = 480 mm) reproduit exactement le point de levage utilisé
/// pour vérifier ces valeurs à la main, avant que le bumper ne devienne
/// obligatoire (brief) — un bumper obligatoire ne doit pas changer
/// silencieusement des valeurs déjà vérifiées.
fn golden_reference_bumper() -> BumperModel {
    BumperModel {
        shackle_height_above_bumper: 105.0,
        ..default_bumper()
    }
}

#[test]
fn golden_grosse_banane_12() {
    let sm = default_speaker();
    let bumper = golden_reference_bumper();
    let settings = default_settings();
    let cluster = flown_cluster(
        "Grosse banane 12",
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0],
        None,
        &bumper.id,
    );
    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    let j = |n: usize| &result.joints[n - 1];

    const TOL_N: f64 = 1.0;
    const TOL_DEG: f64 = 0.2;

    // J1
    check_case(j(1), 4353.0, 354.3, TOL_N, TOL_DEG);
    let mp = j(1).f_pivot.norm();
    assert!((mp - 1574.0).abs() <= TOL_N, "J1 F pivot {mp}");
    angle_close(j(1).f_pivot, 336.0, TOL_DEG);

    // J2
    check_case(j(2), 4776.0, 356.8, TOL_N, TOL_DEG);
    let mp = j(2).f_pivot.norm();
    assert!((mp - 789.0).abs() <= TOL_N, "J2 F pivot {mp}");
    angle_close(j(2).f_pivot, 308.6, TOL_DEG);

    // J3
    check_case(j(3), 5138.0, 352.3, TOL_N, TOL_DEG);
    let mp = j(3).f_pivot.norm();
    assert!((mp - 336.0).abs() <= TOL_N, "J3 F pivot {mp}");
    angle_close(j(3).f_pivot, 169.8, TOL_DEG);

    // J7
    check_case(j(7), 3662.0, 348.3, TOL_N, TOL_DEG);
    let mp = j(7).f_pivot.norm();
    assert!((mp - 1560.0).abs() <= TOL_N, "J7 F pivot {mp}");
    angle_close(j(7).f_pivot, 128.1, TOL_DEG);
}

#[test]
fn golden_stack_0_20_bottom_40() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let cluster = stack_cluster("Stack classique 3", &[0.0, 20.0], 40.0, &bumper.id);
    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    let j = |n: usize| &result.joints[n - 1];

    const TOL_N: f64 = 1.0;
    const TOL_DEG: f64 = 0.2;

    // J1 : flanc n°2, 1 enceinte portée
    assert_eq!(j(1).loaded_flank + 1, 2);
    assert_eq!(j(1).free_body_count, 1);
    check_case(j(1), 180.0, 358.8, TOL_N, TOL_DEG);
    let mp = j(1).f_pivot.norm();
    assert!((mp - 371.0).abs() <= TOL_N, "J1 F pivot {mp}");
    angle_close(j(1).f_pivot, 30.1, TOL_DEG);
    angle_close(j(1).gravity_local, 20.0, TOL_DEG);

    // J2 : flanc n°3, 2 enceintes portées
    assert_eq!(j(2).loaded_flank + 1, 3);
    assert_eq!(j(2).free_body_count, 2);
    check_case(j(2), 208.0, 358.8, TOL_N, TOL_DEG);
    let mp = j(2).f_pivot.norm();
    assert!((mp - 921.0).abs() <= TOL_N, "J2 F pivot {mp}");
    angle_close(j(2).f_pivot, 48.6, TOL_DEG);
    angle_close(j(2).gravity_local, 40.0, TOL_DEG);
}

#[test]
fn geometry_report_succeeds_and_matches_derived_reference_values() {
    let sm = default_speaker();
    let report = sa303_core::speaker::geometry_report(&sm)
        .expect("géométrie par défaut, doit recouper sans erreur");
    assert_eq!(report.ha, 10.0);
    // entraxe_bielle ≈ 38.12 mm (brief §3)
    assert!(
        (report.bielle_entraxe - 38.12).abs() < 0.01,
        "entraxe bielle {}",
        report.bielle_entraxe
    );
    assert_eq!(report.holes.len(), sm.mechanical.splay_grid.len());
    for hole in &report.holes {
        assert!(hole.discrepancy_mm <= sa303_core::speaker::LEVER_TOLERANCE_MM);
    }
    // Valeurs attendues du §3 : 660.5 mm sur splay pair, 649.4 mm sur splay impair.
    let even = report.holes.iter().find(|h| h.splay_deg == 2.0).unwrap();
    assert!((even.lever_mm - 660.5).abs() < 0.5);
    let odd = report.holes.iter().find(|h| h.splay_deg == 1.0).unwrap();
    assert!((odd.lever_mm - 649.4).abs() < 0.5);
}

#[test]
fn compute_aggregate_splits_compartments_and_selects_non_empty_blocks() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let speakers = vec![sm];
    let bumpers = vec![bumper.clone()];
    let clusters = representative_clusters(&bumper.id);
    let report = compute_aggregate(&speakers, &clusters, &settings, &bumpers, &[]);

    assert!(!report.flown.block_a.is_empty());
    assert!(!report.flown.block_b.is_empty());
    assert!(!report.stacked.block_a.is_empty());
    assert!(!report.stacked.block_b.is_empty());
    assert!(
        report.impossible_clusters.is_empty(),
        "aucune grappe ne devrait être impossible ici : {:?}",
        report.impossible_clusters
    );

    // Bloc B : un cas par splay distinct rencontré dans le compartiment.
    let flown_splays: std::collections::HashSet<i64> = clusters
        .iter()
        .filter(|c| c.compartment == Compartment::Flown)
        .flat_map(|c| c.joints.iter().map(|j| (j.splay * 10.0).round() as i64))
        .collect();
    assert_eq!(report.flown.block_b.len(), flown_splays.len());

    // Toutes les grandeurs publiées sont finies et non négatives.
    for case in report
        .flown
        .block_a
        .iter()
        .chain(report.flown.block_b.iter())
    {
        assert!(case.utilization_orientation.is_finite() && case.utilization_orientation >= 0.0);
        assert!(case.utilization_pivot.is_finite() && case.utilization_pivot >= 0.0);
    }
}

#[test]
fn missing_bumper_reference_is_reported_as_impossible_not_a_panic() {
    // Un bumper est obligatoire (brief §11.6) : une référence qui ne pointe
    // vers rien doit être signalée, jamais faire planter tout l'agrégat.
    let sm = default_speaker();
    let settings = default_settings();
    let speakers = vec![sm];
    let cluster = flown_cluster("sans bumper valide", &[0.0; 3], None, "bumper-inexistant");
    let report = compute_aggregate(&speakers, &[cluster], &settings, &[], &[]);

    assert_eq!(report.impossible_clusters.len(), 1);
    assert!(report.flown.block_a.is_empty());
    assert!(report.flown.block_b.is_empty());
}

fn default_bumper_bar() -> BumperBarModel {
    BumperBarModel {
        id: "sa303-bumper-bar".into(),
        name: "SA303-BUMPER-BAR".into(),
        schema_version: 1,
        max_deport_mm: 1500.0,
        compatible_bumpers: vec![BumperBarCompatibility {
            bumper_model_id: "sa303-bumper".into(),
        }],
    }
}

#[test]
fn bumper_free_hang_matches_manual_pickup_at_bumper_center() {
    // Le bumper ne fait que dériver un point d'accroche : la pendaison libre
    // qu'il produit doit reproduire exactement celle de la formule de base
    // (`phi_initial_free_hang`) appelée directement avec ce même point
    // (même formule, brief §4) — aucun écart introduit par le passage par
    // le bumper.
    use sa303_core::cluster::{phi_initial_free_hang, ChainSpeaker};
    use sa303_core::vector::Vec2;

    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    let cluster = flown_cluster("avec bumper", &splays, None, &bumper.id);
    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");

    let pickup_height =
        sm.mechanical.height / 2.0 + bumper.height + bumper.shackle_height_above_bumper;
    assert!(
        (pickup_height - 415.0).abs() < 1e-9,
        "hauteur attendue 415 mm, obtenu {pickup_height}"
    );

    let chain = vec![ChainSpeaker::from_model(&sm); splays.len() + 1];
    let expected_phi = phi_initial_free_hang(&chain, &splays, Vec2::new(0.0, pickup_height));

    assert!((result.phi_initial - expected_phi).abs() < 1e-9);
}

#[test]
fn bumper_imposed_tilt_solves_pickup_directly_above_cm() {
    // Propriété physique que le point d'accroche résolu doit vérifier : sans
    // tirette, la grappe suspendue s'oriente pour que le CG passe sous
    // l'accroche — donc pickup_global.x == cg.x exactement.
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let cluster = flown_cluster(
        "assiette imposée",
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0],
        Some(-6.0),
        &bumper.id,
    );
    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    let pickup_global = result.pickup_global.expect("pickup dérivé attendu en vol");
    assert!(
        (pickup_global.x - result.cg.x).abs() < 1e-6,
        "pickup.x {} != cg.x {}",
        pickup_global.x,
        result.cg.x
    );
}

#[test]
fn bumper_reports_deport_bar_only_when_pickup_exceeds_bumper_depth() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    // Assiette libre, accrochage centré : tient dans le bumper, pas de barre.
    let free = flown_cluster("libre", &splays, None, &bumper.id);
    let free_result = compute_cluster(std::slice::from_ref(&sm), &free, &settings, &bumper, &[])
        .expect("configuration possible");
    let free_view = free_result.bumper_view;
    assert_eq!(free_view.bar_deport_mm, Some(0.0));
    assert!(free_view.bumper_bar_start_global.is_none());

    // Assiette imposée modérée, loin de l'assiette libre naturelle : le point
    // d'accroche résolu doit sortir de l'aplomb du bumper (±351 mm), mais
    // rester dans la portée de la SA303-BUMPER-BAR (1500 mm, dérivée automatiquement
    // du bumper actif) : pas de tirette ici.
    let imposed = flown_cluster("imposée extrême", &splays, Some(-20.0), &bumper.id);
    let imposed_result = compute_cluster(
        std::slice::from_ref(&sm),
        &imposed,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect("configuration possible");
    let imposed_view = imposed_result.bumper_view;
    let deport = imposed_view.bar_deport_mm.expect("déport calculé en vol");
    assert!(deport.abs() > 0.0, "un déport était attendu, obtenu 0");
    assert!(imposed_view.bumper_bar_start_global.is_some());
}

#[test]
fn bumper_view_on_stack_has_no_pickup_fields() {
    // En stack le bumper est purement décoratif (posée sous l'enceinte du bas) :
    // aucune notion d'accroche à dériver.
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let cluster = stack_cluster("stack avec bumper", &[0.0, 20.0], 40.0, &bumper.id);

    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    let view = result.bumper_view;
    assert!(view.pickup_global.is_none());
    assert!(view.bumper_bar_start_global.is_none());
    assert!(view.bar_deport_mm.is_none());
    assert!(view.pickup_offset_mm.is_none());
}

#[test]
fn stack_bumper_stays_parallel_to_ground_regardless_of_bottom_speaker_tilt() {
    // Le bumper ne doit jamais tourner en stack : c'est l'enceinte de
    // référence (le bas) qui prend l'angle de calage, pas le bumper.
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let cluster = stack_cluster("stack incliné", &[0.0, 20.0], 40.0, &bumper.id);

    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    let outline = result.bumper_view.outline_global;

    // Rectangle non tourné : côtés horizontaux/verticaux (mêmes y par paires,
    // mêmes x par paires), quel que soit l'angle de calage du bas (40° ici).
    assert!(
        (outline[0].y - outline[1].y).abs() < 1e-9,
        "haut non horizontal"
    );
    assert!(
        (outline[2].y - outline[3].y).abs() < 1e-9,
        "bas non horizontal"
    );
    assert!(
        (outline[0].x - outline[3].x).abs() < 1e-9,
        "côté gauche non vertical"
    );
    assert!(
        (outline[1].x - outline[2].x).abs() < 1e-9,
        "côté droit non vertical"
    );
    assert!(
        outline[0].y > outline[2].y,
        "le haut doit être au-dessus du bas"
    );
}

#[test]
fn stack_bumper_front_top_corner_touches_bottom_speaker_front_bottom_corner() {
    // Correction utilisateur : le point de contact n'est pas "le coin le plus
    // bas au sens large", mais précisément le coin avant-bas de l'enceinte —
    // sinon, à angle nul le bumper décolle du sol d'un côté et à angle prononcé
    // l'enceinte passe au travers de l'autre.
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    for bottom_angle in [0.0, 20.0, 40.0] {
        let cluster = stack_cluster("stack incliné", &[0.0, 20.0], bottom_angle, &bumper.id);
        let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
            .expect("configuration possible");
        let outline = result.bumper_view.outline_global;

        let last = result.speakers[result.speakers.len() - 1];
        let front_bottom_local = sa303_core::speaker::speaker_outline(&sm)[3];
        let front_bottom_global = last.o + front_bottom_local.rotate(last.phi);

        assert!(
            (outline[0].x - front_bottom_global.x).abs() < 1e-9
                && (outline[0].y - front_bottom_global.y).abs() < 1e-9,
            "à {bottom_angle}°, coin avant-haut du bumper {:?} != coin avant-bas de l'enceinte {:?}",
            outline[0],
            front_bottom_global
        );
    }
}

#[test]
fn deport_beyond_bar_max_reach_auto_activates_a_tie_and_keeps_equilibrium() {
    // Au-delà de la portée de la SA303-BUMPER-BAR (dérivée automatiquement du bumper
    // actif — pas de sélection manuelle), le solveur plafonne l'accroche et
    // met lui-même en place une tirette — il n'y a pas de case à cocher, ni de
    // barre à choisir — sur le point 0° arrière-bas de l'enceinte du bas (même
    // référence que le bumper).
    let sm = default_speaker();
    let bumper = default_bumper();
    // Portée volontairement minuscule : garantit le dépassement quel que soit
    // le déport réellement nécessaire à -20°, sans avoir à le prédire.
    let bumper_bar = BumperBarModel {
        max_deport_mm: 50.0,
        ..default_bumper_bar()
    };
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0]; // somme 67°

    // Assiette imposée à -20°, direction de traction à 0° (connue tenable et
    // dégagée à cette assiette, cf. probe manuel) : rien ne bloque le câble.
    let mut cluster = flown_cluster("hors de portée", &splays, Some(-20.0), &bumper.id);
    cluster.tie_angle = Some(0.0);
    let result = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect("configuration possible via tirette");
    let view = result.bumper_view;

    assert!(
        view.bumper_bar_exceeded,
        "la barre aurait dû être jugée insuffisante"
    );
    assert!(
        result.tie_tension_n > 0.0,
        "une tirette aurait dû être activée automatiquement"
    );
    assert!(
        result.pickup_global.is_some(),
        "accroche toujours calculée en vol, même plafonnée"
    );

    let last = result.speakers[result.speakers.len() - 1];
    let expected_tie_point = last.o
        + sa303_core::speaker::SpeakerGeometry::compute(&sm)
            .anchor_at(0.0)
            .rotate(last.phi);
    let tie_point = result.tie_point_global.expect("point de tirette attendu");
    assert!(
        (tie_point.x - expected_tie_point.x).abs() < 1e-6
            && (tie_point.y - expected_tie_point.y).abs() < 1e-6,
        "la tirette doit s'accrocher au point 0° arrière-bas de l'enceinte du bas"
    );

    // L'accroche est plafonnée exactement à la portée de la barre.
    let pickup_local = (result.pickup_global.unwrap() - result.speakers[0].o)
        .rotate_transpose(result.speakers[0].phi);
    assert!(
        (pickup_local.x.abs() - bumper_bar.max_deport_mm).abs() < 1e-6,
        "l'accroche doit être plafonnée exactement à la portée de la barre, obtenu x={}",
        pickup_local.x
    );
}

#[test]
fn tie_direction_that_crosses_another_enceinte_is_impossible() {
    // La direction est un choix utilisateur (`tie_angle`), mais le solveur la
    // valide quand même : ici 90°, à cette assiette, vise en plein dans la
    // grappe au-dessus — physiquement impossible quelle que soit la tension.
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = BumperBarModel {
        max_deport_mm: 50.0,
        ..default_bumper_bar()
    };
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    let mut cluster = flown_cluster("tirette bloquée", &splays, Some(-6.0), &bumper.id);
    cluster.tie_angle = Some(90.0);
    let err = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect_err("la tirette à 90° traverse une autre enceinte de la grappe");
    assert!(
        err.reason.to_lowercase().contains("impossible"),
        "message d'erreur attendu explicite, obtenu : {}",
        err.reason
    );
}

#[test]
fn tie_direction_clear_of_other_enceintes_stays_possible() {
    // Même mécanisme (barre insuffisante) mais avec une direction (0°) qui,
    // à cette assiette, ne traverse rien et correspond à une vraie traction
    // (tension positive) — la tirette s'active normalement.
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = BumperBarModel {
        max_deport_mm: 50.0,
        ..default_bumper_bar()
    };
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    let mut cluster = flown_cluster("tirette possible", &splays, Some(-20.0), &bumper.id);
    cluster.tie_angle = Some(0.0);
    let result = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect("cette direction ne traverse rien et tire (ne pousse pas)");
    assert!(result.tie_tension_n > 0.0);
}

#[test]
fn user_can_pick_their_own_angle_inside_the_reported_range() {
    // Correction utilisateur : la direction n'est pas imposée par l'algorithme,
    // seulement délimitée. Ici on part de la plage renvoyée par le solveur (sans
    // angle choisi), puis on ré-exécute avec un angle explicite pris dedans : le
    // même résultat doit rester possible, avec la tension attendue pour CET angle.
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = BumperBarModel {
        max_deport_mm: 50.0,
        ..default_bumper_bar()
    };
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    let mut baseline = flown_cluster("angle connu", &splays, Some(-20.0), &bumper.id);
    baseline.tie_angle = Some(0.0); // connu tenable et dégagé à cette assiette
    let baseline_result = compute_cluster(
        std::slice::from_ref(&sm),
        &baseline,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect("angle de référence possible");
    let range = baseline_result
        .bumper_view
        .tie_angle_range_deg
        .expect("plage attendue dès qu'une tirette est nécessaire");

    let chosen_angle = (range[0] + range[1]) / 2.0 + 2.0; // un point différent de l'optimal, mais dans la plage
    let mut chosen = flown_cluster("avec choix", &splays, Some(-20.0), &bumper.id);
    chosen.tie_angle = Some(chosen_angle);
    let chosen_result = compute_cluster(
        std::slice::from_ref(&sm),
        &chosen,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect("un angle dans la plage annoncée doit rester tenable");
    let returned = chosen_result.tie_direction_angle_deg.unwrap();
    let diff = ((returned - chosen_angle) % 360.0 + 360.0) % 360.0;
    let diff = diff.min(360.0 - diff);
    assert!(
        diff < 1e-6,
        "l'angle choisi par l'utilisateur doit être respecté tel quel (modulo 360°) : demandé {chosen_angle}°, obtenu {returned}°"
    );
}

#[test]
fn user_chosen_angle_outside_the_valid_range_is_impossible() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = BumperBarModel {
        max_deport_mm: 50.0,
        ..default_bumper_bar()
    };
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    let mut cluster = flown_cluster("hors plage", &splays, Some(-20.0), &bumper.id);
    cluster.tie_angle = Some(180.0); // hors de la plage valable à -20° (cf. probe manuel)
    let err = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect_err("180° est hors de la plage valable à cette assiette");
    assert!(
        err.reason.to_lowercase().contains("impossible") && err.reason.contains("entre"),
        "message d'erreur avec la plage attendu, obtenu : {}",
        err.reason
    );
}

#[test]
fn tie_direction_is_never_a_pushing_direction() {
    // Régression signalée par l'utilisateur : avec une direction fixe par
    // défaut (0°), certaines assiettes (ex. -61° sur grosse banane 12)
    // donnaient une tension négative — le solveur l'acceptait silencieusement
    // en inversant l'effort à l'affichage, ce qui dessinait un câble
    // traversant la grappe. Il n'y a plus de direction fixe : le solveur
    // dérive celle qui minimise la tension parmi les seules directions
    // valables (positives). Ici, -61° est en fait tenable — mais seulement
    // dans une autre direction que 0°, que le solveur doit trouver seul.
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = BumperBarModel {
        max_deport_mm: 50.0,
        ..default_bumper_bar()
    };
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    let mut cluster = flown_cluster("grosse banane -61", &splays, Some(-61.0), &bumper.id);
    cluster.tie_angle = Some(90.0); // connu tenable à -61°, contrairement à 0°
    let result = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect("-61° est tenable dans une autre direction que 0°");
    assert!(
        result.tie_tension_n > 0.0,
        "une tension positive était attendue"
    );
}

#[test]
fn bumper_loads_satisfy_equilibrium_like_a_real_joint() {
    // Même invariant qu'un joint réel (brief §7, invariant 1) : la somme des
    // deux efforts au bumper doit reproduire le poids du corps libre (toutes
    // les enceintes), à l'échelle de `sharePerFlank` près.
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];
    let cluster = flown_cluster("charge bumper", &splays, Some(-6.0), &bumper.id);
    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    let view = result.bumper_view;

    let of = sa303_core::vector::dir_from_angle(view.orientation_angle_deg.unwrap())
        * view.orientation_force_n.unwrap();
    let pf = sa303_core::vector::dir_from_angle(view.pivot_angle_deg.unwrap())
        * view.pivot_force_n.unwrap();

    let w_total = result.total_mass_kg
        * settings.gravity
        * settings.dynamic_factor
        * settings.share_per_flank;
    let b0 = result.speakers[0];
    let gravity_local = sa303_core::vector::Vec2::new(0.0, -1.0).rotate_transpose(b0.phi) * w_total;

    let residual = (of + pf - gravity_local).norm();
    assert!(
        residual < 1e-6 * w_total.max(1.0),
        "résidu d'équilibre au bumper trop grand: {residual}"
    );
}

#[test]
fn bar_is_derived_automatically_from_the_active_bumper_never_selected_manually() {
    // Il n'y a pas de sélection de barre côté grappe : si plusieurs barres
    // sont connues, seule celle qui déclare le bumper actif compatible est
    // utilisée — les autres sont ignorées silencieusement, comme il se doit
    // pour un paramètre qui n'existe plus côté utilisateur.
    let sm = default_speaker();
    let bumper = default_bumper();
    let compatible_bar = BumperBarModel {
        max_deport_mm: 50.0,
        ..default_bumper_bar()
    };
    let irrelevant_bar = BumperBarModel {
        id: "autre-barre".into(),
        name: "Autre barre".into(),
        schema_version: 1,
        max_deport_mm: 1.0,
        compatible_bumpers: vec![BumperBarCompatibility {
            bumper_model_id: "un-autre-bumper".into(),
        }],
    };
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];
    let mut cluster = flown_cluster("auto barre", &splays, Some(-20.0), &bumper.id);
    cluster.tie_angle = Some(0.0); // connu tenable et dégagé à cette assiette

    let bars = [irrelevant_bar, compatible_bar.clone()];
    let result = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        &bars,
    )
    .expect("configuration possible");

    let pickup_local = (result.pickup_global.unwrap() - result.speakers[0].o)
        .rotate_transpose(result.speakers[0].phi);
    assert!(
        (pickup_local.x.abs() - compatible_bar.max_deport_mm).abs() < 1e-6,
        "la barre compatible avec le bumper actif aurait dû être choisie automatiquement, obtenu x={}",
        pickup_local.x
    );
}

// ---------------------------------------------------------------------------
// Grappes hétérogènes : plusieurs modèles d'enceinte dans la même chaîne.
// ---------------------------------------------------------------------------

/// Deuxième modèle, mécaniquement identique au premier mais deux fois plus
/// lourd et acoustiquement distinct : accrochable sous `sa303` à 10°
/// exactement, et sous lui-même à 20°.
fn heavy_speaker() -> SpeakerModel {
    let mut model = default_speaker();
    model.id = "sa303-heavy".into();
    model.name = "SA303-HEAVY".into();
    model.mechanical.mass_kg *= 2.0;
    model.compatible_below = vec![BelowCompatibility {
        speaker_model_id: "sa303-heavy".into(),
        flown: true,
        stacked: true,
        recommended_splay: Some(SplayRange::exactly(20.0)),
    }];
    model
}

/// `sa303` accepte `sa303-heavy` dessous, à 10° recommandés.
fn speaker_accepting_heavy_below() -> SpeakerModel {
    let mut model = default_speaker();
    model.compatible_below.push(BelowCompatibility {
        speaker_model_id: "sa303-heavy".into(),
        flown: true,
        stacked: true,
        recommended_splay: Some(SplayRange::exactly(10.0)),
    });
    model
}

fn mixed_cluster(splays: &[f64], model_ids: &[&str], bumper_id: &str) -> Cluster {
    Cluster {
        id: "mixte".into(),
        name: "Mixte".into(),
        schema_version: 2,
        speaker_model_ids: model_ids.iter().map(|id| (*id).to_string()).collect(),
        compartment: Compartment::Flown,
        joints: joints(splays),
        imposed_tilt: None,
        tie_angle: None,
        bumper_model_id: bumper_id.into(),
        // Références figées : au sol, pour que l'altitude n'introduise aucune
        // variable dans des valeurs vérifiées à la main.
        bumper_height: 0.0,
    }
}

#[test]
fn mixed_chain_weighs_each_speaker_on_its_own_mass() {
    // Une grappe hétérogène n'a pas une masse unique répétée : la masse totale
    // est la somme réelle, et le CG penche vers les enceintes lourdes.
    let light = speaker_accepting_heavy_below();
    let heavy = heavy_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let catalogue = vec![light.clone(), heavy.clone()];

    let cluster = mixed_cluster(
        &[10.0, 20.0],
        &["sa303", "sa303-heavy", "sa303-heavy"],
        &bumper.id,
    );
    let result = compute_cluster(&catalogue, &cluster, &settings, &bumper, &[])
        .expect("chaîne déclarée compatible");

    let expected_mass = light.mechanical.mass_kg + 2.0 * heavy.mechanical.mass_kg;
    assert!(
        (result.total_mass_kg - expected_mass).abs() < 1e-9,
        "masse totale attendue {expected_mass}, obtenue {}",
        result.total_mass_kg
    );

    // Le CG pondéré doit tomber plus bas que la moyenne géométrique simple,
    // puisque les deux enceintes du bas pèsent le double de celle du haut.
    let simple_mean_y =
        result.speakers.iter().map(|s| s.cg.y).sum::<f64>() / result.speakers.len() as f64;
    assert!(
        result.cg.y < simple_mean_y,
        "CG pondéré {} devrait être sous la moyenne simple {simple_mean_y}",
        result.cg.y
    );
}

#[test]
fn undeclared_joint_between_two_models_is_impossible() {
    // `sa303-heavy` n'accepte que lui-même dessous : remettre un `sa303` sous
    // lui n'est pas déclaré, donc refusé — jamais assemblé silencieusement.
    let light = speaker_accepting_heavy_below();
    let heavy = heavy_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let catalogue = vec![light, heavy];

    let cluster = mixed_cluster(&[10.0, 5.0], &["sa303", "sa303-heavy", "sa303"], &bumper.id);
    let err = compute_cluster(&catalogue, &cluster, &settings, &bumper, &[])
        .expect_err("jonction heavy → sa303 non déclarée");
    assert!(
        err.reason.contains("accrochable"),
        "message explicite attendu, obtenu : {}",
        err.reason
    );
}

#[test]
fn unknown_speaker_reference_is_impossible() {
    let bumper = default_bumper();
    let settings = default_settings();
    let catalogue = vec![default_speaker()];
    let cluster = mixed_cluster(&[0.0], &["sa303", "sa303-inconnue"], &bumper.id);

    let err =
        compute_cluster(&catalogue, &cluster, &settings, &bumper, &[]).expect_err("modèle inconnu");
    assert!(
        err.reason.contains("introuvable"),
        "obtenu : {}",
        err.reason
    );
}

#[test]
fn acoustic_recommendation_flags_the_joint_without_ever_failing() {
    // Le splay recommandé est un conseil de calage : hors plage, la jonction
    // reste calculée normalement, seulement signalée comme non optimale.
    let light = speaker_accepting_heavy_below();
    let heavy = heavy_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let catalogue = vec![light, heavy];
    let ids = ["sa303", "sa303-heavy", "sa303-heavy"];

    // 10° puis 20° : exactement ce que les deux modèles recommandent.
    let optimal = mixed_cluster(&[10.0, 20.0], &ids, &bumper.id);
    let optimal_result = compute_cluster(&catalogue, &optimal, &settings, &bumper, &[])
        .expect("configuration possible");
    assert!(optimal_result.joints.iter().all(|j| j.acoustically_optimal));
    assert_eq!(
        optimal_result.joints[0].recommended_splay_range_deg,
        Some([10.0, 10.0])
    );
    assert_eq!(
        optimal_result.joints[1].recommended_splay_range_deg,
        Some([20.0, 20.0])
    );

    // 4° au lieu de 10° sur la première jonction : signalée, mais calculée.
    let off = mixed_cluster(&[4.0, 20.0], &ids, &bumper.id);
    let off_result =
        compute_cluster(&catalogue, &off, &settings, &bumper, &[]).expect("jamais une erreur");
    assert!(!off_result.joints[0].acoustically_optimal);
    assert!(off_result.joints[1].acoustically_optimal);
    assert!(
        off_result.joints[0].residual_n < 1e-6 * off_result.joints[0].mag_pivot().max(1.0),
        "la jonction non optimale reste résolue à l'équilibre"
    );
}

#[test]
fn a_joint_without_declared_recommendation_is_never_flagged() {
    // `default_speaker` ne déclare aucune recommandation pour elle-même :
    // aucun splay ne doit être signalé, quel qu'il soit.
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let cluster = flown_cluster("libre", &[0.0, 15.0], None, &bumper.id);

    let result = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect("configuration possible");
    assert!(result.joints.iter().all(|j| j.acoustically_optimal));
    assert!(result
        .joints
        .iter()
        .all(|j| j.recommended_splay_range_deg.is_none()));
}

#[test]
fn speaker_count_must_match_joint_count() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    // 2 jonctions annoncées, mais seulement 2 enceintes déclarées.
    let cluster = mixed_cluster(&[0.0, 0.0], &["sa303", "sa303"], &bumper.id);

    let err = compute_cluster(std::slice::from_ref(&sm), &cluster, &settings, &bumper, &[])
        .expect_err("chaîne incohérente");
    assert!(
        err.reason.contains("incohérente"),
        "obtenu : {}",
        err.reason
    );
}

#[test]
fn mutually_declared_models_can_alternate_in_both_directions() {
    // Une jonction est une paire ordonnée : déclarer A→B ET B→A rend les deux
    // sens possibles, donc une chaîne peut redescendre puis remonter de modèle.
    // Hors de l'angle recommandé, la jonction est signalée — jamais refusée.
    let mut light = default_speaker();
    light.compatible_below.push(BelowCompatibility {
        speaker_model_id: "sa303-heavy".into(),
        flown: true,
        stacked: true,
        recommended_splay: Some(SplayRange::exactly(10.0)),
    });
    let mut heavy = heavy_speaker();
    heavy.compatible_below.push(BelowCompatibility {
        speaker_model_id: "sa303".into(),
        flown: true,
        stacked: true,
        recommended_splay: Some(SplayRange::exactly(10.0)),
    });
    let bumper = default_bumper();
    let settings = default_settings();
    let catalogue = vec![light, heavy];

    // sa303 → heavy (10°, recommandé) → sa303 (4°, hors reco).
    let cluster = mixed_cluster(&[10.0, 4.0], &["sa303", "sa303-heavy", "sa303"], &bumper.id);
    let result = compute_cluster(&catalogue, &cluster, &settings, &bumper, &[])
        .expect("les deux sens sont déclarés");

    assert!(result.joints[0].acoustically_optimal);
    assert!(
        !result.joints[1].acoustically_optimal,
        "4° au lieu de 10° : signalé, mais la grappe reste calculée"
    );
    assert_eq!(result.speaker_names.len(), 3);
    assert_eq!(result.speaker_names[1], "SA303-HEAVY");
}

/// Un angle sans trou percé n'est pas un angle « presque bon » : la broche n'a
/// nulle part où passer, donc la grappe n'est pas montable (brief §11.6). Sans
/// ce refus, changer l'accastillage laisserait les grappes existantes rendre
/// des efforts calculés sur une quincaillerie qui n'existe plus.
#[test]
fn a_splay_with_no_drilled_hole_is_impossible() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();

    assert!(
        !sm.mechanical.splay_grid.contains(&11.0),
        "ce test suppose que 11° n'est pas percé sur l'enceinte de référence"
    );
    let cluster = flown_cluster("trou absent", &[5.0, 11.0], None, &bumper.id);
    let err = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .expect_err("11° n'est percé nulle part sur cette enceinte");
    assert!(
        err.reason.contains("11°") && err.reason.contains("aucun trou"),
        "message attendu nommant l'angle manquant, obtenu : {}",
        err.reason
    );
}

/// Le pendant du test précédent : tous les angles percés doivent passer, sinon
/// le contrôle serait trop strict et bloquerait des grappes montables.
#[test]
fn every_drilled_splay_is_accepted() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();

    for &splay in &sm.mechanical.splay_grid {
        let cluster = flown_cluster("trou percé", &[splay], None, &bumper.id);
        compute_cluster(
            std::slice::from_ref(&sm),
            &cluster,
            &settings,
            &bumper,
            std::slice::from_ref(&bumper_bar),
        )
        .unwrap_or_else(|e| panic!("{splay}° est percé mais refusé : {}", e.reason));
    }
}

/// La hauteur de bumper situe la grappe dans l'espace ; elle ne doit toucher
/// à aucun effort. Une grappe pèse le même poids à 2 m qu'à 12 m.
#[test]
fn the_trim_height_moves_the_cluster_without_changing_a_single_force() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();
    let splays = [0.0, 5.0, 10.0];

    let ground = flown_cluster("au sol", &splays, None, &bumper.id);
    let mut flown = ground.clone();
    flown.bumper_height = 12_000.0;

    let at_ground = compute_cluster(
        std::slice::from_ref(&sm),
        &ground,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .unwrap();
    let in_the_air = compute_cluster(
        std::slice::from_ref(&sm),
        &flown,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .unwrap();

    assert_eq!(at_ground.tie_tension_n, in_the_air.tie_tension_n);
    assert_eq!(at_ground.phi_initial, in_the_air.phi_initial);
    for (a, b) in at_ground.joints.iter().zip(&in_the_air.joints) {
        assert_eq!(a.f_orientation_n, b.f_orientation_n);
        assert_eq!(a.f_pivot_n, b.f_pivot_n);
    }
    // Seule l'altitude bouge, et exactement de la hauteur demandée.
    assert_eq!(in_the_air.elevation.bumper_bottom_mm, 12_000.0);
    let rise = in_the_air.elevation.offset_mm - at_ground.elevation.offset_mm;
    assert!((rise - 12_000.0).abs() < 1e-9, "obtenu {rise}");
}

/// En stack à hauteur nulle, le dessous du bumper est le sol : rien de
/// l'ensemble ne doit passer sous zéro, sinon la mise en situation décrirait
/// un stack enterré.
#[test]
fn a_grounded_stack_sits_exactly_on_the_floor() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();

    let cluster = stack_cluster("stack au sol", &[0.0, 20.0], 40.0, &bumper.id);
    let result = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .unwrap();

    assert_eq!(result.elevation.bumper_bottom_mm, 0.0);
    assert!(
        result.elevation.lowest_point_mm >= -1e-9,
        "point le plus bas à {} mm : le stack passerait sous le sol",
        result.elevation.lowest_point_mm
    );
    assert!(result.elevation.highest_point_mm > result.elevation.lowest_point_mm);
    assert!(
        result.elevation.pickup_mm.is_none(),
        "pas de levage en stack"
    );
}

/// En vol, le point de levage est au-dessus du bumper : c'est par lui que
/// passe la charge.
#[test]
fn the_pickup_sits_above_the_bumper_in_the_air() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();

    let mut cluster = flown_cluster("en vol", &[0.0, 5.0], None, &bumper.id);
    cluster.bumper_height = 9_000.0;
    let result = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .unwrap();

    let pickup = result.elevation.pickup_mm.expect("levage en vol");
    assert!(pickup > 9_000.0, "levage à {pickup} mm, sous le bumper");
    assert!(result.elevation.lowest_point_mm < 9_000.0, "la grappe pend");
}

/// La cote du dessous de chaque enceinte est calculée côté Rust : c'est de la
/// trigonométrie sur une silhouette tournée, l'écran n'a pas à la refaire
/// (brief §1). Dans une grappe qui pend, elle décroît du haut vers le bas.
#[test]
fn each_speaker_reports_the_height_a_rigger_would_measure() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();

    let mut cluster = flown_cluster("cotes", &[0.0, 0.0, 0.0], None, &bumper.id);
    cluster.bumper_height = 10_000.0;
    let result = compute_cluster(
        std::slice::from_ref(&sm),
        &cluster,
        &settings,
        &bumper,
        std::slice::from_ref(&bumper_bar),
    )
    .unwrap();

    let bottoms = &result.elevation.speaker_bottom_mm;
    assert_eq!(bottoms.len(), result.speakers.len());
    for pair in bottoms.windows(2) {
        assert!(pair[0] > pair[1], "cotes non décroissantes : {bottoms:?}");
    }
    // La plus basse est le bas de l'ensemble, et tout est sous le bumper.
    let lowest = bottoms[bottoms.len() - 1];
    assert!((lowest - result.elevation.lowest_point_mm).abs() < 1e-9);
    assert!(bottoms[0] < result.elevation.bumper_bottom_mm);
}

/// Le bug corrigé : tant que l'accroche glissait le long du bumper, l'écran
/// affichait « 0 mm (centré) » parce qu'il lisait le déport de **barre**, nul
/// par définition dans cette zone. La position de l'accroche par rapport au
/// centre du bumper est une autre grandeur, et elle doit bouger dès que
/// l'accroche n'est plus centrée.
#[test]
fn a_pickup_that_slides_along_the_bumper_is_reported_as_off_centre() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let bumper_bar = default_bumper_bar();
    let settings = default_settings();
    let splays = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0];

    let view_at = |tilt: Option<f64>| {
        let cluster = flown_cluster("accroche", &splays, tilt, &bumper.id);
        compute_cluster(
            std::slice::from_ref(&sm),
            &cluster,
            &settings,
            &bumper,
            std::slice::from_ref(&bumper_bar),
        )
        .expect("configuration possible")
        .bumper_view
    };

    // Pendaison libre : l'accroche est bien centrée, les deux cotes sont nulles.
    let free = view_at(None);
    assert_eq!(free.pickup_offset_mm, Some(0.0));
    assert_eq!(free.bar_deport_mm, Some(0.0));

    // Assiette imposée proche de la pendaison libre : l'accroche se décale de
    // quelques dizaines de millimètres et reste dans l'aplomb du bumper. Aucune
    // barre n'est nécessaire — et pourtant la cote ne doit pas être nulle.
    // C'est exactement le cas que l'écran donnait pour « centré ».
    let nudged = view_at(Some(-10.0));
    let offset = nudged.pickup_offset_mm.expect("accroche calculée en vol");
    assert!(
        offset.abs() > 1.0,
        "accroche donnée pour centrée alors qu'elle est décalée : {offset} mm"
    );
    assert!(
        offset.abs() <= bumper.max_direct_deport_mm,
        "ce cas doit rester dans l'aplomb du bumper, obtenu {offset} mm"
    );
    assert_eq!(
        nudged.bar_deport_mm,
        Some(0.0),
        "aucune barre n'est engagée tant que l'accroche est sur le bumper"
    );
    assert!(nudged.bumper_bar_start_global.is_none());

    // Hors du bumper : les deux cotes deviennent non nulles, et l'accroche est
    // toujours plus loin du centre que ce que porte la barre.
    let far = view_at(Some(-20.0));
    let far_offset = far.pickup_offset_mm.expect("accroche calculée en vol");
    let far_bar = far.bar_deport_mm.expect("déport de barre calculé en vol");
    assert!(far_bar.abs() > 0.0);
    assert!(
        far_offset.abs() > far_bar.abs(),
        "l'accroche ({far_offset}) doit être plus excentrée que le déport de barre ({far_bar})"
    );
    // Et l'écart entre les deux vaut exactement la demi-longueur du bumper.
    let bumper_share = far_offset.abs() - far_bar.abs();
    assert!((bumper_share - bumper.max_direct_deport_mm).abs() < 1e-9);
}

/// L'enveloppe par splay ne dimensionne que les angles réellement montés. Les
/// trous percés qu'aucune grappe n'exploite doivent être nommés : sans ça, un
/// tableau de sept lignes pour dix-sept trous a l'air complet et laisse croire
/// que tous les angles sont couverts (brief §11.6).
#[test]
fn the_envelope_names_the_drilled_holes_it_says_nothing_about() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let speakers = vec![sm.clone()];
    let bumpers = vec![bumper.clone()];
    let clusters = representative_clusters(&bumper.id);
    let report = compute_aggregate(&speakers, &clusters, &settings, &bumpers, &[]);

    for compartment in [&report.flown, &report.stacked] {
        // Couverts et non couverts partitionnent exactement le perçage : ni
        // trou oublié des deux côtés, ni angle compté deux fois.
        let mut seen: Vec<f64> = compartment
            .block_b
            .iter()
            .map(|c| c.result.splay_deg)
            .chain(compartment.uncovered_splays_deg.iter().copied())
            .collect();
        seen.sort_by(f64::total_cmp);
        seen.dedup();
        assert_eq!(
            seen, sm.mechanical.splay_grid,
            "la partition ne recouvre pas le perçage de l'enceinte"
        );

        // Un angle signalé comme non couvert ne doit surtout pas figurer dans
        // le tableau : ce serait se contredire.
        for hole in &compartment.uncovered_splays_deg {
            assert!(
                !compartment
                    .block_b
                    .iter()
                    .any(|c| (c.result.splay_deg - hole).abs() < 1e-9),
                "{hole}° est à la fois couvert et signalé comme non couvert"
            );
        }
    }
}

/// Un perçage qui s'élargit sans nouvelle grappe fait grandir la liste des
/// angles non couverts : c'est exactement ce qui arrive après une modification
/// d'accastillage, et ça doit se voir.
#[test]
fn widening_the_drilling_widens_the_uncovered_list() {
    let mut sm = default_speaker();
    let bumper = default_bumper();
    let settings = default_settings();
    let bumpers = vec![bumper.clone()];
    let clusters = representative_clusters(&bumper.id);

    let before = compute_aggregate(
        std::slice::from_ref(&sm),
        &clusters,
        &settings,
        &bumpers,
        &[],
    );

    // On perce un angle de plus, sans toucher aux grappes.
    sm.mechanical.splay_grid.push(19.0);
    sm.mechanical.splay_grid.sort_by(f64::total_cmp);
    let after = compute_aggregate(
        std::slice::from_ref(&sm),
        &clusters,
        &settings,
        &bumpers,
        &[],
    );

    assert_eq!(
        after.flown.uncovered_splays_deg.len(),
        before.flown.uncovered_splays_deg.len() + 1
    );
    assert!(after.flown.uncovered_splays_deg.contains(&19.0));
    // Le tableau lui-même n'a pas bougé : aucune grappe ne monte 19°.
    assert_eq!(after.flown.block_b.len(), before.flown.block_b.len());
}

//! Accroche sur trous réels (bumper + barre de déport), avec le perçage livré
//! dans `src-tauri/assets` — provisoire tant que les cotes CAO ne sont pas
//! reportées, mais la logique de choix ne dépend pas des valeurs exactes.

use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::{
    Cluster, Compartment, JointSetting, RiggingRequest, RiggingSupport, RiggingView,
};
use sa303_core::compute_cluster;
use sa303_core::settings::{AxisMapping, PinSpec, PlateSpec, Settings};
use sa303_core::speaker::SpeakerModel;

fn settings() -> Settings {
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
        pull_back_tolerance_deg: 10.0,
    }
}

fn catalogue() -> (SpeakerModel, BumperModel, BumperBarModel) {
    let speaker = serde_json::from_str(include_str!(
        "../../../src-tauri/assets/speakers/sa303-isophase.json"
    ))
    .unwrap();
    let bumper = serde_json::from_str(include_str!(
        "../../../src-tauri/assets/bumpers/sa303-bumper.json"
    ))
    .unwrap();
    let bar = serde_json::from_str(include_str!(
        "../../../src-tauri/assets/bumper-bars/sa303-bumper-bar.json"
    ))
    .unwrap();
    (speaker, bumper, bar)
}

fn flown(splays: &[f64], tilt: Option<f64>, rigging: RiggingRequest) -> Cluster {
    Cluster {
        id: "t".into(),
        name: "t".into(),
        schema_version: 1,
        speaker_model_ids: vec!["sa303-isophase".into(); splays.len() + 1],
        compartment: Compartment::Flown,
        joints: splays.iter().map(|&splay| JointSetting { splay }).collect(),
        imposed_tilt: tilt,
        pull_back_angle: None,
        pull_back_enabled: false,
        manual_pull_back_tension_n: None,
        rigging,
        bumper_model_id: "sa303-bumper".into(),
        bumper_height: 0.0,
    }
}

fn solve(cluster: &Cluster) -> RiggingView {
    let (speaker, bumper, bar) = catalogue();
    let result = compute_cluster(&[speaker], cluster, &settings(), &bumper, &[bar])
        .unwrap_or_else(|e| panic!("{}", e.reason));
    result.bumper_view.rigging.expect("bumper aux trous déclarés")
}

fn request(support: RiggingSupport, points: u8) -> RiggingRequest {
    RiggingRequest {
        support,
        points,
        bar_mount_index: None,
    }
}

/// L'accroche tombe toujours sur un trou percé, jamais entre deux.
#[test]
fn single_point_lands_on_a_declared_hole() {
    let (_, bumper, _) = catalogue();
    let holes: Vec<f64> = bumper.rigging.unwrap().shackle_holes.iter().map(|h| h[0]).collect();
    let view = solve(&flown(&[2.0, 4.0, 5.0], Some(-2.0), request(RiggingSupport::Bumper, 1)));
    assert_eq!(view.support, RiggingSupport::Bumper);
    assert_eq!(view.points.len(), 1);
    let x = view.points[0].bumper_x_mm;
    assert!(holes.iter().any(|h| (h - x).abs() < 1e-9), "{x} hors perçage");
    assert!(view.tilt_error_deg.is_some());
}

/// En Auto, le bumper passe avant la barre ; quand il ne peut pas approcher
/// l'assiette, le solveur passe à la barre, qui fait mieux.
#[test]
fn auto_moves_to_the_bar_only_when_the_bumper_falls_short() {
    let view = solve(&flown(&[0.0, 0.0, 0.0], Some(2.0), RiggingRequest::default()));
    assert_eq!(view.support, RiggingSupport::Bar);
    let bumper_only = solve(&flown(
        &[0.0, 0.0, 0.0],
        Some(2.0),
        request(RiggingSupport::Bumper, 1),
    ));
    assert!(
        view.tilt_error_deg.unwrap().abs() < bumper_only.tilt_error_deg.unwrap().abs(),
        "barre {:?} vs bumper {:?}",
        view.tilt_error_deg,
        bumper_only.tilt_error_deg
    );
}

/// Les quatre montages de barre sortent du seul perçage.
#[test]
fn four_bar_mounts_are_offered() {
    let view = solve(&flown(&[2.0, 4.0], Some(0.0), request(RiggingSupport::Bar, 1)));
    let mut centres: Vec<f64> = view.bar_mounts.iter().map(|m| m.center_x_mm).collect();
    centres.sort_by(f64::total_cmp);
    assert_eq!(centres, vec![-288.0, -258.0, 258.0, 288.0]);
    assert_eq!(view.bar_holes_global.len(), 21);
}

/// À deux points, l'assiette demandée est tenue exactement, et les deux
/// chaînes reprennent le poids à elles deux, chacune en traction.
#[test]
fn two_points_hold_the_exact_tilt_and_share_the_weight() {
    for support in [RiggingSupport::Auto, RiggingSupport::Bar] {
        let cluster = flown(&[2.0, 4.0, 5.0], Some(-3.0), request(support, 2));
        let (speaker, bumper, bar) = catalogue();
        let result = compute_cluster(&[speaker], &cluster, &settings(), &bumper, &[bar]).unwrap();
        let view = result.bumper_view.rigging.unwrap();
        assert!((view.achieved_tilt_deg + 3.0).abs() < 1e-9);
        assert_eq!(view.points.len(), 2);
        let total: f64 = view.points.iter().map(|p| p.tension_n).sum();
        let weight = result.total_mass_kg * 9.80665 * 1.3;
        assert!((total - weight).abs() < 1e-6 * weight, "{total} ≠ {weight}");
        assert!(view.points.iter().all(|p| p.tension_n > 0.0));
        if support == RiggingSupport::Bar {
            assert_eq!(view.support, RiggingSupport::Bar);
        }
    }
}

/// Un montage imposé est respecté.
#[test]
fn a_requested_bar_mount_is_honoured() {
    let mut req = request(RiggingSupport::Bar, 2);
    req.bar_mount_index = Some(3);
    let view = solve(&flown(&[2.0, 4.0], Some(0.0), req));
    assert_eq!(view.bar_mount_index, Some(3));
}


/// Les deux goupilles de la barre reprennent exactement ce que les chaînes
/// tirent : la barre est en équilibre.
#[test]
fn bar_link_forces_balance_the_chains() {
    for points in [1, 2] {
        let view = solve(&flown(&[2.0, 4.0], Some(0.0), request(RiggingSupport::Bar, points)));
        assert_eq!(view.bar_link_forces.len(), 2);
        let links: f64 = view.bar_link_forces.iter().map(|f| f.force_global.y).sum();
        let chains: f64 = view.points.iter().map(|p| p.tension_n).sum();
        assert!((links - chains).abs() < 1e-6 * chains, "{links} ≠ {chains}");
        assert!(!view.bar_outline_global.is_empty());
    }
}

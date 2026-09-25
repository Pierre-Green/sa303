//! Tests proposés pour `crates/sa303-core/tests/` (livrable 5 de la validation
//! 2026-09). À copier dans `tests/validation.rs`. Les helpers `default_speaker`,
//! `default_settings`, `default_bumper`, `flown_cluster`, `stack_cluster`,
//! `joints` sont ceux de `tests/golden.rs` (à factoriser dans un module commun
//! `tests/common/mod.rs` si on ne veut pas les dupliquer).
//!
//! Chaque test fige un comportement **validé** par le solveur de référence,
//! ou un comportement **attendu** qu'une correction proposée doit produire ;
//! ces derniers sont marqués `#[ignore]` avec la référence de la correction.
//!
//! Tolérances : 1e-6 relatif sur les efforts (le solveur est en f64 et tout est
//! linéaire), 0,01 mm sur les fermetures géométriques.

use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::{Cluster, Compartment, JointResult};
use sa303_core::settings::Settings;
use sa303_core::speaker::SpeakerModel;
use sa303_core::vector::Vec2;
use sa303_core::compute_cluster;

// --- helpers repris de golden.rs -------------------------------------------
// fn default_speaker() -> SpeakerModel { ... }
// fn default_settings() -> Settings { ... }
// fn default_bumper() -> BumperModel { ... }
// fn flown_cluster(name, splays, imposed_tilt, bumper_id) -> Cluster { ... }
// fn stack_cluster(name, splays, bottom_angle_deg, bumper_id) -> Cluster { ... }

fn solve(cluster: &Cluster) -> sa303_core::cluster::ClusterResult {
    let sm = default_speaker();
    let bumper = default_bumper();
    compute_cluster(&[sm], cluster, &default_settings(), &bumper, &[]).expect("grappe résoluble")
}

fn cross(a: Vec2, b: Vec2) -> f64 {
    a.x * b.y - a.y * b.x
}

fn rel_close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * a.abs().max(b.abs()).max(1e-9)
}

/// Équilibre de **chaque** corps libre de chaque jonction, recomposé depuis
/// les efforts exportés et la position réelle de la tirette. Ce n'est pas
/// `residual_n` (nul par construction) ni `moment_residual_nmm` (faux avec
/// tirette, voir correction C2) : c'est un recalcul indépendant.
fn assert_joint_equilibrium(result: &sa303_core::cluster::ClusterResult, settings: &Settings, masses: &[f64]) {
    let share = settings.share_per_flank;
    let n = result.speakers.len();
    let tie = result.tie_direction_global.map(|d| d * result.tie_tension_n);
    for j in &result.joints {
        assert_eq!(j.compartment, Compartment::Flown, "ce test couvre le vol");
        let i = j.joint_index;
        // Efforts sur le corps libre (caissons i+1..n), globaux, les deux flancs.
        let f_ori = j.f_orientation_global * (-1.0 / share);
        let f_piv = j.f_pivot_global * (-1.0 / share);
        let bo = j.loaded_orientation_hole_global;
        let pb = {
            // goupille basse de bielle = ht du caisson i+1 en global
            let s = result.speakers[i + 1];
            s.o + Vec2::new(-338.433, 257.127).rotate(s.phi)
        };
        let mut f = f_ori + f_piv;
        let mut m = cross(bo, f_ori) + cross(pb, f_piv);
        for k in (i + 1)..n {
            let w = masses[k] * settings.gravity * settings.dynamic_factor;
            let wv = Vec2::new(0.0, -w);
            f = f + wv;
            m += cross(result.speakers[k].cg, wv);
        }
        if let (Some(t), Some(q)) = (tie, result.tie_point_global) {
            f = f + t;
            m += cross(q, t);
        }
        assert!(f.norm() < 1e-6, "jonction {i} : résidu de force {}", f.norm());
        assert!(m.abs() < 1e-3, "jonction {i} : résidu de moment {m} N·mm");
    }
}

#[test]
fn every_free_body_is_in_equilibrium_with_and_without_tie() {
    let sm = default_speaker();
    let masses = vec![sm.mechanical.mass_kg; 14];
    for (tilt, tie) in [(None, None), (Some(25.0), Some(180.0)), (Some(25.0), Some(270.0))] {
        let mut c = flown_cluster("eq", &[2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0, 10.5], tilt, "sa303-bumper");
        c.tie_angle = tie;
        let r = solve(&c);
        assert_joint_equilibrium(&r, &default_settings(), &masses);
    }
}

/// La bielle est un élément à deux forces : `f_pivot_global` colinéaire à
/// hb(haut) → ht(bas), en traction comme en compression.
#[test]
fn the_bielle_force_stays_on_its_axis_even_when_compressed() {
    let mut c = flown_cluster("col", &[2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0, 10.5], Some(25.0), "sa303-bumper");
    c.tie_angle = Some(180.0);
    let r = solve(&c);
    for j in &r.joints {
        let pa = j.loaded_pivot_hole_global;
        let s = r.speakers[j.joint_index + 1];
        let pb = s.o + Vec2::new(-338.433, 257.127).rotate(s.phi);
        let u = (pb - pa).normalize();
        assert!(cross(u, j.f_pivot_global).abs() < 1e-9 * j.f_pivot_n.max(1.0));
    }
}

/// Fermeture géométrique : le `ht` du caisson du bas tombe sur la goupille
/// basse de bielle, et le trou de couronne construit par la cotation de la
/// barre coïncide avec la polaire du caisson (< 0,01 mm).
#[test]
fn joints_close_geometrically_for_every_drilled_splay() {
    let sm = default_speaker();
    let geo = sa303_core::speaker::SpeakerGeometry::compute(&sm);
    let bar = &sm.mechanical.rear_bar;
    for &s in &sm.mechanical.splay_grid {
        let row = geo.crown_row_at(s);
        let hole = bar.crown_hole(row);
        let along = hole.along() - bar.holes.anchor.along();
        let lateral = hole.lateral() - bar.holes.anchor.lateral();
        let e_axis = (geo.anchor_local - geo.latch_local).normalize();
        let e_front = Vec2::new(-e_axis.y, e_axis.x);
        let top_in_lower = geo.anchor_local + e_axis * along + e_front * lateral;
        let carried = geo.pv_at(s) + (top_in_lower - geo.ht).rotate(s.to_radians());
        let gap = (carried - geo.crown(s)).norm();
        assert!(gap < 0.01, "splay {s} : écart couronne polaire/barre {gap} mm");
    }
}

/// Bumper : bielle (71,6) + barre bumper ferment sur l'entraxe des pions du
/// plan (656,829 mm) à mieux que 5 µm, et les deux pions sont à la même
/// hauteur dans le repère caisson.
#[test]
fn the_bumper_pins_close_on_the_drawing_span_and_are_level() {
    let c = flown_cluster("b", &[0.0, 0.0, 0.0], None, "sa303-bumper");
    let r = solve(&c);
    let b = &r.bumper_view;
    let span = b.pin_span_mm;
    assert!((span - (702.0 - 12.567 - 32.604)).abs() < 0.005, "entraxe {span}");
    let s0 = r.speakers[0];
    let top = (b.orientation_point_global.unwrap() - s0.o).rotate_transpose(s0.phi);
    let front = (b.pivot_point_global.unwrap() - s0.o).rotate_transpose(s0.phi);
    assert!((top.y - front.y).abs() < 0.01, "pions non alignés : {} / {}", top.y, front.y);
}

/// La manille reprend exactement `W·k_dyn − tirette` et le moment de
/// l'ensemble autour du pickup est nul.
#[test]
fn the_pickup_balances_weight_and_tie_with_zero_moment() {
    let sm = default_speaker();
    let st = default_settings();
    let mut c = flown_cluster("pk", &[2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0, 10.5], Some(25.0), "sa303-bumper");
    c.tie_angle = Some(180.0);
    let r = solve(&c);
    let w = r.total_mass_kg * st.gravity * st.dynamic_factor;
    let tie = r.tie_direction_global.unwrap() * r.tie_tension_n;
    let expected = Vec2::new(0.0, w) - tie;
    assert!((r.bumper_view.support_force_global - expected).norm() < 1e-6);
    let pk = r.pickup_global.unwrap();
    let m = cross(r.cg - pk, Vec2::new(0.0, -w)) + cross(r.tie_point_global.unwrap() - pk, tie);
    assert!(m.abs() < 1e-3, "moment résiduel au pickup {m}");
    let _ = sm;
}

/// Invariances : élévation, tirette nulle ≡ pendaison libre, linéarité,
/// k_dyn.
#[test]
fn forces_are_invariant_to_trim_height_and_scale_linearly() {
    let base = flown_cluster("inv", &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 10.5, 10.5, 10.5], None, "sa303-bumper");
    let mut high = base.clone();
    high.bumper_height = 8000.0;
    let (a, b) = (solve(&base), solve(&high));
    for (x, y) in a.joints.iter().zip(&b.joints) {
        assert_eq!(x.f_anchor_n, y.f_anchor_n);
    }
    // tirette nulle : assiette imposée égale à la pendaison libre → pickup centré, mêmes efforts
    let mut same = base.clone();
    same.imposed_tilt = Some(a.phi_free_hang.unwrap().to_degrees());
    let c = solve(&same);
    assert!(c.bumper_view.pickup_offset_mm.unwrap().abs() < 1e-6);
    assert_eq!(c.tie_tension_n, 0.0);
    for (x, y) in a.joints.iter().zip(&c.joints) {
        assert!(rel_close(x.f_anchor_n, y.f_anchor_n, 1e-9));
    }
    // k_dyn : tout est proportionnel
    let mut st = default_settings();
    st.dynamic_factor = 1.1;
    let d = compute_cluster(&[default_speaker()], &base, &st, &default_bumper(), &[]).unwrap();
    for (x, y) in a.joints.iter().zip(&d.joints) {
        assert!(rel_close(y.f_anchor_n / x.f_anchor_n, 1.1 / 1.3, 1e-9));
    }
}

/// Cas à la main : 2 caissons. λ = −M_ext / bras, paire = F/2 ± M_G/d.
#[test]
fn two_cabinets_match_the_closed_form() {
    let sm = default_speaker();
    let st = default_settings();
    let c = flown_cluster("2", &[0.0], None, "sa303-bumper");
    let r = solve(&c);
    let j = &r.joints[0];
    let w = sm.mechanical.mass_kg * st.gravity * st.dynamic_factor;
    let s1 = r.speakers[1];
    let pb = s1.o + Vec2::new(-338.433, 257.127).rotate(s1.phi);
    let pa = j.loaded_pivot_hole_global;
    let bo = j.loaded_orientation_hole_global;
    let u = (pb - pa).normalize();
    let lever = cross(pb - bo, u);
    let m_ext = cross(s1.cg - bo, Vec2::new(0.0, -w));
    let lambda = -m_ext / lever;
    assert!(rel_close(j.f_pivot_n, lambda.abs() * st.share_per_flank, 1e-9));
    let f_ori = Vec2::new(0.0, w) - u * lambda;
    assert!(rel_close(j.f_orientation_n, f_ori.norm() * st.share_per_flank, 1e-9));
    let g = (j.anchor_hole_global + j.latch_hole_global) * 0.5;
    let m_g = cross(bo - g, f_ori);
    assert!(rel_close(j.bar_moment_at_pair_nm, m_g * st.share_per_flank / 1000.0, 1e-9));
    assert!(rel_close((j.anchor_hole_global - j.latch_hole_global).norm(), 100.0, 1e-5));
}

/// Refus des configurations invalides.
#[test]
fn invalid_configurations_are_refused_with_a_reason() {
    let sm = default_speaker();
    let bumper = default_bumper();
    let st = default_settings();
    // splay hors grille (dont l'ancien 10°)
    for s in [7.0, 10.0, 2.5] {
        let c = flown_cluster("bad", &[s, 0.0], None, "sa303-bumper");
        assert!(compute_cluster(&[sm.clone()], &c, &st, &bumper, &[]).is_err(), "splay {s} accepté");
    }
    // tirette en poussée
    let mut c = flown_cluster("push", &[2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0, 10.5], Some(25.0), "sa303-bumper");
    for angle in [0.0, 90.0, 45.0] {
        c.tie_angle = Some(angle);
        let bars = [BumperBarModel { id: "bar".into(), name: "bar".into(), schema_version: 1, max_deport_mm: 650.0, compatible_bumpers: vec![sa303_core::bumper::BumperBarCompatibility { bumper_model_id: "sa303-bumper".into() }] }];
        assert!(compute_cluster(&[sm.clone()], &c, &st, &bumper, &bars).is_err(), "tirette à {angle}° acceptée");
    }
}

// ---------------------------------------------------------------------------
// Comportements attendus après correction (ignorés tant que non intégrés)
// ---------------------------------------------------------------------------

/// C2 : le résidu de moment exporté doit être nul aussi avec tirette.
#[test]
#[ignore = "correction C2 : moment_residual_nmm doit prendre la tirette en son point d'application"]
fn moment_residual_is_zero_even_with_a_tie() {
    let mut c = flown_cluster("res", &[2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0, 10.5], Some(25.0), "sa303-bumper");
    c.tie_angle = Some(180.0);
    let bars = [BumperBarModel { id: "bar".into(), name: "bar".into(), schema_version: 1, max_deport_mm: 650.0, compatible_bumpers: vec![sa303_core::bumper::BumperBarCompatibility { bumper_model_id: "sa303-bumper".into() }] }];
    let r = compute_cluster(&[default_speaker()], &c, &default_settings(), &default_bumper(), &bars).unwrap();
    for j in &r.joints {
        assert!(j.moment_residual_nmm < 1e-3, "J{} : {}", j.joint_index, j.moment_residual_nmm);
    }
}

/// C3 : `hinge_reversed` = bielle en compression (signe de λ), pas signe du
/// produit scalaire avec la gravité.
#[test]
#[ignore = "correction C3 : hinge_reversed défini par le signe de l'effort axial de bielle"]
fn hinge_reversed_means_compression_regardless_of_cabinet_attitude() {
    let c = flown_cluster("rev", &[20.0; 7], None, "sa303-bumper");
    let r = solve(&c);
    for j in &r.joints {
        let s = r.speakers[j.joint_index + 1];
        let pb = s.o + Vec2::new(-338.433, 257.127).rotate(s.phi);
        let u = (pb - j.loaded_pivot_hole_global).normalize();
        // f_pivot_global = −u·λ·share ⇒ λ > 0 (compression) ⇔ f_pivot_global·u < 0
        let compressed = j.f_pivot_global.dot(u) < 0.0;
        assert_eq!(j.hinge_reversed, compressed, "J{}", j.joint_index);
    }
}

/// C4 : la convention §2 met 180° **vers le haut** et 270° vers l'arrière. Ce
/// test fige la convention pour que les commentaires (« 180° = vers
/// l'arrière ») et l'écran soient corrigés dans le bon sens, et non le code.
#[test]
fn the_section_2_convention_puts_180_up_and_270_backwards() {
    let up = sa303_core::vector::dir_from_angle(180.0);
    let back = sa303_core::vector::dir_from_angle(270.0);
    assert!((up.x).abs() < 1e-12 && (up.y - 1.0).abs() < 1e-12);
    assert!((back.x - 1.0).abs() < 1e-12 && back.y.abs() < 1e-12);
}

/// C1 : l'export doit porter l'enveloppe « tout l'axial sur un pion » pour les
/// vérifications de goupille de la paire.
#[test]
#[ignore = "correction C1 : champs f_anchor_env_n / f_latch_env_n à ajouter"]
fn the_pair_envelope_puts_all_the_axial_on_one_pin() {
    // f_env = hypot(F_transverse/2 ± M_G/d, F_axial), par flanc.
    // À implémenter avec les nouveaux champs ; voir compare.py pour la formule.
}

/// C6 : en stack, l'assiette du premier caisson doit être l'une des barres de
/// bumper déclarées (0/10/20°), et la liaison doit passer par bielle + barre.
#[test]
#[ignore = "correction C6 : stack sur barres de bumper déclarées"]
fn a_stack_tilt_that_matches_no_bumper_bar_is_refused() {
    let c = stack_cluster("st", &[0.0, 0.0], 7.0, "sa303-bumper");
    assert!(compute_cluster(&[default_speaker()], &c, &default_settings(), &default_bumper(), &[]).is_err());
}

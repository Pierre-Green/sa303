//! Tests non négociables de la liaison par bielle (brief §8). Ils portent sur
//! la cinématique seule — pas sur les charges — parce que c'est elle qui a
//! changé : le pivot avant n'est plus un point fixe du caisson, et les huit
//! trous de couronne ne sont plus sur un arc centré sur un point unique.

use sa303_core::speaker::{
    BelowCompatibility, Crown, Hinge, RearBar, SpeakerGeometry, SpeakerMechanicalModel, SpeakerModel,
};

/// Perçage de référence SA303 (brief §1) : les distances au bord viennent de
/// la table 3.9 EN 1993-1-8 appliquée à la charge réelle, pas d'un forfait.
fn sa303() -> SpeakerModel {
    SpeakerModel {
        id: "sa303".into(),
        name: "SA303".into(),
        schema_version: 4,
        mechanical: SpeakerMechanicalModel {
            depth: 700.0,
            height: 550.0,
            total_vertical_angle: 20.0,
            mass_kg: 83.695,
            cg: [10.84, 11.91],
            hinge: Hinge {
                x: -338.431,
                y: 257.122,
                joint_separation: 552.379,
                edge_perp: 12.569,
            },
            crown: Crown {
                radius: 680.0,
                delta: 20.0,
                anchor_angle: 3.0,
                latch_angle: 1.0,
                splay0_angle: 5.0,
            },
            splay_grid: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 20.0],
            frame_hole_splay: 0.0,
            rear_bar: RearBar {
                thickness: 10.0,
                length: 360.739,
                wide_width: 64.0,
                wide_length: 150.739,
                narrow_width: 40.0,
                hole_diameter: 12.08,
                crown_hole_outer_at: 15.863,
                crown_hole_inner_at: 20.121,
                latch_hole_at: 321.93,
                anchor_hole_at: 344.877,
                yield_strength: 355.0,
                ultimate_strength: 510.0,
            },
        },
        acoustics: Default::default(),
        compatible_below: vec![BelowCompatibility {
            speaker_model_id: "sa303".into(),
            flown: true,
            stacked: true,
            recommended_splay: None,
        }],
    }
}

/// Les huit crans relevés sur le perçage réel (brief §4), repère du caisson.
/// C'est la référence : si le calcul direct s'en écarte, c'est le calcul qui a
/// tort, pas la table.
const CROWN_TABLE: [(f64, f64, f64, f64); 8] = [
    (0.0, 680.0, 318.396, -119.260),
    (1.0, 660.0, 296.332, -113.335),
    (2.0, 680.0, 312.519, -96.438),
    (3.0, 660.0, 290.262, -91.293),
    (4.0, 680.0, 305.850, -73.847),
    (5.0, 660.0, 283.427, -69.487),
    (10.0, 680.0, 281.179, -7.731),
    (20.0, 680.0, 225.211, 95.354),
];

/// §8.1 — le calcul direct du trou de couronne (§6) doit retomber sur la table
/// relevée, à 0.01 mm près.
#[test]
fn every_crown_hole_matches_the_drilled_table() {
    let geo = SpeakerGeometry::compute(&sa303());
    for (splay, radius, x, y) in CROWN_TABLE {
        assert_eq!(
            geo.crown_radius_at(splay),
            radius,
            "splay {splay}° : mauvaise couronne (parité)"
        );
        let hole = geo.crown(splay);
        let d = ((hole.x - x).powi(2) + (hole.y - y).powi(2)).sqrt();
        assert!(
            d < 0.01,
            "splay {splay}° : calculé ({:.3}, {:.3}), table ({x}, {y}), écart {d:.4} mm",
            hole.x,
            hole.y
        );
    }
}

/// §8.2 — le décalage avant reste invisible sur la plage line source, et borné
/// partout ailleurs. C'est ce qui autorise à n'afficher que la composante
/// verticale comme espacement entre caissons.
#[test]
fn the_front_offset_stays_negligible_over_the_line_source_range() {
    let geo = SpeakerGeometry::compute(&sa303());
    for (splay, ..) in CROWN_TABLE {
        let front = geo.joint_offset(splay).front_mm;
        assert!(front < 0.9, "splay {splay}° : décalage avant {front:.4} mm");
        if splay <= 5.0 {
            assert!(
                front < 0.06,
                "splay {splay}° (line source) : décalage avant {front:.4} mm"
            );
        }
    }
}

/// §8.3 — l'écartement vertical est ce qu'on affiche : il doit croître avec le
/// splay, sinon deux crans voisins deviendraient indiscernables à l'écran.
#[test]
fn the_vertical_gap_grows_monotonically_with_the_splay() {
    let geo = SpeakerGeometry::compute(&sa303());
    let mut previous = f64::NEG_INFINITY;
    for (splay, ..) in CROWN_TABLE {
        // Négatif quand les coins s'écartent : c'est l'amplitude qui croît.
        let gap = -geo.joint_offset(splay).vertical_mm;
        assert!(
            gap > previous,
            "splay {splay}° : jour {gap:.4} mm, pas plus que le cran précédent ({previous:.4} mm)"
        );
        previous = gap;
    }
}

/// §8.4 — le partage est égal et imposé par construction : la bielle prend
/// exactement la moitié du splay, jamais une fraction calculée ni ajustée.
#[test]
fn the_bielle_takes_exactly_half_the_splay() {
    let geo = SpeakerGeometry::compute(&sa303());
    for (splay, ..) in CROWN_TABLE {
        assert_eq!(SpeakerGeometry::bielle_rotation_deg(splay), splay / 2.0);
        // Et la géométrie elle-même doit être d'accord : la bielle étant un
        // élément à deux forces, sa ligne d'action passe par ses deux
        // goupilles, donc son inclinaison se lit sur `hb -> pv_at`.
        let axis = geo.pv_at(splay) - geo.hb;
        let tilt = axis.x.atan2(-axis.y).to_degrees();
        assert!(
            (tilt - splay / 2.0).abs() < 1e-9,
            "splay {splay}° : bielle inclinée de {tilt}°"
        );
        assert!(
            (axis.norm() - geo.bielle_entraxe).abs() < 1e-9,
            "splay {splay}° : la bielle a changé de longueur"
        );
    }
}

/// §8.5 — couronne et ancrage sont tous deux radiaux depuis la goupille basse
/// de bielle : leur entraxe ne peut donc pas dépendre du splay. C'est ce qui
/// permet à une seule longueur de barre de desservir tous les crans d'une même
/// couronne.
#[test]
fn the_bar_length_does_not_depend_on_the_splay() {
    let geo = SpeakerGeometry::compute(&sa303());
    for row in [680.0, 660.0] {
        let mut reference: Option<f64> = None;
        for (splay, radius, ..) in CROWN_TABLE {
            if radius != row {
                continue;
            }
            let length = (geo.crown(splay) - geo.anchor_at(splay)).norm();
            match reference {
                None => reference = Some(length),
                Some(r) => assert!(
                    (length - r).abs() < 1e-9,
                    "couronne {row} : barre de {length} mm à {splay}°, {r} mm ailleurs"
                ),
            }
        }
        assert!(reference.is_some(), "aucun cran sur la couronne {row}");
    }
}


/// §1 — la barre déclarée doit tomber sur les trous que la jonction lui impose.
/// Les avertissements restants sont ceux qu'on accepte de vivre : ils doivent
/// être nommés ici, sinon une dérive de cotation passerait pour normale.
#[test]
fn the_declared_rear_bar_fits_the_joint_it_serves() {
    let warnings = sa303_core::speaker::check_rear_bar(&sa303())
        .expect("la barre déclarée doit desservir les deux couronnes");
    for w in &warnings {
        println!(
            "{} : attendu {:.3}, déclaré {:.3}, écart {:+.3} mm",
            w.what, w.expected_mm, w.declared_mm, w.delta_mm
        );
        // Tout ce qui subsiste reste sous le demi-millimètre : au-delà ce ne
        // serait plus une question de cotation.
        assert!(w.delta_mm.abs() < 0.5, "{} : {:+.3} mm", w.what, w.delta_mm);
    }
}

/// Une barre qui ne tombe franchement pas sur ses trous n'est pas un
/// avertissement : il n'existe pas de barre droite qui monte.
#[test]
fn a_bar_that_misses_its_holes_is_an_error_not_a_warning() {
    let mut sm = sa303();
    sm.mechanical.rear_bar.anchor_hole_at += 5.0;
    assert!(sa303_core::speaker::check_rear_bar(&sm).is_err());
}

/// Les distances au bord sous le minimum EN 1993-1-8 ressortent, sans bloquer.
#[test]
fn an_edge_distance_below_the_minimum_is_reported() {
    let mut sm = sa303();
    // Toute la barre glisse vers le bout couronne : les entraxes ne bougent
    // pas, seule la distance au bord se referme.
    let shift = sm.mechanical.rear_bar.crown_hole_outer_at - 5.0;
    sm.mechanical.rear_bar.crown_hole_outer_at -= shift;
    sm.mechanical.rear_bar.crown_hole_inner_at -= shift;
    sm.mechanical.rear_bar.latch_hole_at -= shift;
    sm.mechanical.rear_bar.anchor_hole_at -= shift;
    let warnings = sa303_core::speaker::check_rear_bar(&sm).expect("pas bloquant");
    assert!(
        warnings.iter().any(|w| w.what.starts_with("e1 bout couronne")),
        "{warnings:?}"
    );
}


/// §7 — un bras de bielle nul doit remonter une erreur, pas un NaN. Les
/// comparaisons sur NaN étant fausses, un NaN traverserait tous les seuils de
/// vérification sans en déclencher un seul : la grappe passerait pour bonne.
#[test]
fn a_degenerate_bielle_lever_is_an_error_not_a_silent_nan() {
    use sa303_core::cluster::{build_cluster, compute_joint, ChainSpeaker, Compartment, JointInput};

    let mut sm = sa303();
    // On amène la couronne sur la ligne d'action de la bielle : au splay 0
    // celle-ci est verticale sous `hb`, donc un rayon nul y place la goupille
    // de couronne exactement.
    sm.mechanical.crown.radius = 0.0;
    sm.mechanical.crown.delta = 0.0;
    let chain = vec![ChainSpeaker::from_model(&sm); 2];
    let splays = [0.0];
    let speakers = build_cluster(&chain, &splays, 0.0);
    let err = compute_joint(&JointInput {
        chain: &chain,
        speakers: &speakers,
        splays_deg: &splays,
        joint_index: 0,
        compartment: Compartment::Flown,
        g: 9.80665,
        k_dyn: 1.3,
        share_per_flank: 0.5,
        tie: None,
        recommended_splay: None,
    })
    .expect_err("bras nul : la jonction n'a pas de solution");
    assert!(err.reason.contains("bras de bielle"), "{}", err.reason);
}

/// §10 — valeurs de référence recoupées à la main sur une grappe de trois
/// SA303, vol, φ₀ = 0, splays [5°, 10°], jonction 0.
///
/// Passe par `compute_joint` directement plutôt que par `compute_cluster` : ce
/// dernier impose un bumper et une pendaison libre, donc un φ₀ qui n'est pas
/// zéro. Les valeurs ci-dessous ne vaudraient plus.
#[test]
fn golden_bar_loads_on_the_reference_joint() {
    use sa303_core::checks::check_bar;
    use sa303_core::cluster::{build_cluster, compute_joint, ChainSpeaker, Compartment, JointInput};

    let sm = sa303();
    let splays = [5.0, 10.0];
    let chain = vec![ChainSpeaker::from_model(&sm); 3];
    let speakers = build_cluster(&chain, &splays, 0.0);
    let j = compute_joint(&JointInput {
        chain: &chain,
        speakers: &speakers,
        splays_deg: &splays,
        joint_index: 0,
        compartment: Compartment::Flown,
        g: 9.80665,
        k_dyn: 1.3,
        share_per_flank: 0.5,
        tie: None,
        recommended_splay: None,
    })
    .expect("jonction résoluble");

    // Efforts totaux sur le corps libre (l'inverse du `sg = −0,5` de sortie).
    let total = |v: sa303_core::vector::Vec2| v * -2.0;
    let f_bielle = total(j.f_pivot);
    let f_couronne = total(j.f_orientation);
    assert!((f_bielle.x - -30.966).abs() < 0.01, "{f_bielle:?}");
    assert!((f_bielle.y - 709.247).abs() < 0.01, "{f_bielle:?}");
    assert!((f_couronne.x - 30.966).abs() < 0.01, "{f_couronne:?}");
    assert!((f_couronne.y - 1424.749).abs() < 0.01, "{f_couronne:?}");

    // Repère de barre. L = 324.756 mm, l'entraxe couronne-ancrage de la
    // couronne intérieure : le splay 5 est impair.
    let l = (j.anchor_hole_local - j.crown_hole_local).norm();
    assert!((l - 324.8).abs() < 0.1, "L = {l}");

    // Décomposition, par flanc. Axial négatif = barre tendue.
    assert!((j.bar_axial_n - -700.0).abs() < 1.0, "N = {}", j.bar_axial_n);
    assert!((j.bar_shear_n.abs() - 132.0).abs() < 1.0, "V = {}", j.bar_shear_n);

    // Moment. Le maximum est au verrou — première goupille depuis la couronne —
    // et non à l'ancrage : au-delà du verrou la réaction de la paire le fait
    // redescendre. Réduit à l'ancrage il vaudrait 42,85 N·m par flanc.
    assert_eq!(j.bar_moment_max_at_mm, sm.mechanical.rear_bar.latch_hole_at);
    assert!(
        (j.bar_moment_max_nm - 39.82).abs() < 0.05,
        "M_max = {}",
        j.bar_moment_max_nm
    );
    assert!(
        (j.bar_moment_at_pair_nm - 41.33).abs() < 0.05,
        "M_paire = {}",
        j.bar_moment_at_pair_nm
    );

    // Paire : entraxe 23,7 mm, couple issu du moment réduit au barycentre.
    let d = (j.anchor_hole_local - j.latch_hole_local).norm();
    assert!((d - 23.735).abs() < 0.01, "d = {d}");
    let couple = j.bar_moment_at_pair_nm * 1000.0 / d;
    assert!((couple - 1741.0).abs() < 5.0, "couple = {couple}");
    assert!((j.f_anchor_n - 1910.0).abs() < 20.0, "F_a = {}", j.f_anchor_n);
    assert!((j.f_latch_n - 1840.0).abs() < 20.0, "F_v = {}", j.f_latch_n);

    // Contraintes de barre. Section étroite 40×10 percée Ø12,08 au verrou.
    let check = check_bar(
        &sm.mechanical.rear_bar,
        j.splay_deg,
        j.bar_shear_n,
        j.bar_axial_n,
        4.0,
    );
    let latch = check.sections.iter().find(|x| x.location == "verrou").unwrap();
    assert_eq!(latch.width_mm, 40.0);
    assert!((latch.stress_mpa - 17.9).abs() < 0.3, "σ = {}", latch.stress_mpa);
    let transition = check
        .sections
        .iter()
        .find(|x| x.location.starts_with("transition"))
        .unwrap();
    assert!(
        (transition.stress_mpa - 8.21).abs() < 0.05,
        "σ transition = {}",
        transition.stress_mpa
    );
}

/// Loi d'échelle, pour attraper une unité perdue : la contrainte est linéaire
/// en effort, donc 1 kN de transverse par flanc doit rendre exactement le
/// produit du bras par le module de section.
#[test]
fn golden_bar_stress_scales_with_a_kilonewton_of_shear() {
    use sa303_core::checks::check_bar;
    let bar = sa303().mechanical.rear_bar;

    // Couronne intérieure (splay impair) : bras verrou = 301,81 mm.
    let check = check_bar(&bar, 5.0, 1000.0, 0.0, 4.0);
    let latch = check.sections.iter().find(|x| x.location == "verrou").unwrap();
    let arm = bar.latch_hole_at - bar.crown_hole_inner_at;
    assert!((arm - 301.81).abs() < 0.01, "bras = {arm}");
    assert!((latch.moment_nmm / 1000.0 - 301.81).abs() < 0.01);
    // 301,81 N·m sur W_net = 2593,22 mm³.
    assert!(
        (latch.stress_mpa - 116.4).abs() < 0.5,
        "σ = {}",
        latch.stress_mpa
    );
}

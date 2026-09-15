//! Tests non négociables de la liaison par bielle (brief §8). Ils portent sur
//! la cinématique seule — pas sur les charges — parce que c'est elle qui a
//! changé : le pivot avant n'est plus un point fixe du caisson, et les huit
//! trous de couronne ne sont plus sur un arc centré sur un point unique.

use sa303_core::speaker::{
    BarHole, BarHoles, BelowCompatibility, Crown, Hinge, PolarHole, RearBar, SpeakerGeometry,
    SpeakerMechanicalModel, SpeakerModel,
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
                splay0_angle: 5.0,
            },
            latch: PolarHole {
                radius: 710.845,
                angle_deg: -20.8453,
            },
            anchor: PolarHole {
                radius: 680.0,
                angle_deg: -13.0,
            },
            latch_offset: 100.0,
            splay_grid: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 20.0],
            frame_hole_splay: 0.0,
            rear_bar: RearBar {
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
    sm.mechanical.rear_bar.holes.anchor.0[0] += 5.0;
    assert!(sa303_core::speaker::check_rear_bar(&sm).is_err());
}

/// Les distances au bord sous le minimum EN 1993-1-8 ressortent, sans bloquer.
#[test]
fn an_edge_distance_below_the_minimum_is_reported() {
    let mut sm = sa303();
    // La barre est rallongée côté couronne sans que le trou suive : les
    // entraxes ne bougent pas, seule la distance au bord se referme.
    sm.mechanical.rear_bar.length = sm.mechanical.rear_bar.holes.up680.along() + 5.0;
    sm.mechanical.rear_bar.step_position =
        sm.mechanical.rear_bar.length - sm.mechanical.rear_bar.wide_length;
    let warnings = sa303_core::speaker::check_rear_bar(&sm).expect("pas bloquant");
    assert!(
        warnings
            .iter()
            .any(|w| w.what.starts_with("e1 bout couronne")),
        "{warnings:?}"
    );
}

/// §7 — un bras de bielle nul doit remonter une erreur, pas un NaN. Les
/// comparaisons sur NaN étant fausses, un NaN traverserait tous les seuils de
/// vérification sans en déclencher un seul : la grappe passerait pour bonne.
#[test]
fn a_degenerate_bielle_lever_is_an_error_not_a_silent_nan() {
    use sa303_core::cluster::{
        build_cluster, compute_joint, ChainSpeaker, Compartment, JointInput,
    };

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
    use sa303_core::cluster::{
        build_cluster, compute_joint, ChainSpeaker, Compartment, JointInput,
    };

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
    assert!(
        (j.bar_axial_n - -700.0).abs() < 1.0,
        "N = {}",
        j.bar_axial_n
    );
    assert!(
        (j.bar_shear_n.abs() - 132.0).abs() < 1.0,
        "V = {}",
        j.bar_shear_n
    );
}

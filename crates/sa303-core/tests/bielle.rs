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
                x: -338.433,
                y: 257.127,
                joint_separation: 552.379,
                edge_perp: 12.569,
            },
            crown: Crown {
                radius: 680.0,
                delta: 20.0,
                splay0_angle: 5.0,
                // Perçage relevé : rangée intérieure sur 1, 3 et 5°.
                inner_splays: vec![1.0, 3.0, 5.0],
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
            splay_grid: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.5, 20.0],
            frame_hole_splay: 0.0,
            rear_face_x: 351.0,
            rear_bar: RearBar {
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
    let short = sm.mechanical.rear_bar.holes.up680.along() + 5.0;
    sm.mechanical.rear_bar.length = short;
    // Le profil suit la nouvelle longueur, sinon il ne couvrirait plus la barre.
    let last = sm.mechanical.rear_bar.width_profile.len() - 1;
    sm.mechanical.rear_bar.width_profile[last][0] = short;
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
    use sa303_core::cluster::{
        build_cluster, compute_joint, ChainSpeaker, Compartment, JointInput,
    };

    let sm = sa303();
    let splays = [5.0, 10.5];
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
    assert!((f_bielle.x - -30.860).abs() < 0.01, "{f_bielle:?}");
    assert!((f_bielle.y - 706.822).abs() < 0.01, "{f_bielle:?}");
    assert!((f_couronne.x - 30.860).abs() < 0.01, "{f_couronne:?}");
    assert!((f_couronne.y - 1427.1731).abs() < 0.01, "{f_couronne:?}");

    // Repère de barre. L = 324.756 mm, l'entraxe couronne-ancrage de la
    // couronne intérieure : le splay 5 est impair.
    let l = (j.anchor_hole_local - j.crown_hole_local).norm();
    assert!((l - 324.8).abs() < 0.1, "L = {l}");

    // Décomposition dans le repère de barre, par flanc. Axial négatif = tendue.
    //
    // N et V ont bougé par rapport à la géométrie précédente, et c'est
    // attendu : l'axe de barre n'est plus la droite couronne-ancrage mais la
    // droite ancrage-`up680`. Au splay 5, impair, la couronne utilisée est
    // `up660`, déportée de 19,4 mm — les deux droites font donc 3,43° entre
    // elles, et la décomposition tourne d'autant. |F| ne bouge pas.
    assert!(
        (j.bar_axial_n - -708.065).abs() < 0.01,
        "N = {}",
        j.bar_axial_n
    );
    assert!(
        (j.bar_shear_n - -89.932).abs() < 0.01,
        "V = {}",
        j.bar_shear_n
    );
    // La résultante, elle, est invariante : c'est bien la même force, lue dans
    // un repère qui a tourné.
    let f_norm = (j.bar_axial_n.powi(2) + j.bar_shear_n.powi(2)).sqrt();
    assert!((f_norm - 713.75).abs() < 0.02, "|F| = {f_norm}");

    // Moment. La section critique est l'**ancrage**, premier pion depuis la
    // couronne. Le moment y est un invariant de repère — c'est |F| fois la
    // distance de l'ancrage à la ligne d'action — donc il ne bouge pas avec
    // l'axe, contrairement à N et V.
    assert_eq!(
        j.bar_moment_max_at_mm,
        sm.mechanical.rear_bar.holes.anchor.along()
    );
    assert!(
        (j.bar_moment_max_nm - 42.894).abs() < 0.01,
        "M_ancrage = {}",
        j.bar_moment_max_nm
    );

    // Paire : entraxe 100 mm au lieu de 23,7, donc un couple quatre fois plus
    // petit pour un moment comparable.
    let d = (j.anchor_hole_local - j.latch_hole_local).norm();
    assert!((d - 100.0).abs() < 0.001, "d = {d}");
    let couple = j.bar_moment_at_pair_nm.abs() * 1000.0 / d;
    assert!((couple - 473.4).abs() < 1.0, "couple = {couple}");
    assert!(
        (j.f_anchor_n - 627.4).abs() < 1.0,
        "F_ancrage = {}",
        j.f_anchor_n
    );
    assert!(
        (j.f_latch_n - 555.4).abs() < 1.0,
        "F_verrou = {}",
        j.f_latch_n
    );

    // Les deux branches du diagramme se raccordent à l'ancrage : c'est
    // l'équilibre de la barre, donc le contrôle que la paire est bien résolue.
    let check = sa303_core::checks::check_bar(
        &sm.mechanical.rear_bar,
        j.row,
        &j.bar_loads(),
        4.0,
        Some(j.rear_face_x),
        Some(j.bar_rear_edge_max_x),
    );
    assert!(
        check.moment_continuity_nmm.abs() < sa303_core::checks::MOMENT_CONTINUITY_TOLERANCE_NMM,
        "raccord à l'ancrage : {} N·mm",
        check.moment_continuity_nmm
    );
    // La section critique est l'ancrage : percé, dans la largeur 70.
    // L'abscisse critique est celle de l'ancrage calculé, à quelques microns de
    // la cotation déclarée — c'est l'écart d'ajustement, pas une dérive.
    assert!((check.critical.at_mm - 116.0).abs() < 0.001);
    assert!(check.critical.drilled);
    assert!((check.critical.width_mm - 70.0).abs() < 1e-9);
    assert!(
        (check.critical.moment_nmm / 1000.0 - 42.89).abs() < 0.01,
        "M ancrage = {}",
        check.critical.moment_nmm / 1000.0
    );

    // Moment nul au verrou, et à mi-paire il vaut la moitié : le tronçon
    // verrou-ancrage ne porte qu'un effort, donc le diagramme y est droit.
    let m_at = |a: f64| {
        check
            .profile
            .iter()
            .min_by(|x, y| (x.at_mm - a).abs().total_cmp(&(y.at_mm - a).abs()))
            .unwrap()
            .moment_nmm
    };
    assert!(m_at(16.0) < 1.0, "M verrou = {}", m_at(16.0));
    assert!(
        (m_at(66.0) / 1000.0 - 21.4).abs() < 0.3,
        "M à mi-paire = {}",
        m_at(66.0) / 1000.0
    );

    // Aucun avertissement géométrique sur la pièce livrée.
    assert!(check.warnings.is_empty(), "{:?}", check.warnings);
}

/// §4 — le bord arrière de la barre doit rester devant la face arrière du
/// caisson. La marge se mesure au **petit bout** et non à l'ancrage : le bord
/// arrière est parallèle à l'axe de barre, qui s'incline vers l'avant en
/// montant, donc son abscisse recule à mesure qu'on s'éloigne de la paire.
#[test]
fn the_bar_rear_edge_stays_inside_the_cabinet() {
    use sa303_core::cluster::{
        build_cluster, compute_joint, ChainSpeaker, Compartment, JointInput,
    };
    let sm = sa303();
    let chain = vec![ChainSpeaker::from_model(&sm); 2];

    for splay in [0.0, 5.0, 20.0] {
        let splays = [splay];
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

        let margin = j.rear_face_x - j.bar_rear_edge_max_x;
        assert!(
            margin > 0.0,
            "splay {splay}° : la barre dépasse de {:.3} mm derrière le caisson",
            -margin
        );
        // Le splay ne fait pas bouger la barre par rapport au caisson qui la
        // porte : elle est goupillée dessus, elle tourne avec lui.
        assert!(
            (margin - 4.84).abs() < 0.02,
            "splay {splay}° : marge {margin:.3} mm, attendue 4,84"
        );
    }
}

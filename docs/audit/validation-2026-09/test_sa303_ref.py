"""Tests du solveur de référence. Exécution : python3 test_sa303_ref.py
(sans dépendance à pytest ; chaque test est une fonction `test_*`).

Les cas « à la main » reprennent les nombres posés dans le rapport §3.3 :
ils sont calculés ici avec des formules fermées, sans passer par la matrice.
"""
import copy
import json
import math
import os

import numpy as np

from sa303_ref import (
    SpeakerGeo,
    axial_share_with_gap,
    bar_pair_spring_model,
    build_chain,
    cluster_cg,
    cross,
    huth_stiffness,
    inputs_from_export,
    norm,
    polar,
    rot,
    solve_cluster,
    unit,
    v,
)

HERE = os.path.dirname(os.path.abspath(__file__))
DOC = json.load(open(os.path.join(HERE, "corpus-definitions.json")))
S = DOC["settings"]


def cluster_named(name):
    c = next(c for c in DOC["clusters"] if c["name"] == name)
    return inputs_from_export(DOC, {"definition": c})


def approx(a, b, tol=1e-6, rel=1e-9):
    return abs(a - b) <= max(tol, rel * max(abs(a), abs(b)))


# ---------------------------------------------------------------------------
# Géométrie
# ---------------------------------------------------------------------------


def test_geometry_closes_on_the_bar_cotation():
    """Les trous de couronne obtenus par la polaire du caisson et par la
    cotation de la barre coïncident à mieux que 0,01 mm (tolérance de montage
    0,05 mm), pour toutes les rangées et tous les splays."""
    inp = cluster_named("iso4 droit free")
    g = inp.geos[0]
    for s in g.splay_grid:
        gap = norm(g.crown_polar(s) - g.crown_from_bar(s, g))
        assert gap < 0.01, (s, gap)


def test_bumper_pins_close_on_the_drawing_span():
    """Bielle (71,6) + barre bumper (224,626 / 1,822) : entraxe des deux pions
    égal à la cote du plan (702 − 12,567 − 32,604 = 656,829 mm) à 2 µm, et
    les deux pions sur une même horizontale du repère caisson."""
    inp = cluster_named("iso4 droit free")
    g = inp.geos[0]
    bm = inp.bumper
    rb = bm["rearBars"][0]
    pvg = g.ht + v(0, bm["pivotBarUsableLengthMm"])
    top = g.anchor + rb["topHoleAlongMm"] * g.e_axis + rb["topHoleLateralMm"] * g.e_front
    span = norm(top - pvg)
    assert abs(span - (bm["depth"] - bm["pins"]["frontFromFrontMm"] - bm["pins"]["rearFromRearMm"])) < 0.005
    assert abs(top[1] - pvg[1]) < 0.01
    # mais la hauteur des pions au-dessus de la face supérieure n'est pas
    # `heightFromBottomMm` : écart connu de 6,3 mm (question Q1 du rapport)
    assert abs((pvg[1] - g.height / 2) - bm["pins"]["heightFromBottomMm"]) > 6.0


def test_front_faces_stay_aligned_at_20_degrees():
    """Partage angulaire égal : recul des faces avant < 0,9 mm à 20°."""
    inp = cluster_named("iso4 droit free")
    g = inp.geos[0]
    s = 20.0
    pb = g.pv(s)
    corner_lower = pb + rot(v(g.hb[0] - 12.569, g.ht[1] + g.bielle_L / 2) - g.ht, math.radians(s))
    front_mm = corner_lower[0] - (g.hb[0] - 12.569)
    assert abs(front_mm) < 0.9, front_mm


# ---------------------------------------------------------------------------
# Cas à la main
# ---------------------------------------------------------------------------


def test_one_cabinet_by_hand():
    """1 caisson en pendaison libre : le pickup est à l'aplomb du CG ; la
    bielle et la barre du bumper tiennent le caisson. Vérification en fermant
    les équations à la main (moment autour du trou haut de barre)."""
    inp = cluster_named("iso1 droit free")
    r = solve_cluster(inp)
    g = inp.geos[0]
    W = g.mass_kg * S["gravity"] * S["dynamicFactor"]
    p = r.placed[0]
    cg = p.to_global(g.cg)
    # pickup à l'aplomb du CG
    assert abs(r.pickup_global[0] - cg[0]) < 1e-9
    # moment autour du trou haut de barre : W × bras = λ × bras de bielle
    b = r.bumper
    u = unit(b["bl"] - b["pvg"])  # direction de la bielle, du bumper vers le caisson
    lever = cross(b["bl"] - b["an"], u)
    m_w = cross(cg - b["an"], v(0, -W))
    lam = -m_w / lever
    assert approx(abs(lam), abs(b["lamB"]), tol=1e-6)
    assert approx(norm(b["f_pivot_g"]), abs(lam) * S["sharePerFlank"])
    # équilibre des forces : f_ori + f_piv = W
    f_ori = -v(0, -W) - u * lam
    assert approx(norm(b["f_orientation_g"]), norm(f_ori) * S["sharePerFlank"])
    # la manille reprend exactement le poids dynamisé
    assert approx(norm(b["support"]), W)


def test_two_cabinets_by_hand():
    """2 caissons : la jonction porte le caisson du bas seul ; λ = −M_ext/bras."""
    inp = cluster_named("iso2 droit free")
    r = solve_cluster(inp)
    g = inp.geos[1]
    W = g.mass_kg * S["gravity"] * S["dynamicFactor"]
    j = r.joints[0]
    cg1 = r.placed[1].to_global(g.cg)
    u = unit(j.pb - j.pa)
    lever = cross(j.pb - j.bo, u)
    m_ext = cross(cg1 - j.bo, v(0, -W))
    lam = -m_ext / lever
    assert approx(lam, j.lambda_bielle)
    f_ori = v(0, W) - u * lam
    assert approx(norm(j.f_orientation_g), norm(f_ori) * S["sharePerFlank"])
    # paire : F/2 ± M_G/d, avec d = 100 mm
    G = (j.an + j.lt) / 2
    m_g = cross(j.bo - G, f_ori)
    d = norm(j.an - j.lt)
    assert approx(d, 100.0, tol=1e-3)
    e = unit(j.an - j.lt)
    perp = np.array([-e[1], e[0]])
    fa = (f_ori / 2 + perp * m_g / d) * S["sharePerFlank"]
    fl = (f_ori / 2 - perp * m_g / d) * S["sharePerFlank"]
    assert approx(norm(fa), norm(j.f_anchor_g)) and approx(norm(fl), norm(j.f_latch_g))
    # le couple des deux pions reproduit exactement le moment réduit
    assert approx(cross(j.an - G, fa) + cross(j.lt - G, fl), m_g * S["sharePerFlank"], tol=1e-6)


# ---------------------------------------------------------------------------
# Propriétés du système
# ---------------------------------------------------------------------------


def test_system_is_square_and_full_rank_in_flight():
    """Avec tirette : système carré de plein rang. En pendaison libre ou
    accroche dans la portée de barre : une équation de plus que d'inconnues
    (le point d'accroche a été résolu en amont par la condition CG sous le
    pickup) ; elle doit être satisfaite à la précision machine, c'est le
    contrôle de fermeture de cette condition."""
    for name, square in (("iso1 droit free", False), ("J 12 free", False), ("J14 tilt -6 sans tirette", False), ("J14 tilt +25 tie180", True)):
        r = solve_cluster(cluster_named(name))
        st = r.statics
        assert st.rank == st.n_unknowns, (name, st.n_eq, st.n_unknowns, st.rank)
        assert st.n_eq == st.n_unknowns + (0 if square else 1), (name, st.n_eq, st.n_unknowns)
        for body, (f, m) in st.body_residuals().items():
            assert f < 1e-6 and m < 1e-3, (name, body, f, m)


def test_bielle_force_is_collinear_and_sign_is_reported():
    r = solve_cluster(cluster_named("J14 tilt +25 tie180"))
    for j in r.joints:
        u = unit(j.pb - j.pa)
        assert abs(cross(u, j.f_pivot_g)) < 1e-9 * max(1.0, norm(j.f_pivot_g))
    # la jonction 0 de cette grappe a sa bielle en compression
    assert r.joints[0].lambda_bielle > 0


def test_tie_zero_equals_free_hang():
    inp = cluster_named("J 12 free")
    r0 = solve_cluster(inp)
    inp2 = copy.deepcopy(inp)
    inp2.imposed_tilt = math.degrees(r0.phi_free_hang)
    r1 = solve_cluster(inp2)
    assert abs(r1.pickup_local[0]) < 1e-9
    assert r1.tie_tension == 0.0
    for a, b in zip(r0.joints, r1.joints):
        assert approx(norm(a.f_anchor_g), norm(b.f_anchor_g), tol=1e-6)


def test_linearity_and_superposition():
    inp = cluster_named("J 12 free")
    inp.imposed_tilt = -1.9
    ra = solve_cluster(inp)
    inpb = copy.deepcopy(inp)
    for g in inpb.geos:
        g.mass_kg *= 2
    rb = solve_cluster(inpb)
    for a, b in zip(ra.joints, rb.joints):
        assert approx(norm(b.f_anchor_g), 2 * norm(a.f_anchor_g), tol=1e-6)
    base = copy.deepcopy(inp)
    for g in base.geos:
        g.mass_kg = 1e-9
    tot = None
    for k in range(len(inp.geos)):
        one = copy.deepcopy(base)
        one.geos[k].mass_kg = inp.geos[k].mass_kg
        vals = np.array([np.concatenate([j.f_orientation_g, j.f_anchor_g]) for j in solve_cluster(one).joints])
        tot = vals if tot is None else tot + vals
    full = np.array([np.concatenate([j.f_orientation_g, j.f_anchor_g]) for j in ra.joints])
    assert np.abs(tot - full).max() < 1e-5


def test_pickup_balances_weight_and_tie():
    r = solve_cluster(cluster_named("J14 tilt +25 tie180"))
    W = r.mass * S["gravity"] * S["dynamicFactor"]
    tie = r.tie_dir * r.tie_tension
    assert norm(r.bumper["support"] - (v(0, W) - tie)) < 1e-6
    assert abs(cross(r.cg - r.pickup_global, v(0, -W)) + cross(r.tie_point_global - r.pickup_global, tie)) < 1e-4


def test_invalid_configurations_are_refused():
    for name in ("INVALIDE splay 7 hors grille", "INVALIDE tirette en poussée (0°)", "INVALIDE tirette en poussée (90°)"):
        try:
            solve_cluster(cluster_named(name))
        except ValueError:
            continue
        raise AssertionError(name + " aurait dû être refusée")


def test_axial_share_rigid_vs_gap():
    """Modèle ressort : à jeu nul et raideurs égales, retombe sur le 50/50 ; un
    jeu de 20 µm suffit à mettre tout l'axial sur un seul pion."""
    k = huth_stiffness()
    assert axial_share_with_gap(5000, 0.0, k) == (2500, 2500)
    f1, f2 = axial_share_with_gap(5000, 0.02, k)
    assert f1 == 5000 and f2 == 0


def test_spring_model_converges_to_rigid_split():
    r = solve_cluster(cluster_named("J14 tilt +25 tie180"))
    j = r.joints[0]
    Fc = -j.f_orientation_g / S["sharePerFlank"]
    e = unit(j.an - j.lt)
    ef = np.array([-e[1], e[0]])
    tb = lambda p: np.array([np.dot(p - j.lt, e), np.dot(p - j.lt, ef)])
    Fb = np.array([np.dot(Fc, e), np.dot(Fc, ef)])
    k = huth_stiffness()
    for kax, ktr in ((k, k), (k * 10, k), (k, k * 10), (1e3, 1e3)):
        ra, rl = bar_pair_spring_model(Fb, tb(j.bo), tb(j.an), tb(j.lt), kax, ktr)
        assert approx(norm(ra) * 0.5, norm(j.f_anchor_g), tol=1e-3)
        assert approx(norm(rl) * 0.5, norm(j.f_latch_g), tol=1e-3)
    ra, rl = bar_pair_spring_model(Fb, tb(j.bo), tb(j.an), tb(j.lt), k, k, gap_latch=0.05)
    assert approx(norm(ra) * 0.5, j.f_anchor_env, tol=1e-3)


if __name__ == "__main__":
    import sys

    failed = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            try:
                fn()
                print("ok  ", name)
            except Exception as e:  # noqa: BLE001
                failed += 1
                print("FAIL", name, repr(e))
    sys.exit(1 if failed else 0)

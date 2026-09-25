"""Compare champ par champ un export d'audit (solveur Rust) au solveur de
référence Python, sur toutes les grappes de l'export.

Usage : python3 compare.py corpus-export.json [autres exports...]
Sorties : comparison.md (tableau des écarts max par champ) et
          corpus-reference.json (résultats de référence par grappe).
"""
import json
import math
import sys

import numpy as np

from sa303_ref import (
    ClusterInput,
    angle_deg_sec2,
    cross,
    inputs_from_export,
    norm,
    solve_cluster,
    v,
)


def vec(d):
    return v(d["x"], d["y"])


class Diff:
    def __init__(self):
        self.max = {}  # field -> (abs diff, rel diff, cluster, joint)

    def add(self, field, ref, code, cluster, joint=None):
        if isinstance(ref, np.ndarray):
            a = float(norm(ref - code))
            scale = max(norm(ref), norm(code), 1e-9)
        else:
            a = abs(float(ref) - float(code))
            scale = max(abs(float(ref)), abs(float(code)), 1e-9)
        r = a / scale
        cur = self.max.get(field)
        if cur is None or a > cur[0]:
            self.max[field] = (a, r, cluster, joint, float(norm(ref)) if isinstance(ref, np.ndarray) else float(ref), float(norm(code)) if isinstance(code, np.ndarray) else float(code))

    def table(self):
        lines = ["| Champ | écart max abs | écart rel | grappe | jonction | réf | code |", "|---|---|---|---|---|---|---|"]
        for f in sorted(self.max):
            a, r, c, j, rv, cv = self.max[f]
            lines.append(f"| `{f}` | {a:.3e} | {r:.2e} | {c} | {'' if j is None else j} | {rv:.6g} | {cv:.6g} |")
        return "\n".join(lines)


def compare_cluster(doc, c, diff: Diff, reference_out: list, findings: list):
    name = c["definition"]["name"]
    inp = inputs_from_export(doc, c)
    ref = solve_cluster(inp)
    res = c["result"]
    S = doc["settings"]

    # --- grappe --------------------------------------------------------------
    diff.add("phiInitial (rad)", ref.phi0, res["phiInitial"], name)
    if ref.phi_free_hang is not None and res["phiFreeHang"] is not None:
        diff.add("phiFreeHang (rad)", ref.phi_free_hang, res["phiFreeHang"], name)
    diff.add("tieTensionN", ref.tie_tension, res["tieTensionN"], name)
    diff.add("cg (mm)", ref.cg, vec(res["cg"]), name)
    diff.add("totalMassKg", ref.mass, res["totalMassKg"], name)
    for i, p in enumerate(ref.placed):
        diff.add("speakers[].o (mm)", p.o, vec(res["speakers"][i]["o"]), name, i)
        diff.add("speakers[].phi (rad)", p.phi, res["speakers"][i]["phi"], name, i)
    if ref.pickup_global is not None:
        diff.add("pickupGlobal (mm)", ref.pickup_global, vec(res["pickupGlobal"]), name)
    if ref.tie_point_global is not None and res["tiePointGlobal"] is not None:
        diff.add("tiePointGlobal (mm)", ref.tie_point_global, vec(res["tiePointGlobal"]), name)
        diff.add("tieDirectionGlobal", ref.tie_dir, vec(res["tieDirectionGlobal"]), name)
        bv = res["bumperView"]
        if bv["tieAngleRangeDeg"] is not None:
            diff.add("tieAngleRangeDeg[0]", ref.tie_range[0], bv["tieAngleRangeDeg"][0], name)
            diff.add("tieAngleRangeDeg[1]", ref.tie_range[1], bv["tieAngleRangeDeg"][1], name)

    weight = ref.mass * S["gravity"] * S["dynamicFactor"]

    # --- jonctions -------------------------------------------------------------
    ref_joints = []
    for j, rj in zip(res["joints"], ref.joints):
        i = rj.index
        share = S["sharePerFlank"]
        diff.add("row", 0.0, 0.0 if j["row"] == rj.row else 1.0, name, i)
        diff.add("fOrientationN", norm(rj.f_orientation_g), j["fOrientationN"], name, i)
        diff.add("fPivotN", norm(rj.f_pivot_g), j["fPivotN"], name, i)
        diff.add("fAnchorN", norm(rj.f_anchor_g), j["fAnchorN"], name, i)
        diff.add("fLatchN", norm(rj.f_latch_g), j["fLatchN"], name, i)
        diff.add("fOrientationGlobal (N)", rj.f_orientation_g, vec(j["fOrientationGlobal"]), name, i)
        diff.add("fPivotGlobal (N)", rj.f_pivot_g, vec(j["fPivotGlobal"]), name, i)
        diff.add("fAnchorGlobal (N)", rj.f_anchor_g, vec(j["fAnchorGlobal"]), name, i)
        diff.add("fLatchGlobal (N)", rj.f_latch_g, vec(j["fLatchGlobal"]), name, i)
        diff.add("barAxialN", rj.bar_axial, j["barAxialN"], name, i)
        diff.add("barShearN", rj.bar_shear, j["barShearN"], name, i)
        diff.add("barMomentAtPairNm", rj.bar_moment_at_pair_nm, j["barMomentAtPairNm"], name, i)
        diff.add("barMomentMaxNm", rj.bar_moment_anchor_nm, j["barMomentMaxNm"], name, i)
        diff.add("anchorHoleGlobal (mm)", rj.an, vec(j["anchorHoleGlobal"]), name, i)
        diff.add("latchHoleGlobal (mm)", rj.lt, vec(j["latchHoleGlobal"]), name, i)
        # trous en repère local du flanc chargé : couronne et pivot
        ti = j["loadedFlank"]
        p = ref.placed[ti]
        if c["definition"]["compartment"] == "flown":
            diff.add("loadedOrientationHoleGlobal (mm)", rj.bo, vec(j["loadedOrientationHoleGlobal"]), name, i)
            diff.add("loadedPivotHoleGlobal (mm)", rj.pa, vec(j["loadedPivotHoleGlobal"]), name, i)
        # repère local : les efforts locaux doivent être la rotation des globaux
        f_ori_local = np.array([j["fOrientation"]["x"], j["fOrientation"]["y"]])
        from sa303_ref import rot
        diff.add("fOrientation (local) vs global tourné", rot(rj.f_orientation_g, -p.phi), f_ori_local, name, i)
        diff.add("fOrientationAngleDeg", angle_deg_sec2(rot(rj.f_orientation_g, -p.phi)) % 360, j["fOrientationAngleDeg"] % 360, name, i)
        # colinéarité bielle
        u = (rj.pb - rj.pa) / norm(rj.pb - rj.pa)
        col = abs(cross(u, vec(j["fPivotGlobal"]))) / max(norm(vec(j["fPivotGlobal"])), 1e-9)
        diff.add("colinéarité bielle |u×fPivot|/|fPivot|", 0.0, col, name, i)
        # hingeReversed = bielle en compression (λ > 0 avec u du haut vers le bas)
        diff.add("hingeReversed", 1.0 if rj.lambda_bielle > 0 else 0.0, 1.0 if j["hingeReversed"] else 0.0, name, i)
        # résidu de moment recalculé correctement (avec le point d'application de la tirette)
        diff.add("résidu force corps (N)", 0.0, rj.residual_force, name, i)
        diff.add("résidu moment corps (N·mm)", 0.0, rj.residual_moment, name, i)
        diff.add("fermeture ht(bas)=pv(haut) (mm)", 0.0, rj.closure_mm, name, i)
        diff.add("couronne polaire vs barre (mm)", 0.0, rj.crown_gap_mm, name, i)
        ref_joints.append(
            dict(
                jointIndex=i, splayDeg=rj.splay, row=rj.row,
                fOrientationN=norm(rj.f_orientation_g), fPivotN=norm(rj.f_pivot_g),
                fAnchorN=norm(rj.f_anchor_g), fLatchN=norm(rj.f_latch_g),
                fAnchorEnvN=rj.f_anchor_env, fLatchEnvN=rj.f_latch_env,
                barAxialN=rj.bar_axial, barShearN=rj.bar_shear,
                barMomentAtPairNm=rj.bar_moment_at_pair_nm, barMomentMaxNm=rj.bar_moment_anchor_nm,
                bielleCompression=rj.lambda_bielle > 0,
                momentResidualExportNmm=j["momentResidualNmm"],
            )
        )
        if j["momentResidualNmm"] > 1.0:
            findings.append((name, i, "momentResidualNmm export", j["momentResidualNmm"]))

    # --- bumper ------------------------------------------------------------------
    bv = res["bumperView"]
    bref = {}
    if c["definition"]["compartment"] == "flown":
        b = ref.bumper
        diff.add("bumper orientationForceN", norm(b["f_orientation_g"]), bv["orientationForceN"], name)
        diff.add("bumper pivotForceN", norm(b["f_pivot_g"]), bv["pivotForceN"], name)
        diff.add("bumper orientationForceGlobal", b["f_orientation_g"], vec(bv["orientationForceGlobal"]), name)
        diff.add("bumper pivotForceGlobal", b["f_pivot_g"], vec(bv["pivotForceGlobal"]), name)
        diff.add("bumper fPairAnchorN", norm(b["f_pair_anchor_g"]), bv["fPairAnchorN"], name)
        diff.add("bumper fPairLatchN", norm(b["f_pair_latch_g"]), bv["fPairLatchN"], name)
        diff.add("bumper fPairAnchorGlobal", b["f_pair_anchor_g"], vec(bv["fPairAnchorGlobal"]), name)
        diff.add("bumper pairMomentNm", b["pair_moment_nm"], bv["pairMomentNm"], name)
        diff.add("bumper supportForceN", norm(b["support"]), bv["supportForceN"], name)
        diff.add("bumper supportForceGlobal", b["support"], vec(bv["supportForceGlobal"]), name)
        diff.add("bumper pinPairMomentNm", b["pin_pair_moment_nm"], bv["pinPairMomentNm"], name)
        diff.add("bumper pinSpanMm", b["pin_span"], bv["pinSpanMm"], name)
        diff.add("bumper orientationPointGlobal (mm)", b["an"], vec(bv["orientationPointGlobal"]), name)
        diff.add("bumper pivotPointGlobal (mm)", b["pvg"], vec(bv["pivotPointGlobal"]), name)
        diff.add("bumper pairAnchorHoleGlobal (mm)", b["anchor"], vec(bv["pairAnchorHoleGlobal"]), name)
        # somme des efforts au pickup = poids dynamisé + tirette (signe : la manille tire)
        tie = ref.tie_dir * ref.tie_tension if ref.tie_dir is not None else v(0, 0)
        expected_support = v(0, weight) - tie
        diff.add("support = W·k − tirette", expected_support, vec(bv["supportForceGlobal"]), name)
        # moment nul de l'ensemble autour du pickup
        cgm = cross(ref.cg - ref.pickup_global, v(0, -weight))
        tm = cross(ref.tie_point_global - ref.pickup_global, tie) if ref.tie_dir is not None else 0.0
        diff.add("moment global autour du pickup (N·mm)", 0.0, cgm + tm, name)
        bref = dict(
            orientationForceN=norm(b["f_orientation_g"]), pivotForceN=norm(b["f_pivot_g"]),
            fPairAnchorN=norm(b["f_pair_anchor_g"]), fPairLatchN=norm(b["f_pair_latch_g"]),
            pairMomentNm=b["pair_moment_nm"], supportForceN=norm(b["support"]),
            pinPairMomentNm=b["pin_pair_moment_nm"], bielleCompression=b["lamB"] > 0,
        )
    else:
        # Stack : le code répartit 50/50 + couple sur deux pions cotés depuis les
        # bords du bumper. La référence rend le torseur exact bumper→caisson et
        # vérifie le non-basculement.
        G = ref.statics.get("G")
        diff.add("stack: réaction sol = W·k", weight, G[1], name)
        fb = ref.bumper["front_bottom"]
        # position de la résultante du sol par rapport au milieu de la face d'appui
        x_res = -G[2] / G[1] if abs(G[1]) > 0 else 0.0
        bref = dict(groundReactionN=float(G[1]), groundResultantOffsetMm=float(x_res), footprintHalfDepthMm=inp.bumper["depth"] / 2)
        diff.add("stack: supportForceN", weight, bv["supportForceN"], name)

    reference_out.append(dict(name=name, phiInitialDeg=math.degrees(ref.phi0), tieTensionN=ref.tie_tension,
                              pickupOffsetMm=None if ref.pickup_local is None else float(ref.pickup_local[0]),
                              rank=int(ref.statics.rank), unknowns=int(ref.statics.n_unknowns), equations=int(ref.statics.n_eq),
                              joints=ref_joints, bumper=bref, notes=ref.notes))


def main(paths):
    diff = Diff()
    reference = []
    findings = []
    n = 0
    for path in paths:
        doc = json.load(open(path))
        for c in doc["clusters"]:
            compare_cluster(doc, c, diff, reference, findings)
            n += 1
    with open("comparison.md", "w") as f:
        f.write(f"# Comparaison code Rust / référence Python\n\n{n} grappes, {sum(len(r['joints']) for r in reference)} jonctions.\n\n")
        f.write("Écart max sur tout le corpus, par champ. `réf`/`code` sont les valeurs (ou normes) là où l'écart est maximal.\n\n")
        f.write(diff.table())
        f.write("\n\n## Résidus de moment exportés > 1 N·mm\n\n")
        f.write("Le champ `momentResidualNmm` de l'export n'est pas un résidu d'équilibre dès qu'une tirette est active (voir rapport §2.6, correction C3).\n\n")
        f.write("| grappe | jonction | momentResidualNmm |\n|---|---|---|\n")
        for nme, i, _, val in findings[:400]:
            f.write(f"| {nme} | {i} | {val:.3e} |\n")
    json.dump(reference, open("corpus-reference.json", "w"), indent=1)
    print(diff.table())
    print(len(findings), "résidus de moment exportés > 1 N·mm")


if __name__ == "__main__":
    main(sys.argv[1:])

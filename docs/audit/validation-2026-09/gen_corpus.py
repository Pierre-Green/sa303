"""Construit le corpus de grappes (définitions au schéma des exports d'audit).

Les définitions d'enceintes, bumper et barre de déport sont reprises telles
quelles de l'export fourni par Pierre, pour que le corpus décrive exactement le
matériel réel. Seules les grappes sont générées ici.

Usage : python3 gen_corpus.py <export-source.json> corpus-definitions.json
"""
import json
import sys
import uuid

ISO = "sa303-isophase"
CCA = "sa303-cca"


def cluster(name, ids, splays, compartment="flown", tilt=None, tie=None, bumper="sa303-bumper", height=0.0):
    assert len(ids) == len(splays) + 1, name
    return {
        "id": str(uuid.uuid5(uuid.NAMESPACE_URL, name)),
        "name": name,
        "schemaVersion": 2,
        "speakerModelIds": ids,
        "compartment": compartment,
        "joints": [{"splay": s} for s in splays],
        "imposedTilt": tilt,
        "tieAngle": tie,
        "bumperModelId": bumper,
        "bumperHeight": height,
    }


def iso(n):
    return [ISO] * n


def corpus():
    c = []
    # --- tailles, sans tirette (pendaison libre) -----------------------------
    for n in (1, 2, 4, 8, 10, 12, 14):
        c.append(cluster(f"iso{n} droit free", iso(n), [0.0] * (n - 1)))
    for n in (2, 4, 8, 14):
        c.append(cluster(f"iso{n} 20 partout free", iso(n), [20.0] * (n - 1)))
    # --- tous les splays de la grille, int et ext, sur 4 caissons -----------------
    for s in (0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.5, 20.0):
        c.append(cluster(f"iso4 splay {s} partout free", iso(4), [s] * 3))
    # --- formes -------------------------------------------------------------------
    c.append(cluster("J 12 free", iso(12), [0, 0, 0, 0, 1, 1, 2, 3, 5, 10.5, 20]))
    c.append(cluster("banane 12 free", iso(12), [1, 2, 3, 4, 5, 5, 5, 10.5, 10.5, 10.5, 10.5]))
    c.append(cluster("iso11+cca3 free", iso(11) + [CCA] * 3, [0, 0, 0, 1, 1, 2, 3, 4, 5, 10.5, 20, 20, 20]))
    c.append(cluster("iso12+cca2 J free", iso(12) + [CCA] * 2, [0, 0, 0, 0, 1, 1, 2, 3, 4, 5, 10.5, 10.5, 20]))
    # --- assiettes imposées : de fortement vers le haut à fortement vers le bas ---
    for tilt in (-15.0, -8.0, -3.0, 0.0, 3.0, 8.0, 15.0, 25.0, 35.0):
        c.append(cluster(f"J14 tilt {tilt:+.0f} tie180", iso(13) + [CCA], [2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 10.5], tilt=tilt, tie=180.0))
    # --- tirette : plusieurs angles dans la plage, faible et forte ------------------
    splJ = [2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 10.5]
    for angle in (150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 315.0):
        c.append(cluster(f"J14 tilt +25 tie{angle:.0f}", iso(13) + [CCA], splJ, tilt=25.0, tie=angle))
    # tirette faible : assiette juste au-delà de la portée de barre
    c.append(cluster("J14 tilt +12 tie270 (faible)", iso(13) + [CCA], splJ, tilt=12.0, tie=270.0))
    # pickup au centre (assiette = pendaison libre approx) et au déport max sans tirette
    c.append(cluster("J14 tilt -6 sans tirette", iso(13) + [CCA], splJ, tilt=-6.0))
    c.append(cluster("J14 tilt +9 barre (proche max)", iso(13) + [CCA], splJ, tilt=9.0))
    c.append(cluster("droit 8 tilt 0", iso(8), [0.0] * 7, tilt=0.0))
    c.append(cluster("droit 8 tilt -20 tie300", iso(8), [0.0] * 7, tilt=-20.0, tie=300.0))
    # --- k_dyn / élévation : mêmes grappes à une autre hauteur ---------------------
    c.append(cluster("iso4 droit free h=8m", iso(4), [0.0] * 3, height=8000.0))
    # --- stacks --------------------------------------------------------------------
    c.append(cluster("stack 3 droit 0", iso(3), [0.0, 0.0], "stacked", tilt=0.0))
    c.append(cluster("stack 3 bas 10", iso(3), [0.0, 10.5], "stacked", tilt=10.0))
    c.append(cluster("stack 3 bas 20", iso(3), [0.0, 20.0], "stacked", tilt=20.0))
    c.append(cluster("stack 4 bas 20", iso(4), [0.0, 0.0, 20.0], "stacked", tilt=20.0))
    c.append(cluster("stack 4 cca bas 20", iso(2) + [CCA] * 2, [0.0, 10.5, 20.0], "stacked", tilt=20.0))
    c.append(cluster("stack 3 bas -10 (nez en l'air)", iso(3), [0.0, 0.0], "stacked", tilt=-10.0))
    # --- cas invalides, à refuser ---------------------------------------------------
    c.append(cluster("INVALIDE splay 7 hors grille", iso(3), [7.0, 0.0]))
    c.append(cluster("INVALIDE splay 10 (ancienne grille)", iso(3), [10.0, 0.0]))
    c.append(cluster("INVALIDE tirette en poussée (0°)", iso(13) + [CCA], splJ, tilt=25.0, tie=0.0))
    c.append(cluster("INVALIDE tirette en poussée (90°)", iso(13) + [CCA], splJ, tilt=25.0, tie=90.0))
    c.append(cluster("INVALIDE tirette traverse la grappe (140°)", iso(13) + [CCA], splJ, tilt=25.0, tie=141.0))
    c.append(cluster("INVALIDE 2 caissons, 3 jonctions", iso(4), [0.0, 0.0, 0.0]) | {"speakerModelIds": iso(2)})
    return c


if __name__ == "__main__":
    src = json.load(open(sys.argv[1]))
    out = {
        "settings": src["settings"],
        "definitions": src["definitions"],
        "clusters": corpus(),
    }
    json.dump(out, open(sys.argv[2], "w"), indent=1)
    print(len(out["clusters"]), "grappes écrites dans", sys.argv[2])

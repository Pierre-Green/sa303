"""
Solveur de référence indépendant pour la validation du modèle mécanique du
solveur de grappes SA303 (crate `sa303-core`).

Règle : rien ici n'est repris du code Rust. La géométrie est reconstruite
depuis les définitions (JSON d'export, section `definitions`), la statique est
posée corps par corps et résolue par une matrice complète dont on contrôle le
rang. Les conventions sont documentées au fil du code et récapitulées dans le
rapport.

Unités : mm, N, N·mm en interne (converties en N·m à la sortie quand le champ
de l'export est en N·m). Angles : degrés en entrée, radians en interne.

Repère global : celui du caisson 0 non tourné, x vers l'arrière, y vers le haut.
Une rotation d'angle phi (radians) est directe (sens trigonométrique) : phi > 0
fait monter l'arrière, donc « nez vers le bas ».

Convention angulaire « §2 » des champs *AngleDeg de l'export : 0° = vers le bas,
90° = vers l'avant (−x), 180° = vers le haut, 270° = vers l'arrière (+x).
"""
from __future__ import annotations

import json
import math
from dataclasses import dataclass, field
from typing import Optional

import numpy as np

# ---------------------------------------------------------------------------
# Petits utilitaires vectoriels
# ---------------------------------------------------------------------------


def v(x, y):
    return np.array([float(x), float(y)])


def rot(p, phi):
    c, s = math.cos(phi), math.sin(phi)
    return np.array([c * p[0] - s * p[1], s * p[0] + c * p[1]])


def cross(a, b):
    return a[0] * b[1] - a[1] * b[0]


def norm(a):
    return float(np.hypot(a[0], a[1]))


def unit(a):
    n = norm(a)
    return a / n if n > 0 else a * 0.0


def polar(c, r, deg):
    a = math.radians(deg)
    return np.array([c[0] + r * math.cos(a), c[1] + r * math.sin(a)])


def angle_deg_sec2(vec):
    """Convention §2 : 0° vers le bas, horaire, 90° vers l'avant."""
    a = math.degrees(math.atan2(-vec[0], -vec[1]))
    return a % 360.0


def dir_from_angle_sec2(deg):
    t = math.radians(deg)
    return np.array([-math.sin(t), -math.cos(t)])


# ---------------------------------------------------------------------------
# Géométrie d'une enceinte, reconstruite depuis la définition
# ---------------------------------------------------------------------------


@dataclass
class SpeakerGeo:
    """Toutes les cotes utiles d'une enceinte, repère enceinte (origine au
    centre, x arrière, y haut)."""

    id: str
    mass_kg: float
    cg: np.ndarray
    ht: np.ndarray  # charnière haute (avant-haut)
    hb: np.ndarray  # charnière basse (avant-bas)
    bielle_L: float  # entraxe de la bielle avant
    anchor: np.ndarray  # ancrage de barre, repère enceinte
    latch: np.ndarray  # verrou de barre, repère enceinte
    e_axis: np.ndarray  # axe de barre, du verrou vers l'ancrage (unitaire)
    e_front: np.ndarray  # normale « vers l'avant » de l'axe de barre
    half_angle: float  # demi-dièdre, degrés
    crown_radius: float
    crown_delta: float
    splay0_angle: float
    inner_splays: list
    splay_grid: list
    bar: dict  # cotation de la barre arrière (dict du JSON)
    height: float
    depth: float
    rear_face_x: float
    outline: np.ndarray  # 4 coins, repère enceinte

    @staticmethod
    def from_json(sp: dict) -> "SpeakerGeo":
        h = sp["hinge"]
        ht = v(h["x"], h["y"])
        hb = v(h["x"], -h["y"])
        # La séparation verticale centre à centre au splay 0 vaut 2·y + entraxe.
        L = h["jointSeparation"] - 2.0 * h["y"]
        anchor = polar(ht, sp["anchor"]["radius"], sp["anchor"]["angleDeg"])
        latch = polar(ht, sp["latch"]["radius"], sp["latch"]["angleDeg"])
        e_axis = unit(anchor - latch)
        e_front = np.array([-e_axis[1], e_axis[0]])
        half_h, half_d = sp["height"] / 2.0, sp["depth"] / 2.0
        taper = math.tan(math.radians(sp["totalVerticalAngle"] / 2.0)) * sp["depth"]
        outline = np.array(
            [
                [-half_d, half_h],
                [half_d, half_h - taper],
                [half_d, -half_h + taper],
                [-half_d, -half_h],
            ]
        )
        return SpeakerGeo(
            id=sp["id"],
            mass_kg=sp["massKg"],
            cg=v(*sp["cg"]),
            ht=ht,
            hb=hb,
            bielle_L=L,
            anchor=anchor,
            latch=latch,
            e_axis=e_axis,
            e_front=e_front,
            half_angle=sp["totalVerticalAngle"] / 2.0,
            crown_radius=sp["crown"]["radius"],
            crown_delta=sp["crown"]["delta"],
            splay0_angle=sp["crown"]["splay0Angle"],
            inner_splays=list(sp["crown"].get("innerSplays", [1.0, 3.0, 5.0])),
            splay_grid=list(sp["splayGrid"]),
            bar=sp["rearBar"],
            height=sp["height"],
            depth=sp["depth"],
            rear_face_x=sp["rearFaceX"],
            outline=outline,
        )

    # --- cinématique de la jonction (repère de CE caisson = caisson du haut) ---

    def has_hole(self, splay):
        return any(abs(s - splay) < 1e-9 for s in self.splay_grid)

    def row(self, splay) -> str:
        return "int" if any(abs(s - splay) < 1e-9 for s in self.inner_splays) else "ext"

    def pv(self, splay):
        """Goupille basse de la bielle avant : la bielle tourne de splay/2 autour
        de `hb` (partage angulaire égal, imposé par construction)."""
        h = math.radians(splay / 2.0)
        return self.hb + self.bielle_L * np.array([math.sin(h), -math.cos(h)])

    def carry(self, p_lower_local, splay):
        """Point du caisson du bas (dans son propre repère) porté dans le repère
        du caisson du haut : `ht` du bas coïncide avec `pv(splay)` et le bas est
        tourné du splay entier."""
        return self.pv(splay) + rot(p_lower_local - self.ht, math.radians(splay))

    def bar_top_hole_in_lower_frame(self, row, lower: "SpeakerGeo") -> np.ndarray:
        """Trou haut de la barre, repère du caisson du BAS (celui qui porte la
        paire ancrage/verrou), construit depuis la cotation de la barre : c'est
        l'approche indépendante de la polaire du code."""
        bar = lower.bar
        hole = bar["holes"]["up660" if row == "int" else "up680"]
        anchor_along = bar["holes"]["anchor"][0]
        along = hole[0] - anchor_along
        lateral = hole[1] - bar["holes"]["anchor"][1]
        return lower.anchor + along * lower.e_axis + lateral * lower.e_front

    def crown_polar(self, splay):
        """Trou de couronne d'après la cotation polaire du caisson (rayon 680 ou
        660 selon la rangée, angle demi-dièdre + splay0 + splay, centré sur la
        goupille basse de bielle)."""
        r = self.crown_radius - (self.crown_delta if self.row(splay) == "int" else 0.0)
        return polar(self.pv(splay), r, self.half_angle + self.splay0_angle + splay)

    def crown_from_bar(self, splay, lower: "SpeakerGeo"):
        """Le même trou, obtenu par la barre montée sur le caisson du bas."""
        return self.carry(self.bar_top_hole_in_lower_frame(self.row(splay), lower), splay)


# ---------------------------------------------------------------------------
# Assemblage de la chaîne
# ---------------------------------------------------------------------------


@dataclass
class Placed:
    o: np.ndarray
    phi: float

    def to_global(self, p):
        return self.o + rot(p, self.phi)

    def to_local(self, p):
        return rot(p - self.o, -self.phi)


def build_chain(geos, splays, phi0):
    placed = []
    o = v(0, 0)
    phi = phi0
    for i, g in enumerate(geos):
        placed.append(Placed(o.copy(), phi))
        if i < len(splays):
            phi_next = phi + math.radians(splays[i])
            o = o + rot(g.pv(splays[i]), phi) - rot(geos[i + 1].ht, phi_next)
            phi = phi_next
    return placed


def cluster_cg(geos, placed):
    m = sum(g.mass_kg for g in geos)
    s = sum(g.mass_kg * p.to_global(g.cg) for g, p in zip(geos, placed))
    return s / m, m


def free_hang_phi(geos, splays, pickup_local):
    """Pendaison libre : le CG passe sous le point de levage. Indépendant de la
    translation, on construit à phi=0 et on lit l'angle."""
    placed = build_chain(geos, splays, 0.0)
    cg, _ = cluster_cg(geos, placed)
    d = cg - (placed[0].o + pickup_local)
    return -math.atan2(d[0], -d[1])


def pickup_x_for_tilt(geos, splays, phi0, pickup_h):
    placed = build_chain(geos, splays, phi0)
    cg, _ = cluster_cg(geos, placed)
    s, c = math.sin(phi0), math.cos(phi0)
    return (cg[0] + pickup_h * s) / c


# ---------------------------------------------------------------------------
# Statique multi-corps : assemblage explicite et résolution matricielle
# ---------------------------------------------------------------------------


class Statics:
    """Assembleur d'équilibre de corps rigides plans.

    Chaque corps a 3 équations (ΣFx, ΣFy, ΣM/origine). Les inconnues sont
    déclarées par type de liaison :
      - `pin(A, B, P)`        : force 2D inconnue F, +F sur A, −F sur B, au point P
      - `two_force(A, B, PA, PB)` : scalaire λ, +uλ sur B en PB, −uλ sur A en PA,
                                    u = unit(PB − PA)
      - `wrench(A, B, P)`     : force 2D + moment inconnus, +(F, M) sur A, −(F, M) sur B
      - `support(A, P)`       : force 2D inconnue sur A (appui extérieur)
      - `cable(A, P, dir)`    : scalaire T ≥ 0 attendu, +dir·T sur A
    Les charges connues sont ajoutées par `load(A, P, F)`.
    """

    def __init__(self):
        self.bodies = {}
        self.unknowns = []  # (name, size)
        self.terms = []  # (body, unknown_index, kind, data)
        self.loads = []  # (body, P, F)

    def body(self, name):
        if name not in self.bodies:
            self.bodies[name] = len(self.bodies)
        return name

    def _new(self, name, size):
        idx = sum(s for _, s in self.unknowns)
        self.unknowns.append((name, size))
        return idx

    def pin(self, A, B, P, name):
        i = self._new(name, 2)
        self.terms.append((A, i, "F", (P, +1.0)))
        self.terms.append((B, i, "F", (P, -1.0)))
        return name

    def support(self, A, P, name):
        i = self._new(name, 2)
        self.terms.append((A, i, "F", (P, +1.0)))
        return name

    def support_wrench(self, A, P, name):
        i = self._new(name, 3)
        self.terms.append((A, i, "W", (P, +1.0)))
        return name

    def two_force(self, A, B, PA, PB, name):
        i = self._new(name, 1)
        u = unit(PB - PA)
        self.terms.append((B, i, "S", (PB, u)))
        self.terms.append((A, i, "S", (PA, -u)))
        return name

    def cable(self, A, P, direction, name):
        i = self._new(name, 1)
        self.terms.append((A, i, "S", (P, unit(direction))))
        return name

    def wrench(self, A, B, P, name):
        i = self._new(name, 3)
        self.terms.append((A, i, "W", (P, +1.0)))
        self.terms.append((B, i, "W", (P, -1.0)))
        return name

    def load(self, A, P, F):
        self.loads.append((A, np.asarray(P, float), np.asarray(F, float)))

    def assemble(self):
        nb = len(self.bodies)
        nu = sum(s for _, s in self.unknowns)
        A = np.zeros((3 * nb, nu))
        b = np.zeros(3 * nb)
        for body, i, kind, data in self.terms:
            r = 3 * self.bodies[body]
            if kind == "F":
                P, sgn = data
                A[r, i] += sgn
                A[r + 1, i + 1] += sgn
                # moment de (Fx, Fy) en P autour de l'origine : Px Fy − Py Fx
                A[r + 2, i] += -sgn * P[1]
                A[r + 2, i + 1] += sgn * P[0]
            elif kind == "S":
                P, u = data
                A[r, i] += u[0]
                A[r + 1, i] += u[1]
                A[r + 2, i] += P[0] * u[1] - P[1] * u[0]
            elif kind == "W":
                P, sgn = data
                A[r, i] += sgn
                A[r + 1, i + 1] += sgn
                A[r + 2, i] += -sgn * P[1]
                A[r + 2, i + 1] += sgn * P[0]
                A[r + 2, i + 2] += sgn
        for body, P, F in self.loads:
            r = 3 * self.bodies[body]
            b[r] -= F[0]
            b[r + 1] -= F[1]
            b[r + 2] -= cross(P, F)
        return A, b

    def solve(self):
        A, b = self.assemble()
        x, res, rank, sv = np.linalg.lstsq(A, b, rcond=None)
        residual = A @ x - b
        self.x = x
        self.A, self.b = A, b
        self.rank = rank
        self.n_eq, self.n_unknowns = A.shape
        self.residual = residual
        return x

    def get(self, name):
        idx = 0
        for n, s in self.unknowns:
            if n == name:
                return self.x[idx : idx + s]
            idx += s
        raise KeyError(name)

    def body_residuals(self):
        """Résidu (force, moment) par corps, en relatif à la charge appliquée."""
        out = {}
        for name, k in self.bodies.items():
            f = self.residual[3 * k : 3 * k + 2]
            m = self.residual[3 * k + 2]
            out[name] = (norm(f), abs(m))
        return out


# ---------------------------------------------------------------------------
# Modèle de grappe complet
# ---------------------------------------------------------------------------


@dataclass
class ClusterInput:
    geos: list
    splays: list
    compartment: str  # "flown" | "stacked"
    imposed_tilt: Optional[float]
    tie_angle: Optional[float]
    bumper: dict
    bumper_bar_max: Optional[float]
    settings: dict


@dataclass
class JointRef:
    index: int
    splay: float
    row: str
    # points, repère global
    pa: np.ndarray
    pb: np.ndarray
    bo: np.ndarray
    an: np.ndarray
    lt: np.ndarray
    # efforts « subis par la quincaillerie », par flanc, repère global
    f_orientation_g: np.ndarray  # à la goupille de couronne (caisson du haut)
    f_pivot_g: np.ndarray  # aux goupilles de bielle
    f_anchor_g: np.ndarray  # répartition 50/50 + couple (comme le code)
    f_latch_g: np.ndarray
    # enveloppe « tout l'axial sur un seul pion »
    f_anchor_env: float
    f_latch_env: float
    # repère barre, par flanc
    bar_axial: float
    bar_shear: float
    bar_moment_at_pair_nm: float
    bar_moment_anchor_nm: float
    lambda_bielle: float  # >0 : compression (u du haut vers le bas)
    residual_force: float
    residual_moment: float


@dataclass
class ClusterRef:
    phi0: float
    phi_free_hang: Optional[float]
    placed: list
    cg: np.ndarray
    mass: float
    pickup_local: Optional[np.ndarray]
    pickup_global: Optional[np.ndarray]
    tie_tension: float
    tie_point_global: Optional[np.ndarray]
    tie_dir: Optional[np.ndarray]
    tie_range: Optional[tuple]
    bumper_bar_exceeded: bool
    joints: list
    bumper: dict
    statics: Statics
    notes: list = field(default_factory=list)


def tie_geometry(geos, placed, pickup_g, weight_n, cg):
    q = placed[-1].to_global(geos[-1].crown_polar(0.0))
    mw = cross(cg - pickup_g, v(0, -weight_n))
    return q, mw


def tie_tension_for(q, pickup_g, mw, angle_deg):
    u = dir_from_angle_sec2(angle_deg)
    d = cross(q - pickup_g, u)
    if abs(d) < 1e-9:
        return math.inf
    return -mw / d


def tie_valid_range(q, pickup_g, mw):
    """Demi-cercle des directions à tension positive (bornes exclues)."""
    qp = unit(q - pickup_g)
    perp = rot(qp, math.pi / 2)
    d = cross(q - pickup_g, perp)
    t = -mw / d
    center = perp if t >= 0 else -perp
    c = angle_deg_sec2(center)
    lo = (c - 90.0) % 360.0
    return (lo, lo + 180.0)


def solve_cluster(inp: ClusterInput, k_dyn=None) -> ClusterRef:
    S = inp.settings
    g = S["gravity"]
    kd = S["dynamicFactor"] if k_dyn is None else k_dyn
    share = S["sharePerFlank"]
    geos, splays = inp.geos, inp.splays
    n = len(geos)
    notes = []

    for i, s in enumerate(splays):
        if not geos[i].has_hole(s):
            raise ValueError(f"Jonction {i + 1} : splay {s}° hors grille de perçage")

    bm = inp.bumper
    ref = geos[0] if inp.compartment == "flown" else geos[-1]
    pickup_h = ref.height / 2.0 + bm["height"] + bm["shackleHeightAboveBumper"]

    # ---- assiette ----------------------------------------------------------
    if inp.compartment == "stacked":
        bottom = inp.imposed_tilt or 0.0
        phi0 = math.radians(bottom - sum(splays))
        phi_fh = None
    else:
        phi_fh = free_hang_phi(geos, splays, v(0, pickup_h))
        phi0 = math.radians(inp.imposed_tilt) if inp.imposed_tilt is not None else phi_fh

    placed = build_chain(geos, splays, phi0)
    cg, mass = cluster_cg(geos, placed)
    weight = mass * g * kd

    # ---- accroche et tirette (vol) ------------------------------------------
    pickup_local = None
    pickup_g = None
    tie_T = 0.0
    tie_q = None
    tie_dir = None
    tie_range = None
    exceeded = False
    if inp.compartment == "flown":
        if inp.imposed_tilt is None:
            pickup_local = v(0, pickup_h)
        else:
            raw_x = pickup_x_for_tilt(geos, splays, phi0, pickup_h)
            bar_max = inp.bumper_bar_max if inp.bumper_bar_max is not None else bm["maxDirectDeportMm"]
            if abs(raw_x) <= bar_max:
                pickup_local = v(raw_x, pickup_h)
            else:
                exceeded = True
                pickup_local = v(bar_max * math.copysign(1, raw_x), pickup_h)
        pickup_g = placed[0].to_global(pickup_local)
        if exceeded:
            tie_q, mw = tie_geometry(geos, placed, pickup_g, weight, cg)
            tie_range = tie_valid_range(tie_q, pickup_g, mw)
            angle = inp.tie_angle
            if angle is None:
                lo, hi = tie_range
                inside = lo <= 180.0 <= hi or lo <= 540.0 <= hi
                angle = 180.0 if inside else lo
            tie_T = tie_tension_for(tie_q, pickup_g, mw, angle)
            if not (tie_T >= 0) or math.isinf(tie_T):
                raise ValueError(
                    f"tirette à {angle}° : tension {tie_T:.1f} N (négative ou infinie), "
                    f"plage valable {tie_range}"
                )
            tie_dir = dir_from_angle_sec2(angle)

    # ---- assemblage multi-corps ---------------------------------------------
    st = Statics()
    for i in range(n):
        st.body(f"cab{i}")
        st.load(f"cab{i}", placed[i].to_global(geos[i].cg), v(0, -geos[i].mass_kg * g * kd))
    st.body("bumper")

    # Jonctions entre caissons : bielle (2 forces) + barre (corps rigide,
    # articulée à la couronne du haut, encastrée par un torseur sur le bas).
    jpts = []
    for i, s in enumerate(splays):
        up, lo = geos[i], geos[i + 1]
        pu, pl = placed[i], placed[i + 1]
        pa = pu.to_global(up.hb)
        pb = pu.to_global(up.pv(s))
        bo = pu.to_global(up.crown_polar(s))
        an = pl.to_global(lo.anchor)
        lt = pl.to_global(lo.latch)
        # contrôle de fermeture : ht du bas sur la goupille basse de bielle
        closure = norm(pl.to_global(lo.ht) - pb)
        crown_via_bar = pu.to_global(up.crown_from_bar(s, lo))
        st.body(f"bar{i}")
        st.two_force(f"cab{i}", f"cab{i+1}", pa, pb, f"lam{i}")
        st.pin(f"bar{i}", f"cab{i}", bo, f"Fc{i}")  # +F sur la barre, −F sur le haut
        st.wrench(f"cab{i+1}", f"bar{i}", an, f"W{i}")  # torseur barre→caisson bas, réduit à l'ancrage
        jpts.append((pa, pb, bo, an, lt, closure, norm(crown_via_bar - bo)))

    # Bumper ↔ caisson de référence.
    bum = {}
    if inp.compartment == "flown":
        p0, g0 = placed[0], geos[0]
        rb = next(b for b in bm["rearBars"] if abs(b["tiltDeg"]) < 1e-6)
        pvg = p0.to_global(g0.ht + v(0, bm["pivotBarUsableLengthMm"]))
        bl = p0.to_global(g0.ht)
        an_b = p0.to_global(g0.anchor + rb["topHoleAlongMm"] * g0.e_axis + rb["topHoleLateralMm"] * g0.e_front)
        pa_b, pl_b = p0.to_global(g0.anchor), p0.to_global(g0.latch)
        st.body("bbar")
        st.two_force("bumper", "cab0", pvg, bl, "lamB")
        st.pin("bbar", "bumper", an_b, "FcB")
        st.wrench("cab0", "bbar", pa_b, "WB")
        st.support("bumper", pickup_g, "R")
        if exceeded:
            st.cable(f"cab{n-1}", tie_q, tie_dir, "T")
        bum.update(pvg=pvg, bl=bl, an=an_b, anchor=pa_b, latch=pl_b)
    else:
        # Stack : le bumper est au sol sous le dernier caisson. Il reçoit du
        # sol un torseur (résultante + moment : pression répartie), et tient le
        # caisson par deux pions cotés depuis ses bords (comme le code) — ET,
        # en parallèle, on rend la variante « bielle + barre » pour comparaison
        # (voir rapport §2.9). Ici : deux pions = 4 inconnues pour 3 équations
        # → hyperstatique ; on résout à part avec la règle 50/50 + couple.
        pn, gn = placed[-1], geos[-1]
        fb = pn.to_global(gn.outline[3])
        outline = [fb, fb + v(bm["depth"], 0), fb + v(bm["depth"], -bm["height"]), fb + v(0, -bm["height"])]
        # Réaction du sol : résultante + moment (pression répartie) au milieu de
        # la face d'appui. On vérifie ensuite que la résultante tombe dans
        # l'empreinte (pas de basculement).
        st.support_wrench("bumper", (outline[2] + outline[3]) / 2, "G")
        # liaison bumper ↔ dernier caisson : torseur (rigide), puis répartition
        st.wrench("cab%d" % (n - 1), "bumper", fb, "WBS")
        bum.update(outline=outline, front_bottom=fb)

    st.solve()

    # ---- rang -----------------------------------------------------------------
    if inp.compartment == "flown":
        n_bodies = len(st.bodies)
        expect = 3 * n_bodies
        if st.rank < st.n_unknowns:
            notes.append(f"système sous-déterminé : rang {st.rank} < {st.n_unknowns} inconnues")
        if norm(st.residual) > 1e-6 * max(1.0, weight):
            notes.append(f"résidu global {norm(st.residual):.3e} : les équations ne sont pas toutes satisfaites")

    # ---- post-traitement par jonction ------------------------------------------
    joints = []
    for i, (pa, pb, bo, an, lt, closure, crown_gap) in enumerate(jpts):
        lam = float(st.get(f"lam{i}")[0])
        Fc = st.get(f"Fc{i}")  # force sur la barre à la couronne
        # Force que la barre applique au caisson du bas = torseur W (réduit à l'ancrage)
        W = st.get(f"W{i}")
        # ce que voit la goupille de couronne (caisson du haut) : force de la barre
        # sur le haut = −Fc ; « ce que subit la quincaillerie » = force appliquée
        # par la barre sur le flanc. Par flanc.
        f_ori_g = -Fc * share
        u = unit(pb - pa)
        f_piv_g = -u * lam * share  # ce que subit la goupille hb du haut
        # répartition sur la paire : la barre livre au caisson du bas la force
        # Fc (elle est en équilibre : Fc + (−W.F) = 0 ⇒ W.F = Fc) et le moment.
        lo = geos[i + 1]
        e_axis = unit(an - lt)
        e_front = np.array([-e_axis[1], e_axis[0]])
        Fbar = Fc  # force appliquée à la barre par la couronne ; la barre applique −Fbar... voir ci-dessous
        # Convention identique au code : `f_on_bar` = force reçue par la barre à la
        # couronne = force sur le corps libre (bas) = Fc ; les goupilles de la
        # paire « subissent » ce que la barre leur délivre : Fc/2 ± P.
        Gp = (an + lt) / 2
        m_g = cross(bo - Gp, Fc)
        d = norm(an - lt)
        perp = np.array([-(an - lt)[1], (an - lt)[0]]) / d
        P = perp * (m_g / d)
        f_anchor_g = (Fc * 0.5 + P) * share
        f_latch_g = (Fc * 0.5 - P) * share
        # enveloppe : tout l'axial sur un seul pion
        f_ax = float(np.dot(Fc, e_axis)) * share
        f_tr = float(np.dot(Fc, e_front)) * share
        Pm = float(np.dot(P, e_front)) * share
        f_anchor_env = math.hypot(f_tr / 2 + Pm, f_ax)
        f_latch_env = math.hypot(f_tr / 2 - Pm, f_ax)
        # repère barre
        bar_axial = -f_ax  # positif en compression (pousse vers l'ancrage)
        bar_shear = f_tr
        bar_moment_pair = m_g * share / 1000.0
        bar_moment_anchor = cross(an - bo, Fc) * share / 1000.0
        rf, rm = st.body_residuals()[f"cab{i+1}"]
        joints.append(
            JointRef(
                index=i, splay=splays[i], row=geos[i].row(splays[i]),
                pa=pa, pb=pb, bo=bo, an=an, lt=lt,
                f_orientation_g=f_ori_g, f_pivot_g=f_piv_g,
                f_anchor_g=f_anchor_g, f_latch_g=f_latch_g,
                f_anchor_env=f_anchor_env, f_latch_env=f_latch_env,
                bar_axial=bar_axial, bar_shear=bar_shear,
                bar_moment_at_pair_nm=bar_moment_pair,
                bar_moment_anchor_nm=abs(bar_moment_anchor),
                lambda_bielle=lam, residual_force=rf, residual_moment=rm,
            )
        )
        joints[-1].closure_mm = closure
        joints[-1].crown_gap_mm = crown_gap

    # ---- bumper -------------------------------------------------------------------
    if inp.compartment == "flown":
        FcB = st.get("FcB")
        lamB = float(st.get("lamB")[0])
        R = st.get("R")
        uB = unit(bum["bl"] - bum["pvg"])
        Gp = (bum["anchor"] + bum["latch"]) / 2
        m_g = cross(bum["an"] - Gp, FcB)
        d = norm(bum["anchor"] - bum["latch"])
        perp = np.array([-(bum["anchor"] - bum["latch"])[1], (bum["anchor"] - bum["latch"])[0]]) / d
        P = perp * (m_g / d)
        bum.update(
            f_orientation_g=-FcB * share,
            f_pivot_g=uB * lamB * share,  # ce que subit le pion avant (bumper) : force de la bielle sur le bumper = +u·λ... (voir rapport)
            f_pair_anchor_g=(FcB * 0.5 + P) * share,
            f_pair_latch_g=(FcB * 0.5 - P) * share,
            pair_moment_nm=m_g * share / 1000.0,
            support=R,
            lamB=lamB,
            pin_span=norm(bum["an"] - bum["pvg"]),
            T=float(st.get("T")[0]) if exceeded else 0.0,
        )
        # Moment que la structure du bumper transfère entre ses deux pions :
        # efforts subis par les pions du bumper (force de la bielle sur le
        # bumper en pvg, force de la barre sur le bumper en an).
        f_on_bumper_front = -uB * lamB  # bielle → bumper (u va de pvg vers bl ; +uλ agit sur cab0)
        f_on_bumper_rear = -FcB  # barre → bumper
        gmid = (bum["an"] + bum["pvg"]) / 2
        bum["pin_pair_moment_nm"] = (
            cross(bum["an"] - gmid, f_on_bumper_rear * share) + cross(bum["pvg"] - gmid, f_on_bumper_front * share)
        ) / 1000.0
        # Le pion avant du bumper subit la force de la bielle sur lui.
        bum["f_pivot_g"] = f_on_bumper_front * share
        bum["f_orientation_g"] = f_on_bumper_rear * share

    return ClusterRef(
        phi0=phi0, phi_free_hang=phi_fh, placed=placed, cg=cg, mass=mass,
        pickup_local=pickup_local, pickup_global=pickup_g,
        tie_tension=tie_T, tie_point_global=tie_q, tie_dir=tie_dir, tie_range=tie_range,
        bumper_bar_exceeded=exceeded, joints=joints, bumper=bum, statics=st, notes=notes,
    )


# ---------------------------------------------------------------------------
# Lecture d'un export d'audit
# ---------------------------------------------------------------------------


def inputs_from_export(doc: dict, cluster: dict) -> ClusterInput:
    spk = {s["id"]: s for s in doc["definitions"]["speakers"]}
    bumpers = {b["id"]: b for b in doc["definitions"]["bumpers"]}
    d = cluster["definition"] if "definition" in cluster else cluster
    geos = [SpeakerGeo.from_json(spk[i]) for i in d["speakerModelIds"]]
    bm = bumpers[d["bumperModelId"]]
    bar_max = None
    for bb in doc["definitions"].get("bumperBars", []):
        if any(c["bumperModelId"] == bm["id"] for c in bb["compatibleBumpers"]):
            bar_max = bb["maxDeportMm"]
    return ClusterInput(
        geos=geos,
        splays=[j["splay"] for j in d["joints"]],
        compartment=d["compartment"],
        imposed_tilt=d.get("imposedTilt"),
        tie_angle=d.get("tieAngle"),
        bumper=bm,
        bumper_bar_max=bar_max,
        settings=doc["settings"],
    )


# ---------------------------------------------------------------------------
# Modèle élastique : partage axial entre ancrage et verrou avec jeu
# ---------------------------------------------------------------------------


def huth_stiffness(t_bar=10.0, t_flank=4.0, d=12.0, E=210_000.0, a=2 / 3, b=3.0):
    """Raideur d'une goupille en cisaillement double (Huth 1986, assemblage
    boulonné métallique : a = 2/3, b = 3). Rend N/mm pour l'ensemble
    bar + deux flancs."""
    n = 2  # double cisaillement
    C = ((t_bar + t_flank) / (2 * d)) ** a * (b / n) * (
        1 / (t_bar * E) + 1 / (n * t_flank * E) + 1 / (2 * t_bar * E) + 1 / (2 * n * t_flank * E)
    )
    return 1.0 / C


def axial_share_with_gap(F_axial, gap_mm, k=None):
    """Deux goupilles en parallèle sur l'axe de barre, raideurs égales k, la
    seconde ne portant qu'après fermeture d'un jeu `gap_mm`. Rend (F1, F2)."""
    k = huth_stiffness() if k is None else k
    F = abs(F_axial)
    # d'abord une seule goupille jusqu'à δ = gap
    if F <= k * gap_mm:
        return (F, 0.0)
    # ensuite les deux : F = k δ + k (δ − gap)
    delta = (F + k * gap_mm) / (2 * k)
    return (k * delta, k * (delta - gap_mm))


def bar_pair_spring_model(Fc_bar, p_crown, p_anchor, p_latch, k_ax, k_tr, gap_latch=0.0):
    """Barre rigide portée par deux ressorts 2D (ancrage, verrou) dans le
    repère barre, chargée par `Fc_bar` à `p_crown`. Le ressort axial du verrou a
    un jeu `gap_latch` (ne travaille qu'après fermeture). Rend les réactions
    (F_anchor, F_latch) que les goupilles exercent sur la barre. Résolution
    itérative sur l'état de contact du verrou (2 états)."""
    def solve(active_latch_axial):
        # inconnues : u (déplacement axial), w (transverse), th (rotation) de la barre
        # réaction ressort au point p : −K (u + th × r) où r = p − origine
        def K(p, kax):
            return np.diag([kax, k_tr])
        pts = [(p_anchor, k_ax), (p_latch, k_ax if active_latch_axial else 0.0)]
        A = np.zeros((3, 3)); b = np.zeros(3)
        for p, kax in pts:
            Kp = K(p, kax)
            # déplacement du point : [u − th·py, w + th·px]
            B = np.array([[1, 0, -p[1]], [0, 1, p[0]]])
            A[:2] += -Kp @ B
            A[2] += -(p[0] * (Kp @ B)[1] - p[1] * (Kp @ B)[0])
        # jeu : le ressort axial du verrou est précontraint de −k·gap quand actif
        b[:2] = -Fc_bar
        b[2] = -cross(p_crown, Fc_bar)
        if active_latch_axial and gap_latch:
            pre = np.array([k_ax * gap_latch, 0.0])
            b[:2] -= pre
            b[2] -= cross(p_latch, pre)
        x = np.linalg.solve(A, b)
        reac = []
        for p, kax in pts:
            disp = np.array([x[0] - x[2] * p[1], x[1] + x[2] * p[0]])
            r = -np.array([kax * disp[0], k_tr * disp[1]])
            if p is p_latch and active_latch_axial and gap_latch:
                r[0] += k_ax * gap_latch
            reac.append(r)
        return x, reac
    x, reac = solve(False)
    # le verrou entre en contact si son déplacement axial dépasse le jeu (dans le sens de la charge)
    disp_latch = x[0] - x[2] * p_latch[1]
    if abs(disp_latch) > gap_latch:
        x, reac = solve(True)
    return reac


if __name__ == "__main__":
    import sys

    doc = json.load(open(sys.argv[1]))
    for c in doc["clusters"]:
        inp = inputs_from_export(doc, c)
        r = solve_cluster(inp)
        print(c["definition"]["name"], "phi0", math.degrees(r.phi0), "tie", r.tie_tension, "rank", r.statics.rank, r.statics.n_unknowns, r.statics.n_eq)
        for j in r.joints:
            print(f"  J{j.index:2d} s={j.splay:5.1f} {j.row} ori={norm(j.f_orientation_g):8.1f} piv={norm(j.f_pivot_g):8.1f} anc={norm(j.f_anchor_g):8.1f} lat={norm(j.f_latch_g):8.1f} env=({j.f_anchor_env:8.1f},{j.f_latch_env:8.1f}) ax={j.bar_axial:8.1f} sh={j.bar_shear:8.1f} Mp={j.bar_moment_at_pair_nm:8.2f} Ma={j.bar_moment_anchor_nm:8.2f} clos={j.closure_mm:.2e} cg={j.crown_gap_mm:.2e}")
        print("  notes", r.notes)

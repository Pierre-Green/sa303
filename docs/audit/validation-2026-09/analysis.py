"""Analyses complémentaires : enveloppe axiale, modèle élastique, stack
alternatif, décomposition des comportements (§2.12), invariances."""
import json, math, copy
import numpy as np
from sa303_ref import *

doc = json.load(open('corpus-export.json'))
src = json.load(open('/Users/pierre/Downloads/sa303-audit-2-grappes-2026-09-21.json'))
S = doc['settings']

def solve(c, **kw):
    inp = inputs_from_export(doc, c)
    for k, val in kw.items(): setattr(inp, k, val)
    return inp, solve_cluster(inp)

print("=== 1. Enveloppe 'tout l'axial sur un pion' vs 50/50 (corpus, vol et stack, cas valides) ===")
worst = []
for c in doc['clusters']:
    if c['definition']['name'].startswith('INVALIDE'): continue
    inp, r = solve(c)
    for j in r.joints:
        a50, l50 = norm(j.f_anchor_g), norm(j.f_latch_g)
        worst.append((max(j.f_anchor_env/a50, j.f_latch_env/l50), c['definition']['name'], j.index, a50, j.f_anchor_env, l50, j.f_latch_env, j.bar_axial, j.bar_shear, j.bar_moment_at_pair_nm))
worst.sort(reverse=True)
for w in worst[:8]: print(f"  ratio {w[0]:.3f}  {w[1]} J{w[2]}  anc {w[3]:.0f}->{w[4]:.0f}  lat {w[5]:.0f}->{w[6]:.0f}  ax={w[7]:.0f} sh={w[8]:.0f} M={w[9]:.0f}")
# ratio sur l'effort MAX de la paire (celui qui dimensionne)
gov = [(max(w[4], w[6]) / max(w[3], w[5]), w[1], w[2]) for w in worst]
gov.sort(reverse=True); print("  pire ratio sur l'effort gouvernant de la paire :", gov[0])

print("\n=== 2. Raideur de goupille (Huth) et jeu nécessaire pour découpler les deux pions ===")
k = huth_stiffness(); print(f"  k_Huth = {k/1000:.0f} kN/mm (barre 10, flancs 2x4, Ø12, E=210 GPa)")
for F in (2000, 5000, 8400):
    print(f"  axial {F} N : δ élastique 1 pion = {F/k*1000:.1f} µm ; partage à jeu 0/20/50/100 µm :",
          [tuple(round(x) for x in axial_share_with_gap(F, g, k)) for g in (0, 0.02, 0.05, 0.1)])
# convergence du modèle ressort 2D vers le modèle rigide (jeu nul, raideurs égales)
inp, r = solve(next(c for c in doc['clusters'] if c['definition']['name']=='J14 tilt +25 tie180'))
j = r.joints[0]
Fc = -j.f_orientation_g/S['sharePerFlank']  # force sur la barre à la couronne (total)
e_axis = unit(j.an - j.lt); e_front = np.array([-e_axis[1], e_axis[0]])
tb = lambda p: np.array([np.dot(p - j.lt, e_axis), np.dot(p - j.lt, e_front)])
Fb = np.array([np.dot(Fc, e_axis), np.dot(Fc, e_front)])
for kax, ktr in ((k, k), (k*10, k), (k, k*10), (1e3, 1e3)):
    ra, rl = bar_pair_spring_model(Fb, tb(j.bo), tb(j.an), tb(j.lt), kax, ktr)
    print(f"  ressorts kax={kax/1000:.0f} ktr={ktr/1000:.0f} kN/mm : anchor={norm(ra)*0.5:.1f} latch={norm(rl)*0.5:.1f} (rigide 50/50: {norm(j.f_anchor_g):.1f}/{norm(j.f_latch_g):.1f}) transverse a/l = {ra[1]*0.5:.1f}/{rl[1]*0.5:.1f} axial a/l = {ra[0]*0.5:.1f}/{rl[0]*0.5:.1f}")
for gap in (0.01, 0.02, 0.05, 0.1):
    ra, rl = bar_pair_spring_model(Fb, tb(j.bo), tb(j.an), tb(j.lt), k, k, gap_latch=gap)
    print(f"  jeu verrou {gap*1000:.0f} µm : anchor={norm(ra)*0.5:.1f} latch={norm(rl)*0.5:.1f} axial a/l={ra[0]*0.5:.1f}/{rl[0]*0.5:.1f}")

print("\n=== 3. Cas 1 et 2 caissons, à la main ===")
for name in ('iso1 droit free', 'iso2 droit free'):
    c = next(c for c in doc['clusters'] if c['definition']['name']==name)
    inp, r = solve(c); g0 = inp.geos[0]; W = g0.mass_kg*S['gravity']*S['dynamicFactor']
    print(f"  {name}: phi={math.degrees(r.phi0):.4f}°  W(1 caisson)={W:.2f} N")
    b = r.bumper
    print(f"    bumper: pvg={b['pvg']} an={b['an']} bl={b['bl']} anchor={b['anchor']} latch={b['latch']}")
    print(f"    λ_bielle={b['lamB']:.2f} N (total)  f_pivot/flanc={norm(b['f_pivot_g']):.2f}  f_ori/flanc={norm(b['f_orientation_g']):.2f} pair anchor/latch={norm(b['f_pair_anchor_g']):.2f}/{norm(b['f_pair_latch_g']):.2f} M_pair={b['pair_moment_nm']:.3f}")
    for j in r.joints:
        print(f"    J{j.index}: pa={j.pa} pb={j.pb} bo={j.bo} an={j.an} lt={j.lt} λ={j.lambda_bielle:.2f} ori={norm(j.f_orientation_g):.2f} piv={norm(j.f_pivot_g):.2f} anc={norm(j.f_anchor_g):.2f} lat={norm(j.f_latch_g):.2f} Mpair={j.bar_moment_at_pair_nm:.3f} Ma={j.bar_moment_anchor_nm:.3f}")
    cgs = [p.to_global(g.cg) for g,p in zip(inp.geos, r.placed)]; print("    cg globaux:", cgs, "pickup", r.pickup_global)

print("\n=== 4. Invariances ===")
c4 = next(c for c in doc['clusters'] if c['definition']['name']=='iso4 droit free')
c4h = next(c for c in doc['clusters'] if c['definition']['name']=='iso4 droit free h=8m')
a = [ (j['fOrientationN'], j['fAnchorN']) for j in c4['result']['joints']]; b = [ (j['fOrientationN'], j['fAnchorN']) for j in c4h['result']['joints']]
print("  élévation 0 vs 8 m (code) : écart max", max(abs(x-y) for p,q in zip(a,b) for x,y in zip(p,q)))
# tirette à tension nulle == free hang : imposer tilt = phi_free_hang
cJ = next(c for c in doc['clusters'] if c['definition']['name']=='J 12 free')
inp, r0 = solve(cJ)
inp2, r1 = solve(cJ, imposed_tilt=math.degrees(r0.phi_free_hang))
print(f"  tilt imposé = pendaison libre : pickup x={r1.pickup_local[0]:.3e} mm, tirette={r1.tie_tension}, écart max forces={max(abs(norm(x.f_anchor_g)-norm(y.f_anchor_g)) for x,y in zip(r0.joints,r1.joints)):.3e} N")
# linéarité : masses x2 (à assiette imposée pour figer la géométrie)
inp, ra = solve(cJ, imposed_tilt=-1.9)
inpb = copy.deepcopy(inp)
for g in inpb.geos: g.mass_kg *= 2
rb = solve_cluster(inpb)
print("  masses ×2 : ratio anchor", [round(norm(y.f_anchor_g)/norm(x.f_anchor_g),9) for x,y in zip(ra.joints, rb.joints)][:4], "tirette", ra.tie_tension, rb.tie_tension)
# superposition : masse sur un seul caisson
tot = None
base = copy.deepcopy(inp)
for gg in base.geos: gg.mass_kg = 1e-9
for kk in range(len(inp.geos)):
    one = copy.deepcopy(base); one.geos[kk].mass_kg = inp.geos[kk].mass_kg
    rr = solve_cluster(one)
    vals = np.array([np.concatenate([j.f_orientation_g, j.f_pivot_g, j.f_anchor_g]) for j in rr.joints])
    tot = vals if tot is None else tot + vals
full = np.array([np.concatenate([j.f_orientation_g, j.f_pivot_g, j.f_anchor_g]) for j in ra.joints])
print("  superposition (somme des 12 cas à une masse) vs complet : écart max", np.abs(tot-full).max(), "N")
# rotation globale : même inclinaison -> mêmes efforts locaux ; ici : pendaison libre vs assiette imposée égale (déjà) ; et k_dyn 1.1 vs 1.3 -> ratio exact
r11 = solve_cluster(inp, k_dyn=1.1)
print("  k_dyn 1.1/1.3 : ratio", norm(r11.joints[0].f_anchor_g)/norm(ra.joints[0].f_anchor_g), "attendu", 1.1/1.3)

print("\n=== 5. Comportements §2.12 ===")
c = next(c for c in src['clusters'] if c['definition']['name'].startswith('Grappe 14u'))
inp, r = solve.__wrapped__(c) if hasattr(solve,'__wrapped__') else (inputs_from_export(src, c), None)
r = solve_cluster(inp); b = r.bumper; j0 = r.joints[0]
e_axis=inp.geos[0].e_axis
print(f"  bumper: f_ori(total)={norm(b['f_orientation_g'])*2:.0f} N, bras trou haut->G paire = {norm(b['an']-(b['anchor']+b['latch'])/2):.1f} mm, M_pair={b['pair_moment_nm']:.1f} N·m/flanc, anchor={norm(b['f_pair_anchor_g']):.0f}")
print(f"  J0    : f_ori(total)={norm(j0.f_orientation_g)*2:.0f} N, bras couronne->G paire = {norm(j0.bo-(j0.an+j0.lt)/2):.1f} mm, M_pair={j0.bar_moment_at_pair_nm:.1f} N·m/flanc, anchor={norm(j0.f_anchor_g):.0f}")
# composante transverse (celle qui fait le moment) pour les deux
for lab, F, pt, an, lt in (('bumper', -b['f_orientation_g']*2, b['an'], b['anchor'], b['latch']), ('J0', -j0.f_orientation_g*2, j0.bo, j0.an, j0.lt)):
    ea = unit(an-lt); ef = np.array([-ea[1], ea[0]]); G=(an+lt)/2
    print(f"    {lab}: axial={np.dot(F,ea):.0f} transverse={np.dot(F,ef):.0f}  bras along={np.dot(pt-G,ea):.1f} lat={np.dot(pt-G,ef):.2f}  M=(along*Ft - lat*Fa)={np.dot(pt-G,ea)*np.dot(F,ef)-np.dot(pt-G,ef)*np.dot(F,ea):.0f} N·mm")
print("  marches de rangée : M_pair par jonction", [round(j.bar_moment_at_pair_nm) for j in r.joints], "rangées", [j.row for j in r.joints])
# sans tirette, même géométrie ? on ne peut pas (pickup hors barre). Variante : même grappe en pendaison libre.
inp2 = copy.deepcopy(inp); inp2.imposed_tilt=None; inp2.tie_angle=None
r2 = solve_cluster(inp2)
print("  même grappe en pendaison libre : M_pair", [round(j.bar_moment_at_pair_nm) for j in r2.joints], "anchor", [round(norm(j.f_anchor_g)) for j in r2.joints])
print("  avec tirette 180°           : anchor", [round(norm(j.f_anchor_g)) for j in r.joints])
# moment de la tirette non compensé en bas : moment de la tirette autour de la couronne de la dernière jonction
q = r.tie_point_global; T = r.tie_dir*r.tie_tension; jl = r.joints[-1]
print(f"  moment tirette autour de bo(J12) = {cross(q-jl.bo, T)/1000:.1f} N·m ; poids dernier caisson autour de bo = {cross(r.placed[-1].to_global(inp.geos[-1].cg)-jl.bo, v(0,-inp.geos[-1].mass_kg*S['gravity']*S['dynamicFactor']))/1000:.1f} N·m")

print("\n=== 6. Stack : bumper 2 pions (code) vs bielle+barre 0° (référence) ===")
for name in ('stack 3 droit 0', 'stack 3 bas 20', 'stack 4 cca bas 20'):
    c = next(c for c in doc['clusters'] if c['definition']['name']==name)
    inp, r = solve(c); bv = c['result']['bumperView']
    Wt = r.mass*S['gravity']*S['dynamicFactor']
    # torseur exact bumper -> dernier caisson (repère global), réduit au coin avant-bas
    Wb = r.statics.get('WBS')
    # répartition bielle+barre : résoudre le petit système 3 inconnues (λ, Fc 2) sur le dernier caisson... équivalent : mêmes équations que le vol, poids = -tout ce qui est au-dessus.
    gN = inp.geos[-1]; pN = r.placed[-1]
    rb = next(bb for bb in inp.bumper['rearBars'] if abs(bb['tiltDeg'])<1e-6)
    # en stack le bumper est sous le caisson : bielle entre hb (caisson) et le pion avant du bumper, barre du trou haut (bumper) à la paire... la paire est côté couronne du caisson du bas ? Non : côté bumper la géométrie est symétrique par retournement — on suppose la barre 0° montée à l'envers : trou haut sur le bumper, paire ancrage/verrou = trous ancrage/verrou du caisson du bas.
    # Hypothèse à confirmer (question Q4). On utilise la même construction que le vol, miroir vertical autour du centre du caisson.
    print(f"  {name}: réaction sol {Wt:.0f} N ; code pions: avant {bv['pivotForceN']:.0f} N arrière {bv['orientationForceN']:.0f} N  pinPairMoment {bv['pinPairMomentNm']:.1f} N·m ; résultante sol décalée de {-r.statics.get('G')[2]/r.statics.get('G')[1]:.0f} mm du milieu (demi-profondeur {inp.bumper['depth']/2})")
    print(f"     torseur exact bumper->caisson bas au coin avant-bas : F={Wb[:2]} M={Wb[2]/1000:.1f} N·m")

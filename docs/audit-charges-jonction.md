# Calcul des charges de jonction SA303 — dossier d'audit

Objet : soumettre à relecture externe le calcul des efforts transmis à chaque
jonction entre deux caissons, et la convention dans laquelle leur direction est
rendue.

Périmètre : **statique de la jonction**, et les vérifications de section qui en
découlent — goupilles (`checks::sandwich`) et flexion de la barre arrière
(`checks::bar`). Sont hors périmètre la sélection des pires cas
(`worst_cases_selector`) et l'acoustique (`wst`).

| | |
|---|---|
| Code audité | `crates/sa303-core/src/cluster/joint.rs`, `src/cluster/kinematics.rs`, `src/speaker/geometry.rs`, `src/checks/` |
| Fonction d'entrée | `compute_joint(&JointInput) -> Result<JointResult, JointInconsistency>` |
| Unités | mm, N, kg, degrés en interface / radians en interne |
| Tests | `tests/bielle.rs` (cinématique), `tests/golden.rs` (équilibre, valeurs de référence) |

---

## 1. Convention de repère et d'angle

Repère du caisson : **origine au centre**, **x vers l'arrière**, **y vers le
haut**. Le repère global est celui du caisson du haut de la grappe, non tourné.

`φ` est l'inclinaison absolue d'un caisson (radians, sens trigonométrique). Le
passage local → global est `v.rotate(φ)`, l'inverse `v.rotate_transpose(φ)`.

**Piège d'audit.** La convention d'*angle rendu* n'est pas celle du repère. Les
champs `…AngleDeg` sortent de `vector::angle_of` :

```rust
/// Convention de sortie : 0° vers le bas, horaire, 90° vers l'avant.
pub fn angle_of(v: Vec2) -> f64 {
    let a = (-v.x).atan2(-v.y).to_degrees();
    ((a % 360.0) + 360.0) % 360.0
}
```

Donc **0° = vers le bas**, **90° = vers l'avant (−x)**, **180° = vers le haut**,
**270° = vers l'arrière**. Une force de gravité pure rend 0°. C'est la
convention de lecture sur plan de l'atelier, pas l'atan2 mathématique ; les
comparer directement est l'erreur la plus probable en relecture.

---

## 2. Modèle mécanique de la jonction

Deux caissons voisins sont reliés par **deux** liaisons, et deux seulement.

### 2.1 La bielle avant — élément à deux forces

Goupillée en deux points : en haut sur le trou avant-bas (`hb`) du caisson
supérieur, en bas sur le trou avant-haut (`ht`) du caisson inférieur. Étant un
élément à deux forces, **sa ligne d'action passe par ses deux goupilles**.

Le partage est **égal et imposé par construction** : pour un splay `k`, la
bielle tourne de `k/2` et le caisson inférieur de `k/2` de plus. La goupille
basse n'est donc **pas un point fixe du caisson** — elle décrit un arc :

```rust
pub fn pv_at(&self, splay_deg: f64) -> Vec2 {
    let h = (splay_deg / 2.0).to_radians();
    Vec2::new(
        self.hb.x + self.bielle_entraxe * h.sin(),
        self.hb.y - self.bielle_entraxe * h.cos(),
    )
}
```

→ **1 inconnue scalaire** (l'intensité `λ`).

### 2.2 La barre arrière — encastrée sur le caisson du bas

Elle est goupillée en **deux** points (ancrage et verrou) dans le caisson
inférieur, ce qui la rend rigide par rapport à lui. Elle lui transmet donc une
force **et un moment**, et sa ligne d'action n'a aucune raison de passer par
l'axe couronne–ancrage. Son seul lien avec le caisson supérieur est la goupille
de couronne : une articulation simple, **moment nul**, force de direction
quelconque.

L'ordre le long de la barre est verrou, ancrage, épaulement, couronne : le
premier pion rencontré depuis la couronne est donc l'**ancrage**, et c'est là
que le moment culmine.

→ **2 inconnues** (le vecteur d'effort à la couronne).

### 2.3 Bilan

3 inconnues, 3 équations d'équilibre plan : **isostatique**.

> **Point d'attention n°1.** Ce modèle a remplacé un modèle antérieur où la
> barre arrière était traitée en élément à deux forces et le point avant en
> pivot fixe. Les deux rôles ont *échangé*. Toute note de calcul antérieure à ce
> changement décrit une autre structure.

---

## 3. Géométrie de la jonction

Les points de la jonction, exprimés dans le repère du **caisson du haut**, puis
portés en global. `ha` = demi-angle du caisson (10°), `splay0_angle` = 5°,
`R` = 680 mm (couronne extérieure, splays pairs) ou 660 mm (intérieure, impairs).

| Point | Définition | Formule |
|---|---|---|
| `pa` | goupille haute de bielle | `hb = (hinge.x, −hinge.y)` |
| `pb` | goupille basse de bielle | `pv_at(k)` — voir §2.1 |
| `bo` | goupille de couronne | `pv_at(k) + R·(cos(ha+5+k), sin(ha+5+k))` |
| `an` | ancrage de la barre | porté depuis `polar(ht, 680, −13°)` |
| `lt` | verrou de la barre | porté depuis `polar(ht, 710,845, −20,8453°)` |

> **Point d'attention n°4 — plus de rayon commun.** Le verrou n'est **pas** sur
> le cercle de 680 mm de l'ancrage : il est à 710,845 mm, au bout de l'axe de
> barre. Toute écriture qui les place sur un rayon commun et ne les distingue
> que par un angle est fausse. Ancrage et verrou sont donc donnés en polaires
> **indépendantes** depuis `ht`, et portés dans le repère du caisson du haut par
> la formule générale :
>
> ```rust
> fn carry(&self, local: Vec2, splay_deg: f64) -> Vec2 {
>     self.pv_at(splay_deg) + (local - self.ht).rotate(splay_deg.to_radians())
> }
> ```

### 3.1 La barre arrière

L'ordre le long de la barre, depuis le petit bout : **verrou, ancrage,
épaulement, couronne**. Les abscisses sont comptées depuis ce petit bout, le
latéral positif vers l'avant du caisson.

| | abscisse | latéral | largeur locale |
|---|---|---|---|
| verrou | 14,500 | 0 | 40 |
| ancrage | 114,500 | 0 | 40 |
| épaulement | 380,000 | — | 40 → 55 |
| `up660` (splays impairs) | 438,675 | **+19,406** | 55 |
| `up680` (splays pairs) | 443,514 | 0 | 55 |

Longueur 458,514 mm, épaisseur 10 mm, perçage Ø 12,08. La longueur est pilotée
par l'entraxe de paire : `length = 443,514 + latch_offset − 85`.

**L'élargissement est d'un seul côté.** Le bord arrière est une droite sur toute
la longueur ; c'est le bord avant qui s'écarte à partir de l'épaulement. C'est
ce qui loge `up660`, déporté de 19,4 mm, en lui laissant 15,6 mm de bord.

**`up660` n'est pas sur l'axe** — conséquence directe sur le calcul, voir §7.2.

### 3.2 Invariants vérifiés au chargement

`check_rear_bar` ne suppose aucun de ces points, il les contrôle :

| Invariant | Valeur | Tolérance |
|---|---|---|
| \|ancrage − verrou\| = `latch_offset` | 100,000 mm | 0,05 |
| verrou sur la droite ancrage → couronne 680 | 0 mm hors axe | **0,01** |
| \|ancrage − `up680`\| | 329,014 mm | 0,05 |
| \|ancrage − `up660`\| | 324,756 mm | 0,05 |
| déport latéral `up660` / `up680` | +19,406 / 0 mm | 0,05 |
| épaulement coté depuis les deux bouts | `length − wide_length` | 0,05 |

Puis les distances au bord, signalées sous `1,2·d₀ = 14,496 mm` : e₁ verrou 14,5,
e₁ couronne 15,0, e₂ `up660` 15,6. **Le verrou passe à 4 µm près** — il est
exactement sur la limite réglementaire, et un arrondi de cotation le ferait
basculer.

Sur la géométrie Fusion livrée, aucun avertissement n'est remonté.

**La couronne est à une place fixe dans le repère du caisson qui porte la
paire.** En `ht + R∠(ha + splay0)`, indépendamment du splay : le caisson du bas
tourne avec elle. C'est pour ça qu'une seule barre dessert tous les crans d'une
couronne, et c'est cette position qui sert de référence à l'axe.

Les huit positions de couronne ont été recoupées contre le perçage relevé en
atelier : **écart < 0,003 mm** (`every_crown_hole_matches_the_drilled_table`).

## 4. Chaînage cinématique

La position de chaque caisson découle du précédent en faisant coïncider la
goupille basse de bielle du caisson du haut avec le `ht` de celui du bas :

```rust
let phi_next = phi + splays_deg[i].to_radians();
let pv = speaker.geo.pv_at(splays_deg[i]);
o = (o + pv.rotate(phi)) - chain[i + 1].geo.ht.rotate(phi_next);
phi = phi_next;
```

> **Point d'attention n°2.** `pv_at(splay)` et non un `pv` constant. C'est ce
> qui fait que deux caissons ne pivotent pas autour d'un point commun : le
> décalage qui en résulte est petit (< 0,05 mm en avant/arrière sur 0–5°) mais
> il se cumule sur toute la grappe.

---

## 5. Corps libre et torseur extérieur

Le corps libre dépend du compartiment :

- **Vol** (`Flown`) : les caissons **sous** la jonction, `i+1 … n−1`.
- **Stack** (`Stacked`) : les caissons **au-dessus**, `0 … i`.

```rust
let mut w_total = 0.0;
let mut sum = Vec2::ZERO;
for k in lo..=hi {
    let w = input.chain[k].mass_kg * input.g * input.k_dyn;   // k_dyn = 1,3
    w_total += w;
    sum = sum + input.speakers[k].cg * w;
}
let cm = sum * (1.0 / w_total);                                // barycentre pondéré masse
```

Le poids est sommé **enceinte par enceinte** avec le CG de chacune : une grappe
peut mélanger des modèles de masses et de CG différents, donc jamais une masse
unique multipliée par un nombre de caissons.

Moment extérieur pris **à la goupille de couronne** — c'est là que l'inconnue
vectorielle disparaît de l'équation de moment :

```rust
let mut rext = Vec2::new(0.0, -w_total);
let mut mext = (cm - bo).cross(rext);

if let (Compartment::Flown, Some(tie)) = (input.compartment, input.tie) {
    let last = input.speakers[n - 1];
    let q = last.o + tie.point_local.rotate(last.phi);
    rext = rext + tie.force;
    mext += (q - bo).cross(tie.force);
}
```

La tirette basse n'existe qu'en vol, et s'applique au dernier caisson.

---

## 6. Résolution

```rust
// Bielle : direction imposée par ses deux goupilles, une seule inconnue.
let u = (pb - pa).normalize();
let bielle_lever = (pb - bo).cross(u);
let lambda = -mext / bielle_lever;
let f_piv = u * lambda;

// La couronne reprend tout le reste.
let f_ori = -rext - f_piv;
```

Soit, en clair :

```
ΣM/bo = 0   →   λ · [(pb − bo) × u] + M_ext/bo = 0
ΣF    = 0   →   F_couronne = −R_ext − λ·u
```

`bielle_lever` est le bras de la ligne d'action de la bielle autour de la
goupille de couronne. Il vaut 629 mm sur la géométrie SA303 et ne s'annule pour
aucun splay de la grille. Une garde le vérifie tout de même : sous 1 mm, la
jonction remonte une erreur plutôt qu'un `λ` infini puis des efforts NaN — les
comparaisons sur NaN étant fausses, un NaN traverserait ensuite tous les seuils
de vérification sans en déclencher un seul.

---

## 7. Exemple numérique vérifiable à la main

Grappe de 3 caissons SA303 identiques, suspendue, splays `[5°, 10°]`,
`φ_initial = 0`, `g = 9,80665`, `k_dyn = 1,3`, masse 83,695 kg, CG local
`(10,84 ; 11,91)`. Jonction n°1 (`joint_index = 0`), corps libre = caissons 2 et 3.

Reproduit par `tests/bielle.rs::golden_bar_loads_on_the_reference_joint`.

Cinématique :

| | o (mm) | φ (rad) | CG global (mm) |
|---|---|---|---|
| caisson 1 | (0,0000 ; 0,0000) | 0,000000 | (10,8400 ; 11,9100) |
| caisson 2 | (22,7853 ; −521,8681) | 0,087266 | (32,5460 ; −509,0586) |
| caisson 3 | (108,1211 ; −1005,8319) | 0,261799 | (115,5092 ; −991,5221) |

Points de la jonction :

```
pa = (−338,4310 ; −257,1220)      pb = (−336,7676 ; −295,2207)
bo = ( 283,4296 ;  −69,4874)      an = ( 336,6147 ; −389,8584)
```

### 7.1 Résolution de la jonction

```
W      = 2 133,9957 N          cm = (74,0276 ; −750,2904)
M_ext/bo = 446 862,86 N·mm
u      = (0,043619 ; −0,98650)      ← inclinaison 2,5° = splay/2 ✓
bras   = 629,4532 mm
λ      = −709,9223 N

F_bielle   = (−30,9664 ;   709,2466)   |709,92 N|   (effort total)
F_couronne = ( 30,9664 ; 1 424,7491)   |1 425,09 N| (effort total)
```

Contrôles : `F_bielle + F_couronne = (0 ; 2 133,9957) = −R_ext`, et la direction
de `F_bielle` fait exactement **2,5°** avec la verticale, soit `splay/2`, comme
l'impose l'élément à deux forces.

### 7.2 La barre arrière

Splay 5, donc splay **impair**, donc couronne intérieure : l'effort s'applique à
`up660`, **déporté de 19,406 mm de l'axe de barre**.

Le repère de barre est l'axe ancrage → `up680`, origine au petit bout. Tout ce
qui suit est **par flanc** (× 0,5).

```
N (axial)      = −706,9 N     négatif = barre tendue
V (transverse) = − 89,9 N     |F| = 712,5 N
```

> **Point d'attention n°5 — le moment n'est pas `|V| × abscisse`.** La couronne
> étant hors de l'axe sur les splays impairs, il faut le produit vectoriel
> complet `(P_section − P_application) × F`. Une abscisse seule perdrait le bras
> transversal, donc la part de moment qu'il introduit.

```
bras ancrage → couronne (impair)  = 324,756 mm
M à l'ancrage                     = 42,847 N·m     ← SECTION CRITIQUE
M au verrou                       = 0
M aux trous de couronne           = 0  (articulation)
```

**Le maximum est à l'ancrage**, premier pion rencontré depuis la couronne. Entre
la couronne et lui, la barre ne voit qu'un seul effort et le moment croît depuis
zéro ; au-delà, la réaction de la paire le fait redescendre, et il est nul au
verrou — il ne reste derrière lui que 14,5 mm de barre où rien ne s'applique.

C'est l'**inverse** de la géométrie précédente, où le verrou était le premier
pion. Les deux ont échangé leur rang le long de la barre.

> **Invariant utile en relecture.** Le moment à l'ancrage est indépendant du
> repère : c'est `|F|` fois la distance de l'ancrage à la ligne d'action. La
> décomposition `N`/`V`, elle, dépend de l'axe choisi. Un relecteur qui trouve
> un autre `N` mais le même `M` n'a pas une erreur de calcul, il a un autre axe.

### 7.3 La paire ancrage/verrou

Entraxe `d = 100,000 mm`. Répartition élastique à raideurs égales :

```
part directe = |F_couronne|/2 par flanc  = 356,3 N
couple       = M_barycentre / d          = 473,4 N
F_ancrage = 627,4 N à 229,7°
F_verrou  = 555,4 N à 123,5°
```

L'entraxe est passé de 23,7 mm à 100 mm : le couple qui reprend le moment est
donc environ **quatre fois plus petit** à moment comparable, et les efforts de
pion tombent de ~1,9 kN à ~0,6 kN. C'est le gain principal du nouveau dessin.

### 7.4 Contraintes dans la barre

Section critique : l'ancrage, largeur **40** × 10, perçage Ø 12,08 centré.

```
A_net = t(w − d0)          = 280,0 mm²
W_net = t(w³ − d0³)/(6w)   = 2 593,2 mm³

σ_ancrage = 42 847/2 593,2 + 706,9/280,0 = 16,5 + 2,5 = 19,0 MPa
```

Le module net traite le trou centré comme une **fente sur la fibre neutre**. La
formule `t(w − d0)²/6`, qui vaudrait pour un trou en fibre extrême, donnerait un
facteur 2 sur le dimensionnement. Contrôle : 325 N·m sur cette section donnent
**125,33 MPa**.

Pour `up660`, déporté, le module net est calculé **centroïde décalé** : retirer
de la matière hors de la fibre neutre déplace le centre de gravité, donc les deux
fibres extrêmes n'ont plus le même module et c'est la plus petite qui gouverne.
Le moment y est nul aujourd'hui ; la formule est écrite pour que déplacer un trou
ne demande pas de la retrouver dans l'urgence, et un test vérifie qu'à déport nul
elle retombe exactement sur la formule centrée.

### 7.5 Arrachement de bord

Mode distinct du matage, qui ne regarde pas où est le bord. La goupille chasse
devant elle le bloc de matière qui la sépare du bord libre, cisaillé sur ses deux
flancs : `2 × (e₂ − d₀/2) × t`, comparé à `0,6·R_m/sf`. Seule compte la
composante de l'effort **dirigée vers ce bord** — un effort parallèle au bord ne
chasse rien devant lui.

Sur le jeu de grappes réel, ce mode reste très en dessous des autres (≤ 0,12).
Il devient dimensionnant dès qu'une cote de bord se referme, ce que la largeur de
barre fait varier directement.

## 8. Ce qui est rendu, et dans quel repère

```rust
let ti = match input.compartment {           // flanc chargé
    Compartment::Flown => i,                 // celui du haut en vol
    Compartment::Stacked => i + 1,           // celui du bas en stack
};
let rt_phi = input.speakers[ti].phi;
let sg = -input.share_per_flank;             // = −0,5

let f_orientation = (f_ori * sg).rotate_transpose(rt_phi);
let f_pivot       = (f_piv * sg).rotate_transpose(rt_phi);
// La paire subit l'action de la barre : +share, pas sg.
let f_anchor = (f_anchor_g * input.share_per_flank).rotate_transpose(rt_phi);
let f_latch  = (f_latch_g  * input.share_per_flank).rotate_transpose(rt_phi);
```

> **Point d'attention n°3.** Le facteur `sg = −0,5` fait deux choses à la fois.
> Le **signe** retourne l'effort : on rend la réaction *subie par la
> quincaillerie*, pas l'action sur le corps libre. Le **0,5** partage entre les
> deux flancs. Les valeurs rendues sont donc **par flanc**.
>
> Les deux goupilles de la paire, elles, reçoivent `+share` et non `sg` : elles
> subissent déjà l'action de la barre. Un `sg` les sortirait à l'envers des deux
> autres liaisons.

### 8.1 Efforts de liaison

| Champ | Contenu |
|---|---|
| `f_pivot`, `f_pivot_n`, `f_pivot_angle_deg` | effort de **bielle**, par flanc |
| `f_orientation`, `f_orientation_n`, `f_orientation_angle_deg` | effort à la **goupille de couronne**, par flanc |
| `f_anchor`, `f_anchor_n`, `f_anchor_angle_deg` | effort sur la goupille d'**ancrage**, par flanc |
| `f_latch`, `f_latch_n`, `f_latch_angle_deg` | effort sur la goupille de **verrou**, par flanc |

### 8.2 Sollicitations de la barre

| Champ | Contenu |
|---|---|
| `bar_axial_n` | effort normal, par flanc. Positif = comprimée |
| `bar_shear_n` | composante transverse, celle qui fait fléchir |
| `bar_moment_max_nm` | moment à la **section critique**, c'est-à-dire à l'**ancrage** |
| `bar_moment_max_at_mm` | son abscisse, 114,5 mm depuis le petit bout |
| `bar_moment_at_pair_nm` | moment réduit au barycentre de la paire. **Signé**, et exactement celui qui produit `f_anchor`/`f_latch` — même produit vectoriel, pas une reconstruction `\|V\| × bras` qui perdrait le bras transversal de `up660` |
| `rear_bar` | cotation de la barre de cette jonction |

`bar_axial_n` et `bar_shear_n` dépendent de l'axe de barre ; le moment n'en
dépend pas (§7.2).

### 8.2 bis Vérifications de section (`checks::bar`)

| Champ | Contenu |
|---|---|
| `sections[]` | ancrage, épaulement, verrou, couronne 680, couronne 660 |
| `…width_mm`, `…hole_offset_mm`, `…section_modulus_mm3` | ce qui a servi au calcul, rendu plutôt que sous-entendu |
| `critical_width_mm` | largeur de la section critique — **le paramètre de dimensionnement**, rendu pour qu'un balayage 40/50/60/70 se lise dans la sortie sans recouper le modèle |
| `tear_out[]` | arrachement de bord par trou, avec `edge_distance_mm` et la composante dirigée vers le bord |

### 8.3 Géométrie et contrôles

| Champ | Contenu |
|---|---|
| `crown_hole_local`, `anchor_hole_local`, `latch_hole_local` | les trois trous de barre, repère du flanc chargé, cohérents entre eux et avec les efforts — de quoi recouper un moment |
| `bielle_lever_mm` | bras qui a servi à la résolution |
| `lever_mm` | bras géométrique couronne–ancrage, **recoupement de perçage seulement** |
| `residual_n` | résidu de force. **Nul par construction** : `f_ori` est posé à `−rext − f_piv`, donc il passerait même avec un `λ` faux |
| `moment_residual_nmm` | résidu de moment, pris en `pa` et au CG — **pas** à `bo` où l'inconnue a été annulée. C'est lui qui atteste de la résolution |
| `traction` | signe de la composante axiale de la barre |
| `hinge_reversed` | `f_pivot · gravité < 0` |
| `bumper_model_legacy` | voir §9.2 |

`compute_joint` rend un `Result`. Un bras de bielle sous 1 mm est une erreur, pas
un NaN : les comparaisons sur NaN étant fausses, un NaN traverserait tous les
seuils de vérification sans en déclencher un seul.

### 8.4 Vérifications

`checks::sandwich` (cisaillement double, matage barre, matage flanc, pire des
trois, `R_m/4`) est appliqué aux **quatre** liaisons : couronne, bielle, ancrage,
verrou. `checks::bar` y ajoute la flexion composée de la barre à trois sections.
`utilization_worst` retient le pire des cinq chemins.

---

## 9. Limites connues, à trancher par l'audit

Le résultat est **isostatique** : trois inconnues, trois équations d'équilibre.
Il ne dépend donc ni des jeux dans les trous, ni des raideurs relatives des
pièces. La seule exception est la répartition sur la paire ancrage/verrou
(§9.1), qui est hyperstatique.

### 9.1 Répartition sur la paire — résolue, mais sous hypothèse de raideurs

Deux goupilles dans un même corps rigide, c'est 6 inconnues pour 3 équations.
Le code retient la **solution élastique à raideurs égales** : part directe
partagée en deux, plus un couple perpendiculaire à la ligne ancrage-verrou.

Ce n'est pas arbitraire — les deux goupilles ont le même diamètre, la même
épaisseur de barre et le même flanc, donc la même raideur — mais ça reste la
seule grandeur du dossier qui ne sorte pas de la seule statique. Un jeu
différentiel entre les deux trous redistribuerait le couple.

### 9.2 Jonction bumper ↔ premier caisson — modèle antérieur

`solver::compute_bumper_loads` suit encore le schéma bras à deux forces + pivot
fixe au `ht`. C'est une liaison différente, non décrite par la spécification de
la liaison à bielle. Les jonctions concernées portent `bumper_model_legacy = true`
et **leurs grandeurs de barre ne décrivent pas cette liaison**.

### 9.3 La flexion de barre est LE mode dimensionnant à 40 de large

Ce n'est plus une limite mais un résultat, et il est net. Sur les quatre grappes
de référence du catalogue, **largeur de barre 40 mm** :

| grappe | flexion | pions | arrachement | pire |
|---|---|---|---|---|
| 12u 40d-88d | **2,674** | 0,922 | 0,122 | 2,674 (J11) |
| 14u 0-60m | **2,414** | 0,831 | 0,110 | 2,414 (J13) |
| banane 12u −20/46 | **1,779** | 0,617 | 0,082 | 1,779 (J1) |
| 14u 0-150m | **1,571** | 0,581 | 0,060 | 1,571 (J2) |

La barre est dépassée d'un facteur 1,6 à 2,7 partout, alors que **la paire ne
gouverne plus nulle part** : elle reste sous 0,93. Avec l'entraxe porté de
23,7 mm à 100 mm, le couple qui reprend le moment a été divisé par quatre — le
problème s'est entièrement déplacé du pion vers la section de barre.

À **largeur 70 mm** entre l'ancrage et la couronne, tout repasse :

| grappe | flexion | pire | chemin résiduel |
|---|---|---|---|
| 12u 40d-88d | 0,866 | 0,922 | ancrage (matage flanc 4 mm) |
| 14u 0-60m | 0,783 | 0,831 | ancrage (matage flanc 4 mm) |
| banane 12u −20/46 | 0,573 | 0,617 | ancrage (matage flanc 4 mm) |
| 14u 0-150m | 0,548 | **0,841** | **couronne**, pas ancrage |

Trois grappes sur quatre laissent le matage du flanc à l'ancrage comme critère
résiduel. La quatrième, 14u 0-150m, est gouvernée par la **goupille de
couronne** à 0,841 : 14 caissons à faibles splays alignent la barre sur la
charge, d'où un effort normal de 10,3 kN que le pion de couronne encaisse
entier, là où la paire le partage en deux.

Élargir la barre doit se faire **vers l'arrière**, bord avant fixe : le bord
avant est à 35 mm de l'axe et `up660` vit à 19,4 mm, donc le ramener
étoufferait ce trou (voir `dump_catalogue_load_table`).

Régénérer ces tableaux :

```text
cargo test -p sa303 dump_catalogue_load_table -- --ignored --nocapture
```

### 9.4 Concordance barre / perçage

Les trois trous de l'axe — verrou, ancrage, `up680` — sont **alignés** à mieux
que 0,01 mm, ce que `check_rear_bar` contrôle plutôt que de le supposer. C'est
la condition d'existence d'une barre droite.

Les deux couronnes ne sont pas séparées le long de l'axe mais **en travers** :
`up660` et `up680` sont à 4,84 mm l'un de l'autre en abscisse, et surtout à
**19,4 mm en latéral**. C'est ce déport qui justifie l'élargissement à 55 et qui
met l'effort de couronne hors de l'axe sur les splays impairs (§7.2).

La géométrie Fusion livrée ne remonte **aucun** avertissement de concordance.
Seule réserve : `e₁` au verrou vaut 14,500 mm pour un minimum de 14,496 — la
cote est exactement sur la limite, à 4 µm près.

### 9.4 bis Incohérence connue sur la plage de tirette

`tie_valid_angle_range_deg` ne borne que la statique — les directions où la
tension reste positive. Le test de collision est appliqué séparément, au moment
de retenir l'angle. Les deux ne se parlent pas, et depuis que le point
d'accroche est passé sur `crown(0)` — un vrai trou de l'enceinte, au lieu d'un
point situé 170 mm sous son plancher — une partie des directions annoncées
traverse les enceintes du dessus.

Test `user_can_pick_their_own_angle_inside_the_reported_range`, marqué `#[ignore]`
avec cette raison. Trancher demande de décider si la plage annoncée doit être
filtrée par la collision avant d'être affichée, ou si le refus reste au moment
du choix.

### 9.5 Valeurs de référence de `golden.rs`

Elles sortent du modèle lui-même, pas d'un recoupement indépendant. Ce qui est
vérifié indépendamment : la cinématique (contre le perçage relevé), l'équilibre
en moment, et les valeurs de barre du §7 (recoupées à la main).

### 9.6 Effets négligés, assumés

Élasticité des pièces (tout est rigide hors §9.1), jeu dans les trous,
frottement, effets hors plan, flambement de la barre en compression. Le facteur
dynamique `k_dyn = 1,3` et le coefficient 4:1 sont des réglages utilisateur.

La tirette entre dans l'équilibre **sans** `k_dyn` : sa tension est calculée en
amont à partir d'un poids déjà dynamisé, donc le facteur y est déjà.

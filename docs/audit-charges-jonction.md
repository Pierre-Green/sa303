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

L'ordre le long de la barre est verrou, ancrage, couronne : le
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

### 3.1 La barre arrière — profil v3

L'ordre le long de la barre, depuis le petit bout : **verrou, ancrage,
couronne**. Les abscisses sont comptées depuis ce petit bout, le latéral positif
vers l'avant du caisson.

| | abscisse | latéral | largeur locale |
|---|---|---|---|
| verrou | 16,000 | 0 | 40 |
| ancrage | 116,000 | 0 | **70** |
| `up660` (splays impairs) | 440,175 | **+19,406** | 70 |
| `up680` (splays pairs) | 445,014 | 0 | 70 |

Longueur 460,014 mm, épaisseur 10 mm S355, perçage Ø 12,08, masse 2,32 kg.

**Le profil de largeur n'est plus constant par morceaux** : 40 mm de 0 à 30,
pente rectiligne jusqu'à 70 mm à l'abscisse 100, puis 70 jusqu'au bout.

```
widthProfile: [[0, 40], [30, 40], [100, 70], [460.014, 70]]
rearEdgeOffset: 20
```

**Tout l'élargissement va vers l'avant.** Le bord arrière est une droite
parallèle à l'axe des trous, à 20 mm de lui sur toute la longueur. C'est cette
droite qui doit rester devant la face arrière du caisson, cotée `rearFaceX = 351`
— et non `depth/2 = 350`.

> **Point d'attention n°6 — les trous ne sont plus sur la fibre neutre.** L'axe
> des trous reste à 20 mm du bord arrière ; quand la barre s'élargit à 70, le
> bord avant part à 50. Le trou d'ancrage est alors à **18,1 mm du centroïde** de
> la section nette.
>
> Le module net exact vaut **6 597 mm³** ; la formule « fente sur fibre neutre »
> `t(w³ − d0³)/(6w)` en donnerait 8 125. L'utiliser **sous-estimerait la
> contrainte de 22 %** à la section critique. Elle ne vaut qu'au verrou, où la
> largeur de 40 est symétrique autour de l'axe.

### 3.2 Invariants vérifiés au chargement

`check_rear_bar` ne suppose aucun de ces points, il les contrôle :

| Invariant | Valeur | Tolérance |
|---|---|---|
| \|ancrage − verrou\| = `latch_offset` | 100,000 mm | 0,05 |
| verrou sur la droite ancrage → couronne 680 | 0 mm hors axe | **0,01** |
| \|ancrage − `up680`\| | 329,014 mm | 0,05 |
| \|ancrage − `up660`\| | 324,756 mm | 0,05 |
| déport latéral `up660` / `up680` | +19,406 / 0 mm | 0,05 |
| profil de largeur monotone et couvrant la barre | 460,014 mm | 0,05 |

Puis, en avertissement : distances au bord sous `1,2·d₀ = 14,496 mm` (e₁ verrou
**16,0**, e₁ couronne 15,0, e₂ `up660` 30,6 vers l'avant), et le bord arrière
au-delà de la face arrière du caisson.

> **La marge de bord arrière se mesure au petit bout, pas à l'ancrage.** Le bord
> arrière est parallèle à l'axe de barre, qui s'incline vers l'avant en montant :
> son abscisse recule donc à mesure qu'on s'éloigne de la paire. Le point le plus
> en arrière est le bout du verrou, à **4,84 mm** de la face — contre 6,86 mm si
> on mesurait à l'ancrage. Et la marge ne dépend pas du splay : la barre est
> goupillée sur le caisson, elle tourne avec lui.

Sur la géométrie Fusion livrée, aucun avertissement n'est remonté.

**La couronne est à une place fixe dans le repère du caisson qui porte la
paire**, en `ht + R∠(ha + splay0)`, indépendamment du splay : le caisson du bas
tourne avec elle. C'est pour ça qu'une seule barre dessert tous les crans d'une
couronne.

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

> **Point d'attention n°5 — le moment n'est pas `|V| × abscisse`.** La couronne
> étant hors de l'axe sur les splays impairs, il faut le produit vectoriel
> complet `(P_section − P_application) × F`. Une abscisse seule perdrait le bras
> transversal, donc la part de moment qu'il introduit.

Le diagramme se lit en **deux branches**, balayées au millimètre :

```
côté paire     (16 → 116) : M(a) = (P(a) − P_verrou)  × F_verrou
côté console  (116 → 445) : M(a) = −(P(a) − P_couronne) × F_couronne

M au verrou (a = 16)      =  0        rien derrière lui
M à mi-paire (a = 66)     = 21,4 N·m  le tronçon est droit
M à l'ancrage (a = 116)   = 42,85 N·m ← SECTION CRITIQUE
M aux trous de couronne   =  0        articulation
```

Les deux branches **se raccordent exactement à l'ancrage**. Ce n'est pas une
commodité d'écriture : c'est l'équilibre de la barre, donc le contrôle que la
répartition élastique sur la paire a été correctement résolue en amont. Un écart
y signalerait une erreur dans `compute_joint`, pas dans `checks::bar`.

> Ce raccord n'est exact qu'à condition que les bras de levier **et** les efforts
> viennent de la même source. Les points d'application sont donc pris sur la
> géométrie calculée, pas sur la cotation déclarée de la barre : les deux ne
> coïncident qu'à la tolérance d'ajustement (0,05 mm), et le mélange laissait
> 1,2 N·mm de résidu — trop peu pour être une faute de physique, assez pour
> rendre le contrôle inexploitable.

> **Invariant utile en relecture.** Le moment est indépendant du repère : c'est
> `|F|` fois la distance de la section à la ligne d'action. La décomposition
> `N`/`V`, elle, dépend de l'axe choisi. Un relecteur qui trouve un autre `N`
> mais le même `M` n'a pas une erreur de calcul, il a un autre axe.

### 7.3 La paire ancrage/verrou

Entraxe `d = 100,000 mm`. Répartition élastique à raideurs égales :

```
part directe = |F_couronne|/2 par flanc  = 356,3 N
couple       = M_barycentre / d          ≈ 473 N
F_ancrage ≈ 627 N     F_verrou ≈ 555 N
```

L'entraxe est passé de 23,7 mm à 100 mm : le couple est donc environ **quatre
fois plus petit** à moment comparable, et les efforts de pion tombent de ~1,9 kN
à ~0,6 kN. C'est le gain principal du dessin.

### 7.4 Contraintes dans la barre

Section critique : l'**ancrage**, largeur **70** × 10, perçage Ø 12,08 **à 20 mm
du bord arrière**, donc excentré.

```
A_net = t(w − d0)                         = 579,2 mm²
W_net (excentré, centroïde décalé)        = 6 596,5 mm³
W     (si le trou était centré)           = 8 124,7 mm³   ← faux ici

σ_ancrage = 42 848/6 596,5 + N/A          = 7,7 MPa (jonction de référence)
```

Voir le point d'attention n°6 (§3.1) : la formule centrée sous-estimerait de
22 %.

### 7.5 Arrachement de bord

Mode distinct du matage, qui ne regarde pas où est le bord. La goupille chasse
devant elle le bloc de matière qui la sépare du bord libre, cisaillé sur ses deux
flancs : `2 × (e₂ − d₀/2) × t`, comparé à `0,6·R_m/sf`, avec la **largeur locale**
au droit du trou. Seule compte la composante dirigée vers ce bord — un effort
parallèle au bord ne chasse rien devant lui.

Sur le parc réel ce mode reste très en dessous des autres (≤ 0,12).

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

### 8.2 Sollicitations et vérification de la barre

| Champ | Contenu |
|---|---|
| `bar_axial_n` / `bar_shear_n` | effort normal et transverse à la couronne, par flanc. Dépendent de l'axe de barre ; le moment, non |
| `bar_moment_max_nm` | moment à la section critique, l'**ancrage** |
| `bar_moment_at_pair_nm` | moment réduit au barycentre de la paire. Signé, et exactement celui qui produit `f_anchor`/`f_latch` |
| `bar_load_crown` / `bar_load_latch` | les deux efforts que la barre **reçoit**, repère barre, par flanc |
| `bar_point_crown` / `_latch` / `_anchor` | leurs points d'application, issus de la géométrie **calculée** et non de la cotation |
| `bar_rear_edge_max_x` / `rear_face_x` | le point le plus en arrière du bord de barre, et la face qu'il ne doit pas franchir |

`checks::bar` rend ensuite :

| Champ | Contenu |
|---|---|
| `critical` | la section la plus sollicitée du balayage à 1 mm : abscisse, largeur locale, percée ou non, décalage du trou, M, N, module retenu, σ, taux |
| `profile[]` | σ(a) échantillonné tous les 5 mm, du verrou à la couronne — pour tracer la courbe sans le logiciel |
| `tear_out[]` | arrachement de bord par trou, avec `edge_distance_mm` |
| `warnings[]` | distances au bord sous le minimum, bord arrière au-delà de la face |
| `moment_continuity_nmm` | écart des deux branches à l'ancrage. **Doit être nul** : c'est le contrôle que la paire est bien résolue |

Et l'export ajoute par grappe `safety_factor` (l'inverse du taux, rapporté au
même `sf`) et `safety_factor_static`, le même recalculé à `k_dyn = 1,1` — la
valeur à comparer à Soundvision. Les réglages ne bougent pas : la grappe est
recalculée à part.

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

### 9.3 Section critique : l'ancrage percé

Le balayage continu remplace la liste fixe de sections. Sur le profil v2,
constant par morceaux, regarder les trous et la marche suffisait : entre deux
discontinuités le maximum tombe forcément sur une borne. La pente v3 (40 → 70
entre les abscisses 30 et 100) fait varier largeur **et** moment, et rien ne
garantit que leur rapport culmine à une extrémité.

Ce que le balayage donne sur `12u 40d-88d` J11, la jonction la plus chargée du
parc :

| abscisse | largeur | percé | taux |
|---|---|---|---|
| 30 | 40,0 | non | 0,37 |
| 50 | 48,6 | non | 0,59 |
| 70 | 57,1 | non | 0,69 |
| 100 | 70,0 | non | 0,72 |
| **116** | **70,0** | **oui** | **1,06** |

**La pente tient un taux quasi constant autour de 0,70** entre 70 et 100 : la
largeur y croît juste assez vite pour absorber le moment. C'est un profil bien
dessiné sur ce tronçon. Le saut à 1,06 à l'ancrage vient du perçage, et surtout
de son **excentration** (§3.1).

> **Non reproduit.** Le chiffre annoncé pour la v2 — un taux d'environ 2,0 que
> le balayage aurait attrapé sur la marche à l'abscisse 87,7 — n'a **pas** été
> recalculé ici : le profil v2 n'existe plus dans le schéma, et le reconstruire
> pour la seule contre-épreuve n'a pas été fait. L'argument du balayage tient
> sur la v3 seule (une pente, et non une marche, entre 30 et 100), mais ce
> chiffre-là reste à vérifier si vous voulez vous en servir.

Les trois grappes du parc au-dessus de 80 % :

| grappe | taux | SF | SF (k_dyn 1,1) |
|---|---|---|---|
| 12u 40d-88d | **1,058** | **3,78** | 4,47 |
| 14u 0-60m | 0,956 | 4,19 | 4,95 |
| 14u 0-150m | 0,841 | 4,76 | 5,62 |

Les 26 autres sont sous 0,74. La paire ne gouverne plus nulle part : elle
plafonne à 0,92, conséquence de l'entraxe porté à 100 mm.

Régénérer :

```text
cargo test -p sa303 dump_catalogue_load_table -- --ignored --nocapture
cargo test -p sa303 dump_audit_export_file    -- --ignored --nocapture
```

### 9.4 Concordance barre / perçage

Les trois trous de l'axe — verrou, ancrage, `up680` — sont **alignés** à mieux
que 0,01 mm, ce que `check_rear_bar` contrôle plutôt que de le supposer.

Les deux couronnes ne sont pas séparées le long de l'axe mais **en travers** :
4,84 mm d'écart en abscisse, mais **19,4 mm en latéral**. C'est ce déport qui
justifie l'élargissement à 70 et qui met l'effort de couronne hors de l'axe sur
les splays impairs.

La face arrière du caisson est cotée **351 mm** et non `depth/2 = 350` : c'est la
cote du modèle, et c'est elle que le bord arrière de la barre ne doit pas
franchir. Marge relevée : **4,84 mm**, au petit bout de la barre.

### 9.4 bis Incohérence connue sur la plage de tirette

**Toujours ouverte** : le filtrage de la plage par le test de collision n'a pas
été fait, donc le test `user_can_pick_their_own_angle_inside_the_reported_range`
reste marqué `#[ignore]`.

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

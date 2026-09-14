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

→ **2 inconnues** (le vecteur d'effort à la couronne).

### 2.3 Bilan

3 inconnues, 3 équations d'équilibre plan : **isostatique**.

> **Point d'attention n°1.** Ce modèle a remplacé un modèle antérieur où la
> barre arrière était traitée en élément à deux forces et le point avant en
> pivot fixe. Les deux rôles ont *échangé*. Toute note de calcul antérieure à ce
> changement décrit une autre structure.

---

## 3. Géométrie de la jonction

Les quatre points, exprimés dans le repère du **caisson du haut**, puis portés
en global. `ha` = demi-angle du caisson (10°), `splay0_angle` = 5°,
`anchor_angle` = 3°, `R` = 680 mm (couronne extérieure, splays pairs) ou 660 mm
(intérieure, splays impairs).

| Point | Définition | Formule |
|---|---|---|
| `pa` | goupille haute de bielle | `hb = (hinge.x, −hinge.y)` |
| `pb` | goupille basse de bielle | `pv_at(k)` — voir §2.1 |
| `bo` | goupille de couronne | `pv_at(k) + R·(cos(ha+5+k), sin(ha+5+k))` |
| `an` | ancrage de la barre | `pv_at(k) + 680·(cos(−(ha+3)+k), sin(−(ha+3)+k))` |

```rust
let pa = si.o + geo.hb.rotate(si.phi);
let pb = si.o + geo.pv_at(s).rotate(si.phi);
let bo = si.o + geo.crown(s).rotate(si.phi);
let an = si.o + geo.anchor_at(s).rotate(si.phi);
```

**Couronne et ancrage sont tous deux radiaux depuis `pv_at(k)`**, à écart
angulaire constant : leur entraxe ne dépend donc pas du splay — une seule
longueur de barre dessert tous les crans d'une même couronne. Vérifié par
`the_bar_length_does_not_depend_on_the_splay`.

Il dépend en revanche de **quelle** couronne : 329,01 mm sur l'extérieure,
324,76 sur l'intérieure. La barre porte donc deux trous de couronne (§9.4).

Le verrou, quatrième trou, est à 680 mm et −(ha + 1°) + k.

Les huit positions de couronne calculées ainsi ont été recoupées contre le
perçage relevé en atelier : **écart < 0,003 mm** sur les huit crans
(`every_crown_hole_matches_the_drilled_table`).

---

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

Splay 5, donc splay impair, donc couronne intérieure (660) : entraxe
couronne-ancrage **L = 324,756 mm**, axe `e = (0,16377 ; −0,98650)`.

Tout ce qui suit est **par flanc** (× 0,5).

```
N (axial)   = −700,2 N        négatif = barre tendue
V (transverse) = −131,9 N     c'est lui qui fait fléchir

bras couronne → verrou   = 301,81 mm      M = 39,82 N·m   ← maximum
bras couronne → ancrage  = 324,76 mm      M = 42,85 N·m   (réduction, pas un maximum)
bras couronne → barycentre = 313,28 mm    M = 41,33 N·m   ← ce que la paire reprend
```

Le moment culmine au **verrou**, première goupille rencontrée depuis la
couronne : entre la couronne et lui, la barre ne voit qu'un seul effort et le
moment croît linéairement depuis zéro ; au-delà, la réaction de la paire le fait
redescendre, et il est nul à l'ancrage — la barre s'y termine sur 15,9 mm qui ne
portent rien.

### 7.3 La paire ancrage/verrou

Entraxe `d = 23,735 mm`. Répartition élastique à raideurs égales :

```
part directe  = |F_couronne|/2 par flanc  = 356,3 N
couple        = M_barycentre / d          = 1 741,5 N        ← domine largement
F_ancrage = 1 909,9 N à 107,7°
F_verrou  = 1 839,7 N à 265,9°
```

**Le couple vaut cinq fois la part directe.** C'est la raison pour laquelle
vérifier l'ancrage sur la seule résultante de couronne le sous-estimait d'un
facteur ~3.

### 7.4 Contraintes dans la barre

Section étroite 40 × 10, perçage Ø 12,08.

```
A_net = t(w − d0)          = 280,0 mm²
W_net = t(w³ − d0³)/(6w)   = 2 593,2 mm³

σ_verrou     = 39 820/2 593,2 + 700,2/280,0 = 15,4 + 2,5 = 17,9 MPa
σ_transition = (section 40 brute, non percée)              =  8,2 MPa
```

Le module net traite le trou comme une **fente sur la fibre neutre**. La formule
`t(w − d0)²/6`, qui vaudrait pour un trou en fibre extrême, donnerait 249 MPa au
lieu de 125 pour 325 N·m — un facteur 2 sur le dimensionnement de la barre.
Contrôle : 325 N·m sur cette section donnent **125,33 MPa**.

---

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
| `bar_moment_max_nm` | moment de flexion **maximal**, au premier trou de la paire |
| `bar_moment_max_at_mm` | son abscisse depuis l'extrémité couronne |
| `bar_moment_at_pair_nm` | moment réduit au barycentre de la paire, celui qu'elle reprend en couple |
| `rear_bar` | cotation de la barre de cette jonction |

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

### 9.3 La flexion de barre est un mode dimensionnant

Ce n'est plus une limite mais un résultat : sur le jeu de grappes
représentatives, le chemin couronne ne dépasse jamais **0,41** de taux de
travail, alors que le verrou atteint **1,17** et la flexion de barre **1,03**.
Le mode que l'ancien modèle ne pouvait pas voir est parmi ceux qui gouvernent.

Voir `dump_joint_load_table` pour régénérer le tableau complet.

### 9.4 Concordance barre / perçage

Les trois trous de la barre ne sont **pas alignés** : le verrou est déporté de
5,3 à 6,7 mm de l'axe couronne-ancrage selon la couronne. Il est donc comparé
sur son abscisse le long de cet axe, et il subsiste un écart de +0,12 mm
(couronne extérieure) / −0,24 mm (intérieure) entre la cotation déclarée et ce
que la jonction impose — une seule position de verrou ne peut pas satisfaire les
deux couronnes exactement. Remonté en avertissement par `check_rear_bar`.

La barre porte **deux** trous de couronne, distants de 4,26 mm : l'entraxe
couronne-ancrage vaut 329,01 mm sur la couronne extérieure et 324,76 sur
l'intérieure. Un trou unique ne pourrait pas desservir les deux.

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

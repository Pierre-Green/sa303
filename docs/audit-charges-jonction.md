# Calcul des charges de jonction SA303 — dossier d'audit

Objet : soumettre à relecture externe le calcul des efforts transmis à chaque
jonction entre deux caissons, et la convention dans laquelle leur direction est
rendue.

Périmètre : **statique de la jonction uniquement**. Sont hors périmètre la
vérification de la goupille et des tôles (`checks::sandwich`, résumée au §8), la
sélection des pires cas (`worst_cases_selector`), et l'acoustique (`wst`).

| | |
|---|---|
| Code audité | `crates/sa303-core/src/cluster/joint.rs`, `src/cluster/kinematics.rs`, `src/speaker/geometry.rs` |
| Fonction d'entrée | `compute_joint(&JointInput) -> JointResult` |
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
angulaire constant : leur entraxe ne dépend donc pas du splay (une seule
longueur de barre dessert tous les crans d'une même couronne). Vérifié par
`the_bar_length_does_not_depend_on_the_splay`.

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
goupille de couronne. Il est de l'ordre de 630 mm et ne s'annule pour aucun
splay de la grille ; aucune garde n'est posée sur ce dénominateur. **À
confirmer par l'audit** : une géométrie custom pourrait théoriquement l'annuler.

---

## 7. Exemple numérique vérifiable à la main

Grappe de 3 caissons SA303 identiques, suspendue, splays `[5°, 10°]`,
`φ_initial = 0`, `g = 9,80665`, `k_dyn = 1,3`, masse 83,695 kg, CG local
`(10,84 ; 11,91)`. Jonction n°1 (`joint_index = 0`), corps libre = caissons 2 et 3.

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

Torseur extérieur et résolution :

```
W      = 2 133,9957 N          cm = (74,0276 ; −750,2904)
M_ext/bo = 446 862,86 N·mm
u      = (0,043619 ; −0,999048)     ← inclinaison 2,5° = splay/2 ✓
bras   = 629,4532 mm
λ      = −709,9223 N

F_bielle  = (−30,9664 ;   709,2466)   |709,92 N|
F_couronne = ( 30,9664 ; 1 424,7491)   |1 425,09 N|
M_barre/an = −85,70 N·m
```

Contrôles immédiats :

- `F_bielle + F_couronne = (0 ; 2 133,9957) = −R_ext` → équilibre en force.
- La direction de `F_bielle` fait exactement **2,5°** avec la verticale, soit
  `splay/2`, comme l'impose l'élément à deux forces. C'est le contrôle croisé le
  plus rapide sur toute la chaîne.

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
```

> **Point d'attention n°3.** Le facteur `sg = −0,5` fait deux choses à la fois.
> Le **signe** retourne l'effort : on rend la réaction *subie par la
> quincaillerie*, pas l'action sur le corps libre. Le **0,5** partage entre les
> deux flancs (l'assemblage est symétrique, donc chaque flanc en prend la
> moitié). Les valeurs rendues sont donc **par flanc**. C'est cette valeur-là
> qui entre dans la vérification de goupille.

| Champ rendu | Contenu |
|---|---|
| `f_pivot`, `f_pivot_n`, `f_pivot_angle_deg` | effort de **bielle**, par flanc |
| `f_orientation`, `f_orientation_n`, `f_orientation_angle_deg` | effort à la **goupille de couronne**, par flanc |
| `bar_moment_nm` | moment déversé par la barre dans le caisson du bas, réduit à l'ancrage |
| `bielle_lever_mm` | bras qui a servi à la résolution |
| `lever_mm` | bras géométrique couronne–ancrage — **recoupement de perçage seulement**, plus utilisé par la statique |
| `residual_n` | contrôle d'équilibre, doit être nul |
| `traction` | signe de la composante axiale de `f_ori` sur l'axe couronne–ancrage |
| `hinge_reversed` | `f_pivot · gravité < 0` : la charnière travaille à l'envers |

Contrôle d'équilibre embarqué, rejoué sur tout le jeu de grappes représentatives
(`invariant_equilibrium_residual_is_negligible`, seuil `1e−6` relatif) :

```rust
let ext_local = (rext * input.share_per_flank).rotate_transpose(rt_phi);
let residual = (f_orientation + f_pivot - ext_local).norm();
```

Vérification aval, pour situer (hors périmètre d'audit) : `checks::sandwich`
prend la norme de chaque effort et retient le pire des trois modes — cisaillement
double de la goupille, matage de la barre, matage des flancs — avec `Rm/4` et une
section nette de 86 %.

---

## 9. Limites connues, à trancher par l'audit

1. **Répartition ancrage / verrou non calculée.** Deux goupilles dans un même
   corps rigide, c'est 6 inconnues pour 3 équations : hyperstatique. Le code rend
   la **résultante** (force + `bar_moment_nm` réduit à l'ancrage) et s'arrête là.
   Il faut une hypothèse de groupe de goupilles pour descendre à l'effort par
   goupille — l'hypothèse usuelle (part directe égale + couple repris sur
   l'entraxe ancrage–verrou, ≈ 23,7 mm) n'a **pas** été retenue faute de
   validation. C'est la principale lacune vis-à-vis d'une note de calcul
   complète.

2. **Jonction bumper ↔ premier caisson.** `solver::compute_bumper_loads` suit
   encore le schéma antérieur (bras à deux forces + pivot au `ht`). C'est une
   liaison différente, non décrite par la spécification de la liaison à bielle,
   donc laissée telle quelle. **À confirmer** qu'elle est bien hors périmètre.

3. **Pas de garde sur `bielle_lever`** (§6).

4. **Valeurs de référence de `golden.rs` recalculées, non recoupées à la main.**
   Elles sortent du modèle lui-même. Ce qui est vérifié indépendamment, c'est la
   cinématique (contre le perçage relevé) et l'équilibre. Un recoupement manuel
   du §7 par l'auditeur comblerait ce trou.

5. **Effets négligés, assumés** : élasticité (tout est rigide), jeu dans les
   trous, frottement, effets hors plan. Le facteur dynamique `k_dyn = 1,3` et le
   coefficient de sécurité 4:1 sont des réglages utilisateur, pas des constantes
   du calcul.

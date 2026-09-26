# Validation indépendante du modèle mécanique du solveur de grappes SA303

Date : 22 septembre 2026. Périmètre : chaîne « grappe → efforts aux goupilles » de `crates/sa303-core` (géométrie, liaisons, répartition, bumper, tirette, conventions). Hors périmètre : matage local, contraintes dans les tôles, FEA.

Dossier : `docs/audit/validation-2026-09/`. Tous les fichiers cités ci-dessous y sont.

---

## 0. Synthèse

**Le modèle est physiquement juste et le code l'implémente correctement sur tout le domaine testé, à une exception près qui concerne une hypothèse et non un calcul.** Un solveur multi-corps indépendant (Python, matrice complète, rang contrôlé) reproduit tous les champs d'effort et de géométrie des exports à mieux que 2·10⁻⁹ en relatif sur 51 grappes (et les 2 exports réels), soit 659 jonctions.

Ce qui doit changer, par ordre de gravité :

| # | Nature | Sens de l'erreur | Impact |
|---|---|---|---|
| C1 | **Hypothèse à discuter, tranchée ici :** la répartition 50/50 de la composante **axiale** entre ancrage et verrou n'est pas justifiée avec un jeu de 0,02 mm et des perçages découpés laser. Un jeu de 20 µm suffit pour qu'un seul pion porte tout l'axial. | Non conservatrice pour les goupilles et le matage de la paire (jusqu'à +68 % sur la paire du bumper de la grappe « 14u 12m »). La flexion de barre, elle, n'est pas touchée (le transverse est isostatique). | Sur la grappe « 14u 12m », le chemin gouvernant passerait des barres (taux 0,738) aux pions de la paire (0,854) : SF 4,7 au lieu de 5,4. |
| C2 | **Bug de diagnostic :** `momentResidualNmm` vaut exactement \|(CG_corps − Q_tirette) × T\| dès qu'une tirette est active (14,5 kN·m sur J0 de « 14u 10m »). Il n'atteste rien. | Aucun effet sur les efforts. | Le champ est faux dans tous les exports avec tirette (234 jonctions du corpus). |
| C3 | **Bug de définition :** `hingeReversed` teste le signe de f_pivot·gravité, pas le signe de l'effort de bielle. Diverge de « bielle en compression » dès que le caisson porteur dépasse ~90° d'inclinaison (cas extrêmes seulement). | Sans effet sur les efforts. | Drapeau faux sur des grappes irréalistes (8 × 20°). |
| C4 | **Convention mal documentée :** le commentaire du code et la suggestion par défaut disent « 180° = vers l'arrière ». En convention §2, 180° est **vers le haut** ; l'arrière est 270°. Les deux exports réels ont une tirette à 180°, c'est-à-dire un **second point de levage vertical** au pied de la grappe (pratique Meyer Sound « vertical pull-back »), pas une tirette au sol. | Physiquement valable, mais ce n'est pas la tirette qu'un rigger comprend. | Interprétation des exports ; `supportForceN` = W·k − T (la manille est *déchargée* de 5,5 kN). |
| C5 | Signe de `fOrientationGlobal`/`fPivotGlobal` inversé entre vol et stack (« ce que subit le flanc » en vol, « ce que subit la barre » en stack). Les efforts de la paire (`fAnchor…`) sont exprimés dans le repère du caisson du **haut** alors que les trous sont dans celui du bas. | Sans effet sur les normes. | Flèches du viewer et angles `…AngleDeg` de la paire. |
| C6 | **Stack incomplet :** le bumper y est modélisé comme deux pions en répartition élastique aux cotes de dessin (`heightFromBottom = 60`), pas comme bielle + barre 10°/20° + cale en L. L'assiette du premier caisson est libre au lieu d'être imposée par la barre montée. | Indéterminé (le modèle ne représente pas la liaison réelle). | Pions du bumper en stack, et stabilité au sol non vérifiée (basculement). |
| C7 | Tension de tirette non bornée près des limites de plage : à 141° la grappe « 14u 25° » demande 287 kN et sort avec SF 0,11 sans refus ; à la limite exacte, `tie_tension` renvoie **0** au lieu de refuser. | Non conservatrice dans le cas dégénéré (tension 0 = grappe supposée tenue sans effort). | Cas limite improbable mais silencieux. |
| C8 | Masses ignorées : barre arrière (2,32 kg, 2 par jonction, soit 5,5 % d'un caisson), bumper, barre de déport. `momentResidualNmm` n'aurait pas pu le voir. | Non conservatrice de quelques % sur le levage et sur les jonctions basses. | À confirmer (question Q3). |

Coefficients (§1.4) : le **4:1** est bien le coefficient d'utilisation « éléments métalliques d'un accessoire de levage » de la directive 2006/42/CE, annexe I §4.1.2.5 (d), repris mot pour mot par le règlement (UE) 2023/1230, annexe III §4.1.2.5 (d) ; c'est la pratique déclarée de L-Acoustics. Le **10:1** est le doublement DGUV (« Eigensicherheit ») du coefficient 5 des câbles et manilles quand des personnes sont sous la charge (DGUV Information 215-313, tableau 1). Le **1,1** est le coefficient d'**essai** dynamique de la directive, pas un coefficient de calcul. Le **1,3** n'a été retrouvé dans aucune source lue ; la DGUV 215-313 §1.4 donne « au moins 20 % en plus » comme valeur indicative. Détail en §4.

---

## 1. Méthode et conventions

### 1.1 Solveur de référence

`sa303_ref.py` reconstruit la géométrie depuis `definitions` (rien n'est lu dans `result`) et pose l'équilibre de **chaque** corps : bumper, N caissons, N bielles (éléments à deux forces, 1 inconnue scalaire), N barres (corps rigides : force inconnue à la couronne + torseur inconnu vers le caisson du bas), manille (2 inconnues), tirette (1 inconnue le long d'une direction connue). Assemblage `A·x = b`, résolution par moindres carrés, contrôle du rang et des résidus par corps.

Comptage (vol, N caissons, N liaisons dont celle du bumper) : inconnues 6N + 2 (+1 avec tirette), équations 3(2N + 1) = 6N + 3.

- Avec tirette : système **carré, de plein rang** (87 × 87 sur 14 caissons). La liaison est isostatique, la solution unique.
- Sans tirette (pendaison libre ou accroche dans la portée de barre) : une équation de plus que d'inconnues. C'est la condition « CG sous le pickup » que le code résout **en amont** par géométrie ; la référence vérifie qu'elle est satisfaite à la précision machine (résidu < 10⁻⁸ N·mm). C'est la fermeture du point 7.

Ce qui reste hyperstatique n'est pas dans ce système : c'est la répartition du torseur de barre sur les **deux** pions de la paire (4 inconnues pour 3 équations). Le code y applique une règle ; la référence applique la même règle pour comparer, puis calcule l'enveloppe (point 3).

### 1.2 Comparaison

`compare.py` confronte, pour chaque grappe de `corpus-export.json` (51 grappes calculées par le solveur Rust via le harnais `tests/audit_corpus.rs`) et des deux exports réels, chaque champ d'effort, de géométrie et de tirette. Résultat complet : `comparison.md`. Écart relatif maximal sur tous les champs d'effort : **2·10⁻⁹** (arrondi f64). Écart sur les points : < 10⁻¹² mm. Un seul champ diverge au-delà : `barMomentMaxNm`, 5·10⁻⁵ relatif, parce que le code le calcule sur les abscisses **déclarées** des trous (445,014 / 116) alors que la référence utilise les points **construits** (écart 1,7 µm latéral × effort axial). Sans conséquence.

### 1.3 Conventions, telles que vérifiées

| Élément | Convention constatée |
|---|---|
| Repère caisson | origine au centre, x vers l'**arrière**, y vers le **haut** |
| Repère global | celui du caisson 0 non tourné (`o = 0`) |
| φ | rotation directe (trigonométrique). φ > 0 ⇒ l'arrière monte ⇒ **nez vers le bas**. `phiFreeHang = −10,34°` sur les J14 = nez en l'air |
| Splay | φ(bas) = φ(haut) + splay ; splay > 0 ouvre vers le bas |
| Angles `…AngleDeg` | 0° vers le bas, horaire, **90° vers l'avant, 180° vers le haut, 270° vers l'arrière** (`dir_from_angle`) |
| `fOrientation`, `fPivot` (vol) | force que la barre (resp. la bielle) **exerce sur le flanc du caisson du haut**, par flanc, repère du caisson du haut |
| `fOrientation`, `fPivot` (stack) | force que le caisson du haut exerce **sur la barre** (resp. la bielle) : signe opposé à la convention du vol (C5) |
| `fAnchor`, `fLatch` | force que la barre exerce sur les pions du caisson du **bas**, par flanc, mais exprimée dans le repère du caisson du **haut** (`loadedFlank`) en vol |
| `barAxialN` | > 0 = barre **comprimée** (la couronne pousse vers la paire) |
| `barMomentAtPairNm` | moment du torseur de couronne réduit au **milieu** de la paire (N·m, par flanc). Ce n'est **pas** le moment fléchissant interne en ce point ; il sert au couple ±P des pions |
| `barMomentMaxNm` | moment fléchissant à l'**ancrage** = \|(P_anchor − P_couronne) × F\| (max du diagramme) |
| `hingeReversed` | f_pivot·gravité_locale < 0 (voir C3) |
| `supportForceN` | \|W·k_dyn − T_tirette\| : ce que tire la manille, **non** doublé par flanc |
| `pinPairMomentNm` | moment que la structure du bumper transfère entre ses deux pions, réduit à leur milieu |
| `axisMapping` toolY = −Y | l'outil CAO a y vers le bas ; le solveur y vers le haut. Sans effet sur les efforts |

Le repère local des efforts de paire (repère du haut pour des trous du bas) est l'ambiguïté la plus gênante pour la suite (arrachement de bord dans le flanc du bas : la direction compte). À corriger avec C5.

---

## 2. Les douze points

Format : verdict ; justification ; source ; sens de l'erreur ; impact.

### 2.1 Isostaticité

**Validé.** Voir §1.1 : le système complet est carré et de plein rang avec tirette, surdéterminé d'une équation sans tirette (équation satisfaite). Ce que le solveur suppose pour lever la seule indétermination restante (axial de la paire) est traité au 2.3. Aucune autre hypothèse cachée : la barre n'a pas de raideur, la bielle non plus, tout est rigide (statique des corps rigides, Hibbeler *Engineering Mechanics: Statics*, ch. 5 « Equilibrium of a rigid body » et ch. 6 « Frames and machines » ; référence citée de mémoire, ouvrage non consulté pendant cet audit).

Conditions de validité (petits déplacements, liaisons parfaites) : bielle de 38 mm, jeux de 0,08 mm (goupille Ø12 dans Ø12,08 d'après `settings.pin.diameter = 12` ; 0,02 mm avec la Ø12,06 du brief), flèche de barre de l'ordre de 0,1 mm (calculée plus bas). La géométrie ne bouge pas : hypothèse valide.

### 2.2 Bielle = élément à deux forces

**Validé.** \|u × f_pivot\| / \|f_pivot\| < 3·10⁻¹⁵ sur les 659 jonctions, traction et compression comprises. La direction u va de `hb`(haut) à `pv(splay)` ; `pv` est bien sur l'arc de rayon 38,125 mm tourné de splay/2 ; le `ht` du bas tombe dessus à < 10⁻¹² mm (« fermeture ht(bas)=pv(haut) » dans `comparison.md`). Recul de face avant à 20° : 0,86 mm (< 0,9 mm annoncé).

Compression : sur « 14u 10m tilt 25 tie 180 », la bielle de J0 est comprimée à 844 N/flanc (1,7 kN total) ; sur « 14u 12m », J0 à J5 sont comprimées. Un élément bi-goupillé travaille en compression sans changer de modèle (pas de flambement à 38 mm). Ce qui est faux, c'est le drapeau (C3) : `hingeReversed` vaut `false` sur J6 de « iso8 20° partout » alors que λ > 0. Définir `hinge_reversed = f_pivot_global · u > 0` (u de haut vers bas, f_pivot = −uλ·share).

### 2.3 Barre encastrée par deux pions : répartition axiale et transverse

**Transverse : validé, ce n'est pas une hypothèse.** Deux pions sur un même axe, à 100 mm : ΣF_transverse et ΣM sont deux équations pour deux inconnues. La solution R_a = F_t/2 + M_G/d, R_l = F_t/2 − M_G/d est **exacte**, indépendante des raideurs. La référence le confirme avec des ressorts de raideurs 594 kN/mm, 5 942 kN/mm, 1 kN/mm, ou rapports 10:1 entre axial et transverse : mêmes valeurs à 10⁻⁶ (`analysis.py`, §2). La flexion de barre (`barMomentMaxNm`, `checks::bar`) ne dépend donc que de la statique. **Elle est juste.**

**Axial : hypothèse à discuter, tranchée : non justifiée.** Les deux pions sont en parallèle sur l'axe ; c'est la seule redondance. Le partage réel dépend du jeu et des écarts de position :

- raideur d'une goupille Ø12 en cisaillement double (barre 10, flancs 2 × 4 mm), formule de Huth (a = 2/3, b = 3, assemblage boulonné métallique) : **k ≈ 594 kN/mm** ;
- déplacement élastique pour tout l'axial de J0 (« 14u 12m », 8,4 kN/flanc) sur un seul pion : **14 µm** ;
- jeu diamétral 0,02 à 0,08 mm, plus tolérance de position des trous (deux flancs + barre, découpe laser : typiquement ±0,1 mm par pièce, classe ISO 2768-m ; à confirmer avec le sous-traitant, Q5).

Avec un écart d'entraxe de 20 µm entre la barre et les flancs, le second pion n'entre jamais en contact avant que le premier ait pris **tout** l'axial ; il faudrait > 60 kN pour fermer 0,1 mm élastiquement. Seul le matage plastique du premier trou (à ~1,5 R_e × d × t ≈ 25 kN par flanc) redistribue. Conclusion : **utiliser l'enveloppe « tout l'axial sur un seul pion » pour les vérifications de goupille et de matage de la paire**, et garder le 50/50 pour rien d'autre.

Enveloppe (par flanc) : f_anchor_env = √((F_t/2 + M_G/d)² + F_ax²), f_latch_env = √((F_t/2 − M_G/d)² + F_ax²).

Chiffres sur les deux exports :

| Grappe | Pion de paire le plus chargé, 50/50 | Enveloppe | Taux matage flanc 50/50 → env. | Chemin gouvernant exporté |
|---|---|---|---|---|
| 14u 10m tilt 25 | J12 ancrage 10 291 N | 10 539 N | 0,841 → 0,861 | barre J12, 0,966 (inchangé) |
| 14u 12m tilt 8,5 | J0 ancrage 7 535 N | 10 448 N | 0,616 → 0,854 | barre J0, 0,738 → **pions**, 0,854 |
| 14u 12m, paire du bumper | 5 763 N | 9 675 N | 0,471 → 0,790 | — |

Sur le corpus, le pire ratio enveloppe / 50/50 est 2,0 (grappes droites : effort presque purement axial, faible en valeur absolue). Sens : **non conservateur** en l'état, de 2 % à 70 % selon la part axiale. Le bumper est le plus touché (barre courte, effort presque axial : 12,1 kN axial pour 4,2 kN transverse sur « 14u 10m »).

Source : la directive n'en dit rien ; c'est de la mécanique des assemblages (Huth 1986, ASTM STP 927, *Influence of Fastener Flexibility on the Prediction of Load Transfer and Fatigue Life for Multiple-Row Joints* ; papier non consulté, formule reprise de mémoire, voir §4.6). EN 1993-1-8 §3.13 (axes) n'a pas pu être lu (norme payante).

### 2.4 Bumper modélisé comme une jonction

**Validé.** `compute_bumper_loads` résout exactement le même problème que `compute_joint` : bielle bi-goupillée entre `ht` du caisson 1 et le pion avant du bumper (élément à deux forces), barre du bumper articulée à son trou haut et encastrée par la paire ancrage/verrou du caisson 1. La référence le retrouve à 10⁻⁹ (`bumper orientationForceN`, `pivotForceN`, `fPairAnchorN`, `pairMomentNm`, `pinPairMomentNm`). L'ancien `split_wrench_over_pair` n'est plus appelé en vol.

Remarque : le bumper est sans masse et la barre de déport aussi (C8/Q3).

### 2.5 Fermeture géométrique du bumper

**Corrigé, avec une cote résiduelle à trancher (Q1).** Pion avant = `ht` + (0 ; 71,6) ; pion arrière = ancrage + 224,626·e_axe + 1,822·e_avant. Entraxe obtenu : 656,8308 mm contre 656,829 au plan (702 − 12,567 − 32,604) : écart 1,8 µm. Les deux pions sont à la même hauteur (y = 328,72 et 328,73, repère caisson). La bielle fait bien 71,6 (relevé Fusion : y_ancrage + 224,560 − y_ht = 104,160 + 224,560 − 257,127 = 71,59 ✓). Le point 5 est fermé.

Mais les deux pions sont à **53,7 mm** au-dessus de la face supérieure du caisson (y = 275), alors que `heightFromBottomMm = 60`. Soit le bumper ne pose pas sur la face supérieure (6,3 mm de vide, ou d'appui ailleurs), soit la cote 60 est fausse. Sans effet sur les efforts en vol (le solveur n'utilise plus cette cote), mais :

- la silhouette dessinée (`bumper_outline_top`, tangente à la face sup) est fausse de 6,3 mm, donc `elevation.pickupMm` aussi ;
- en stack, `bumper_pin_points` **utilise** cette cote (C6).

### 2.6 Tirette

**Validé pour le point d'application, la direction, la tension et la plage. Deux bugs de bord (C7) et une convention à documenter (C4).**

- Point : `crown(0)` du dernier caisson, soit le trou de couronne splay 0 percé dans ce caisson (polaire (pv(0), 680, 15°)). `anchor_at(0)` n'apparaît plus nulle part pour la tirette. ✓
- Tension : T = −M_W/((Q − P_k) × u), avec M_W le moment du poids dynamisé autour du pickup. La référence retrouve 5 544,74 N et 2 659,92 N. Le moment global autour du pickup est nul à 4·10⁻⁸ N·mm. ✓
- Plage : demi-cercle ouvert autour de la perpendiculaire à (Q − P_k), côté tension positive. Retrouvée à 10⁻¹³°. ✓ Les angles hors plage sont refusés (« il faudrait pousser »), vérifié à 0°, 90°, 300° selon la grappe.
- Bord de plage : la tension diverge en 1/sin. À 141° (plage [140,3° ; 320,3°]), T = 287 kN et l'export sort **sans refus** avec SF 0,11 (« INVALIDE tirette traverse la grappe » du corpus, qui en fait ne traverse rien). À la borne exacte, \|d\| < 10⁻⁹ ⇒ `tie_tension` renvoie **0** : la grappe est rendue tenue sans tirette alors qu'elle ne l'est pas. Refuser (ou plafonner) quand la tension dépasse, par exemple, le poids de la grappe.
- Convention : `tie_default_angle_deg` cherche 180° « vers l'arrière ». En §2, 180° = **vers le haut** (`dir_from_angle(180) = (0, +1)`). Les deux exports réels ont `tieAngle = 180` : le câble tire **verticalement vers le haut** depuis la couronne du dernier caisson. C'est un second moteur au pied de la grappe, exactement la configuration « pull-back » de Meyer Sound (« Pull-Back should be as close to vertical as possible », voir §4.5), pas une tirette au sol (qui serait vers 300–320° ici et **augmenterait** la charge de la manille au lieu de la réduire de 5,5 kN). Le solveur est juste ; la documentation, le libellé et le défaut doivent dire ce qu'ils font.

### 2.7 Équilibre de l'accroche

**Validé.** `supportForceGlobal` = W·k_dyn − T à 7·10⁻¹² N ; passe par le pickup (moment nul). `pinSpanMm` = 656,83 ; `pinPairMomentNm` recalculé à 10⁻⁹. `bumperBarExceeded` est levé exactement quand \|x_raw\| > 650 (`maxDeportMm` de la barre) ; sinon quand > 130,23 (`maxDirectDeportMm`) c'est `barDeportMm` qui devient non nul. Pickup x = (x_CG + h·sinφ)/cosφ, vérifié (pickup à 6·10⁻¹⁴ mm du centre quand l'assiette imposée égale la pendaison libre).

Réserve : sans masse de bumper ni de barre de déport, la position d'équilibre est celle des caissons seuls (C8).

### 2.8 Trou de couronne selon le splay

**Validé pour la mécanique ; le rattachement de 10,5 est une donnée à confirmer (Q2).** La rangée est **déclarée** (`crown.innerSplays = [1, 3, 5]`), plus déduite d'une parité : 10,5 est donc en rangée extérieure (R = 680, `up680`, sur l'axe), 1/3/5 en intérieure (R = 660, `up660`, latéral 19,406 vers l'avant). La référence reconstruit les 8 trous depuis la cotation de la **barre** (ancrage + 329,014·e_axe pour `up680`, + 324,175·e_axe + 19,406·e_avant pour `up660`) et les compare à la polaire du caisson : écart ≤ 1,7 µm pour les 8 (tolérance de montage 0,05 mm). Un splay hors grille (7°, 10°) est refusé avec la liste des trous percés (message : « Angles percés : 0°, 1°, 2°, 3°, 4°, 5°, 10°, 20° » — le 10,5 s'affiche « 10° », `format!("{s:.0}°")`, à corriger).

Ce que je ne peux pas vérifier : que le plan de perçage met bien 10,5 sur la rangée 680. Ligaments entre trous voisins de couronne : 10,8 mm (entraxe 22,8 pour Ø12,08 entre 4° et 5°, rangées différentes). Hors périmètre ici (matage), mais à regarder : EN 1993-1-8 demande p₁ ≥ 2,2 d₀ = 26,6 mm entre trous chargés simultanément ; ici un seul trou est chargé à la fois, ce qui relève d'un calcul d'arrachement, pas d'espacement.

### 2.9 Cas stack

**Hypothèse à discuter : le modèle ne représente pas la liaison réelle (C6).**

- Jonctions : correctes. Corps libre = caissons au-dessus, gravité seule, même statique qu'en vol retournée. Référence à 10⁻⁹ sur toutes les jonctions des 6 stacks du corpus. Les barres passent en compression de façon cohérente (`barAxialN` > 0, `traction = false`).
- Bumper : le code le pose au sol, parallèle au sol, et goupille le dernier caisson par **deux pions** aux cotes de dessin (`heightFromBottom = 60`, voir 2.5) avec la répartition élastique 50/50 + couple (`compute_stacked_bumper_pins`). Or le matériel réel (brief §1.2) est bielle + barre 10°/20° + paire ancrage/verrou + cale en L. Trois conséquences :
  1. l'assiette du premier caisson (`imposedTilt` en stack) est libre, alors qu'elle ne peut valoir que l'inclinaison d'une barre déclarée (`rearBars[].tiltDeg` : seul 0° existe dans le JSON) ;
  2. la répartition 50/50 entre deux pions **quelconques** est une hypothèse de raideur, pas de la statique ; avec bielle + barre elle serait isostatique comme en vol ;
  3. la cale en L reprenant la compression au pied de barre n'est pas représentée : le solveur envoie tout dans les goupilles (conservateur pour elles, muet sur la cale et le bout de barre).
- Sol : la réaction est bien W·k_dyn appliquée au milieu de la face d'appui. La référence calcule en plus la position de la résultante : sur « stack 3 bas 20 » elle est décalée de 156 mm du milieu (demi-profondeur 351) ; sur « stack 4 cca bas 20 », 114 mm. Pas de basculement sur le corpus, mais **le solveur ne le vérifie pas** (L-Acoustics a un « stability warning » dédié, §4.5).

Ce que ça change : rien sur les jonctions ; les pions du bumper en stack sont des valeurs sans modèle physique validé. Recommandation : refuser un stack dont l'assiette ne correspond à aucune barre déclarée, et réutiliser `compute_bumper_loads` en stack (barre au tilt demandé, poids retourné). Il faut les cotes des barres 10° et 20° (Q4).

### 2.10 Flanc chargé et `sharePerFlank`

**Approximation acceptable, à documenter.** 0,5 couvre une grappe plane, symétrique, à charge statique. Ne couvre pas : un moteur unique excentré (le bumper tourne autour de l'axe longitudinal), une grappe vrillée (les pions d'un flanc reprennent une part du moment de torsion : la répartition n'est plus 50/50 mais dépend des raideurs de flanc), le balancement latéral (charge dynamique hors plan). Aucune de ces situations n'est calculable en 2D ; elles relèvent du k_dyn ou d'un calcul 3D à part. Recommandation : un facteur de flanc réglable (par exemple 0,55–0,6) plutôt qu'une constante, si ces cas sont revendiqués.

### 2.11 k_dyn sur la tirette

**Validé, cohérent.** La tirette est une **réaction** : sa tension est résolue pour tenir une assiette sous un poids déjà multiplié par k_dyn ; elle est donc proportionnelle à k_dyn (vérifié : rapport 1,1/1,3 exact sur tous les efforts). `compute_joint` ne la remultiplie pas. Si la tirette est un moteur (C4), il voit lui aussi la charge dynamisée, ce qui est le sens attendu. Note : le coefficient d'essai dynamique 1,1 de la directive n'est pas un coefficient de calcul (§4.1).

### 2.12 Comportements à confirmer

- **Enceinte 1 moins chargée que l'enceinte 2 : confirmé, et l'explication est juste mais incomplète.** Sur « 14u 10m » : paire du bumper 7 531 N contre 8 806 N à J0. Le moment de barre est −585 N·m au bumper contre −752 N·m à J0. Décomposition (efforts totaux, deux flancs) : bumper F_ax = 12 098 N, F_t = 4 181 N, bras 274,6 mm, latéral 1,8 mm ⇒ M = −1,17 kN·m ; J0 F_ax = 9 040 N, F_t = 3 968 N, bras 379,0 mm, latéral 0 ⇒ M = −1,50 kN·m. Le rapport ≈ 0,78, pas 0,5 : la barre du bumper porte **plus** d'effort (elle tient 14 caissons contre 13) mais avec un bras plus court (274,6 contre 379,0 = 445,014 − 116 + 50). Le « moment divisé par 2 » n'est pas retrouvé ; on retrouve « divisé par 1,3 » sur cette grappe, 1,85 sur « 14u 12m » (−309 contre −572 N·m, où J0 est à splay 1° donc up660 : bras 324,2 mm + 19,4 mm de latéral, et effort presque axial). Avec l'enveloppe axiale (2.3) la conclusion se renverse sur « 14u 12m » : bumper 9 675 N contre 10 448 N à J0, quasi égalité.
- **Marches aux changements de rangée : confirmé.** Sur « 14u 10m », M_pair passe de −602 (J2, ext) à −500 (J3, int) puis de −95 (J6, int) à −24 (J7, ext) : le latéral de 19,4 mm de `up660` change le bras d'un coup, dans le sens du signe de l'axial. Les efforts de pion suivent (6 831 → 5 684 N). C'est de la géométrie, pas un artefact.
- **Courbe en V avec tirette : confirmé, et la cause est bien le moment de la tirette.** Sans tirette (même grappe en pendaison libre) : ancrage 4 747 → 1 468 N, décroissant hors marches. Avec tirette à 180° : 8 806 → 2 309 (J7) → 10 291 N (J12), avec changement de signe de M_pair entre J7 et J8. Le moment de la tirette autour de la couronne de J12 vaut +2 937 N·m contre −338 N·m pour le poids du dernier caisson : la dernière jonction ne porte plus un caisson, elle **retient** une tirette de 5,5 kN. C'est le prix d'un pull-back au pied de la grappe, et c'est réel.

---

## 3. Contrôles systématiques et cas limites

Tout est dans `comparison.md` (tableau des écarts max, 60 champs) et `corpus-reference.json` (résultats de référence par grappe). Résumé :

| Contrôle | Résultat |
|---|---|
| Équilibre force/moment de chaque corps (caissons, bielles, barres, bumper) | résidu < 2·10⁻⁵ N et < 2·10⁻⁵ N·mm, sur le pire cas à 287 kN ; < 10⁻⁹ ailleurs |
| Fermeture ht(bas) = pv(haut) | < 10⁻¹² mm |
| Couronne polaire vs couronne par cotation de barre | ≤ 1,7 µm (8 splays × 2 rangées) |
| Colinéarité bielle | < 3·10⁻¹⁵ |
| Manille = W·k − T ; moment nul au pickup | 7·10⁻¹² N ; 4·10⁻⁸ N·mm |
| Local = R(−φ)·global (vol) | 10⁻¹² ; **signe inversé en stack** (C5) |
| 1 caisson, 2 caissons à la main | formules fermées dans `test_sa303_ref.py` (λ = −M_ext/bras ; paire = F/2 ± M_G/d) : identiques à 10⁻⁹ |
| Élévation 0 → 8 m | écart 0 exactement |
| Assiette imposée = pendaison libre | pickup à 6·10⁻¹⁴ mm, tirette 0, efforts égaux à 2·10⁻⁸ N |
| Masses × 2 | efforts × 2,000000000 |
| Superposition (12 cas à une masse) | écart 5·10⁻⁷ N |
| Splay 0 partout / 20 partout (1 à 14 caissons) | tous résolus, identiques |
| k_dyn 1,1 / 1,3 | rapport exact |
| Ressorts (Huth) → rigide | identique à jeu nul ; bascule à 100/0 axial dès 20 µm de jeu (§2.3) |
| Refus | splay 7°, 10° ; tirette 0°, 90°, 300° (selon grappe) ; 2 caissons pour 3 jonctions : tous refusés avec raison |

Cas à la main, 2 caissons droits en pendaison libre (φ = −0,888°, W = 1 067,0 N par caisson dynamisé) : bras de bielle autour de la couronne 656,8 mm ; M_ext = (CG₂ − bo) × W = +335,1 kN·mm ⇒ λ = −510,2 N (traction) ; f_ori = W − uλ ⇒ 557,0 N total, 278,5 N par flanc ; paire : d = 100,0 mm, M_G = 2,58 N·m ⇒ 140,0 / 139,7 N par flanc. Export : 278,48 / 255,08 / 139,99 / 139,68. Les nombres intermédiaires sont imprimés par `analysis.py §3`.

---

## 4. Sources

Règle appliquée : je cite ce que j'ai lu (texte extrait du PDF officiel, page indiquée). Ce que je n'ai pas pu lire est dit tel quel.

### 4.1 Directive 2006/42/CE (JO L 157 du 9.6.2006), lue sur EUR-Lex (PDF)

- Art. 2 (d), « accessoire de levage » : « a component or equipment not attached to the lifting machinery, allowing the load to be held, which is placed between the machinery and the load or on the load itself, or which is intended to constitute an integral part of the load and which is independently placed on the market ».
- Annexe I §4.1.1 (c) : le **coefficient d'utilisation** est « the arithmetic ratio between the load guaranteed by the manufacturer […] up to which a component is able to hold it and the maximum working load marked on the component » ; (d) le **coefficient d'essai** est le rapport charge d'essai / charge maximale d'utilisation.
- Annexe I §4.1.2.3 (p. L 157/57) : coefficient d'**essai statique** « (a) manually-operated machinery and lifting accessories: 1,5; (b) other machinery: 1,25 » ; coefficient d'**essai dynamique** « as a general rule, equal to 1,1 ».
- Annexe I §4.1.2.4 : câbles complets 5, chaînes de levage 4.
- Annexe I §4.1.2.5 (p. L 157/57–58) : (a) câbles 5 ; (b) chaînes 4 ; (c) textile 7 ; **(d) « all metallic components making up, or used with, a sling must have a working coefficient chosen in such a way as to guarantee an adequate level of safety; this coefficient is, as a general rule, equal to 4 »**.
- Annexe I §6.1.1 : ces coefficients « are inadequate for machinery intended for the lifting of persons and must, as a general rule, be doubled ».

Lecture : le 4:1 « sur Rm » n'est pas écrit comme tel. La directive parle de charge garantie / charge maximale d'utilisation ; c'est la pratique (L-Acoustics, §4.5) qui le traduit en « 4:1 against the rupture ». Et le 1,1 est un coefficient d'**essai** dynamique de la machine de levage, pas un facteur d'amplification de calcul.

### 4.2 Règlement (UE) 2023/1230 (JO L 165 du 29.6.2023), lu sur EUR-Lex (PDF)

- Art. 3 (5) : même définition de l'accessoire de levage (p. L 165/13).
- Annexe III §4.1.2.3, §4.1.2.4, §4.1.2.5 (p. L 165/75–76) : texte identique à la directive, mêmes valeurs (1,5 / 1,25 ; 1,1 ; 5 / 4 / 7 / 4).
- Art. 51 (2) : « Directive 2006/42/EC is repealed with effect from 14 January 2027 » ; Art. 54 : « It shall apply from 14 January 2027 ».

Le passage au règlement ne change donc aucun coefficient.

### 4.3 DGUV Vorschrift 17 (ex BGV C1), « Veranstaltungs- und Produktionsstätten für szenische Darstellung », du 1er avril 1998, PDF publikationen.dguv.de

- § 9 « Tragmittel und Anschlagmittel » (p. 10) : « Tragmittel und Anschlagmittel müssen entsprechend der besonderen Gefährdung beim Betrieb und den beim Betrieb auftretenden Belastungen beschaffen und ausreichend bemessen sein. » Aucun chiffre dans la règle elle-même (le PDF lu ne contient pas les Durchführungsanweisungen).

### 4.4 DGUV Information 215-313 (ex BGI 810-3), « Lasten über Personen », Ausgabe März 2017, PDF unfallkasse-nrw.de

- §1.2 (p. 7) : « Eigensicherheit wird durch Verdoppelung der Betriebskoeffizienten erreicht. »
- §1.4 (p. 7) : « Ist die Tragfähigkeit (z. B. WLL) angegeben, darf dieses Arbeitsmittel maximal mit der Hälfte dieses Wertes belastet werden. » ; « Betriebskoeffizienten für Anschlagmittel sind in der Richtlinie 2006/42/EG […] Anhang 1 Punkt 4.1.2.5 festgelegt. Der Begriff Betriebskoeffizient ersetzt die alten Begriffe Sicherheitsbeiwert und Sicherheitsfaktor. »
- §1.4 (p. 8), dynamique : « Für bewegte Lasten sind […] die aus der Dynamik […] herrührenden Kräfte mit zu berücksichtigen. **Als Richtwert für diese dynamischen Kräfte hat sich die Berücksichtigung von zusätzlich mindestens 20 Prozent bewährt.** »
- Tableau 1 (p. 9), « Mindestens erforderliche Betriebskoeffizienten von Anschlagmitteln » : sans personnes / avec personnes sous la charge : Drahtseile 5 / **10** ; Rundschlingen textiles 7 / 14 ; Anschlagketten 4 / 8 ; Schäkel DIN EN 13889 5 / **10**.
- §2.2 (p. 17), Lastaufnahmemittel : « spezielle Konstruktionen zur Lastaufnahme (z. B. Lautsprecher-, Traversen- oder Beameraufhängungen) » ; « Liegt ein Nachweis für den speziellen Anwendungsfall „Lasten über Personen“ vor, dürfen sie nach Herstellerangabe belastet werden. Liegt kein Nachweis vor, dürfen sie nur mit der Hälfte der Herstellerangabe belastet werden. »
- §2.4 (p. 22) : « Lasten gehören nicht zu den Lastaufnahmemitteln, Anschlagmitteln oder Hebezeugen und unterliegen daher nicht grundsätzlich der Maschinenrichtlinie und deren Betriebskoeffizienten […]. Zur Berechnung […] Eurocode 3/DIN EN 1993-1-1 ».

Lecture pour la SA303 : la quincaillerie de flanc + bielles + barres + bumper est un **Lastaufnahmemittel** (§2.2) : coefficient de la directive (4 pour les éléments métalliques), et pour des personnes dessous, soit un « Nachweis » spécifique, soit la moitié de la charge nominale. Le **10:1** est bien réservé aux **Anschlagmittel** (câbles, manilles) au-dessus de personnes (tableau 1). Le **1,3** n'apparaît pas ; la valeur DGUV indicative est **+20 %**. Le 1,3 est donc conservateur par rapport à la DGUV et sans source retrouvée : à documenter comme choix de conception (Q6).

### 4.5 Fabricants

- **L-Acoustics, K2 Rigging Manual K2_RM_EN_0.5, §2.1 p. 7** (PDF) : « The K2 rigging system complies with 2006/42/EC: Machinery Directive. It has been designed following the guidelines of BGV-C1. 2006/42/EC: Machinery Directive specifies a safety factor of 4:1 against the rupture. The limits specified in the tables below correspond to deployments with a safety factor of 4:1 or higher. Refer to SOUNDVISION for the safety factor of a specific deployment. » §2.2 : « The overall safety factor of a specific mechanical configuration always corresponds to the lowest safety factor among all the linking points. » ; stack : « a distinct stability warning is implemented in SOUNDVISION. It indicates a tipping hazard when the array is not secured ». Le manuel K2 ne décrit pas de tirette ; « pull back » n'y désigne qu'un geste de montage.
- **d&b audiotechnik, J-Series Rigging manual 1.4 EN, §1.2 p. 4** (PDF) : « The Z5300 J Flying frame is designed to suspend a total system weight of 1.5 t (3300 lb) WLL (Working Load Limit) according to BGV C1. » ; §5 (vent) : « Suspension and securing points of the array should be designed to accommodate double the static load ». V-Series 1.12 EN : ArrayCalc obligatoire ; « unpredictable dynamic forces as well as swinging of the array must be taken into account ». **Aucun coefficient numérique de sécurité** dans les deux manuels lus.
- **Meyer Sound, Pull Back Rigging Operating Instructions, PN 05.083.008.01 Rev A (2018), p. 3** (PDF) : « The Vertical Pull-Back Load Status is the pass/fail criteria. The −10º/+10º status indicators provide the user of the tolerance that exists to each side of the vertical. For all purposes Pull-Back should be as close to vertical as possible ». C'est la configuration des deux exports (tirette à 180°). Les coefficients 7:1 (LEO-M) et 5:1 (LYON) trouvés dans un extrait de recherche n'ont **pas** été vérifiés sur un document primaire.

### 4.6 Mécanique

- Huth, H., « Influence of Fastener Flexibility on the Prediction of Load Transfer and Fatigue Life for Multiple-Row Joints », ASTM STP 927, 1986. **Non lu** (accès ASTM refusé, 403). La formule C = ((t₁+t₂)/(2d))^a · (b/n) · (1/(t₁E₁) + 1/(n t₂E₂) + 1/(2t₁E_f) + 1/(2n t₂E_f)), a = 2/3, b = 3 pour assemblage boulonné métallique, est reprise de mémoire ; elle est notoirement du bon ordre de grandeur (±30 %), ce qui ne change pas la conclusion du 2.3 (le jeu domine d'un facteur > 4).
- EN 1993-1-8 (axes, §3.13 ; distances au bord, tableau 3.3) : **non lue**, norme payante. Le 1,2 d₀ du code (`MIN_EDGE_DISTANCE_IN_D0`) correspond au e₁,min du tableau 3.3 de mémoire.
- Hibbeler, *Engineering Mechanics: Statics*, éléments à deux forces (§5.4 dans les éditions récentes), équilibre des corps rigides, superposition en statique linéaire : **de mémoire**, non consulté.
- ISO 2768-1 (classe m) : non lue ; les ±0,1 à ±0,2 mm de position en découpe laser viennent de guides de sous-traitants (recherche web), pas de la norme. À confirmer par le fournisseur (Q5).
- EN 17206:2020 (machinerie de scène) : **non lue**, payante. Je ne lui attribue rien.

---

## 5. Corrections proposées, par gravité

1. **C1 – Enveloppe axiale de la paire.** Ajouter `f_anchor_env_n` / `f_latch_env_n` = √((F_t/2 ± M_G/d)² + F_ax²) par flanc dans `JointResult` et `BumperView` ; les utiliser dans `JointChecks` / `BumperChecks` pour `utilization_anchor` / `utilization_latch` (matage, cisaillement). Garder 50/50 pour la seule information « part directe ». Formule dans `compare.py`. Effet : SF de « 14u 12m » 5,42 → 4,68 ; bumper de la même grappe 0,47 → 0,79.
2. **C2 – `moment_residual_nmm`** (`joint.rs:402`) : remplacer `(cm − o) × rext` par `(cm − o) × W + (q − o) × T`. Test proposé `moment_residual_is_zero_even_with_a_tie`.
3. **C6 – Stack.** Refuser un `imposed_tilt` en stack qui ne correspond à aucune `rear_bars[].tilt_deg` ; résoudre la liaison bumper–caisson par `compute_bumper_loads` avec la barre au tilt demandé (poids retourné, corps libre = toute la pile) ; ajouter un contrôle de basculement (résultante du sol dans l'empreinte). Prérequis : cotes des barres 10° et 20° (Q4) et rôle de la cale en L (Q4).
4. **C7 – Tirette dégénérée.** Dans `tie_tension`, une direction parallèle à (Q − P_k) doit être une erreur, pas 0 ; refuser (ou signaler) toute tension supérieure à, par exemple, 2 × W·k_dyn.
5. **C4 – Convention de tirette. Corrigé les 24 et 25 septembre 2026, voir `correction-C4.md`.** Convention « 180° = vers le haut » documentée et figée par test. Sur décision de Pierre, la tirette est renommée **pull-back** et restreinte à la verticale, 180° ± `pullBackToleranceDeg` (10° par défaut, comme Meyer Sound) : un pull-back est un second moteur qui porte le bas de la grappe et met une partie de la chaîne en compression. Toute direction vers le bas ou en biais est refusée.
6. **C5 – Signes et repères.** Unifier le sens de `fOrientationGlobal`/`fPivotGlobal` entre vol et stack (toujours « force sur le flanc du caisson porteur »), et exprimer `fAnchor`/`fLatch` dans le repère du caisson qui **porte** la paire (le bas en vol), avec `anchorHoleLocal` cohérent.
7. **C3 – `hinge_reversed`** = `f_pivot_global.dot(u) > 0` avec u de `hb`(haut) vers `pv`. Test proposé.
8. **C8 – Masses.** Décider si les 2 × 2,32 kg de barres par jonction et la masse du bumper (+ barre de déport) entrent dans le poids (Q3). Si oui : les ajouter comme charges sur la barre (à son CG) et sur le bumper.
9. Mineur : `bumper_model_legacy` est encore levé sur J0 alors que la liaison bumper a été remodélisée (drapeau obsolète, à retirer) ; `format_splay_grid` affiche 10,5 comme « 10° » ; `barMomentMaxNm` calculé sur les cotes déclarées quand tout le reste l'est sur les points construits (1,7 µm, sans effet, mais deux sources de vérité) ; `heightFromBottomMm` incohérent de 6,3 mm avec la bielle (Q1), ce qui fausse la silhouette et `elevation.pickupMm`.

Au 22 septembre, aucune de ces corrections n'était appliquée au code ; seul `tests/audit_corpus.rs` (harnais ignoré par défaut, lecture/écriture de fichiers sous variables d'environnement) avait été ajouté au crate. C4 a été appliquée le 24 septembre (`correction-C4.md`).

---

## 6. Questions pour Pierre

- **Q1** Les pions du bumper sont à 53,7 mm au-dessus de la face supérieure du caisson d'après bielle + barre, et `heightFromBottomMm = 60`. Le bumper pose-t-il sur la face supérieure ? Si oui, laquelle des cotes est fausse (60, ou 71,6 / 224,626) ?
- **Q2** Le trou 10,5° est-il percé sur la rangée 680 (extérieure), comme déclaré ? Et l'ancien 10° a-t-il disparu du perçage (il est refusé aujourd'hui) ?
- **Q3** Les 83,695 kg incluent-ils les deux barres arrière (2 × 2,32 kg) montées ? Masse du bumper et de la barre de déport ?
- **Q4** Stack : cotes des barres de bumper 10° et 20° (trou haut : abscisse et latéral depuis l'ancrage) ; la barre est-elle montée sur la même paire ancrage/verrou du caisson du bas ? Où et comment la cale en L reprend-elle la compression (bout de barre en appui ? sur quelle pièce ?) ; le bumper est-il attaché au sol ou libre ?
- **Q5** Tolérance de position réelle des perçages (flancs et barre, découpe laser + éventuel alésage) et diamètre de goupille réel (12,00 dans `settings`, 12,06 dans le brief).
- **Q6** Origine du k_dyn = 1,3 : choix interne ? Les sources lues donnent 1,1 (essai, directive) et +20 % (DGUV 215-313). 1,3 est conservateur ; il faut juste l'écrire comme un choix.
- **Q7** La tirette à 180° des deux exports est-elle bien un second moteur vertical au pied de la grappe (déchargeant la manille de 5,5 kN) ? Si l'intention était une tirette au sol vers l'arrière, les grappes sont à recalculer vers 300° (tension 10,2 kN, SF 2,98 sur « 14u 10m » d'après le corpus). **Réponse de Pierre (25 septembre 2026) : oui, c'est un pull-back vertical. Il n'y a pas de tirette tirant vers le bas ; le recalcul vers 300° est sans objet.**
- **Q8** Un seul moteur ou deux (bridle) sur la barre de déport à 650 mm ? Le `sharePerFlank = 0,5` suppose une charge centrée entre les flancs.

---

## 7. Fichiers livrés

| Fichier | Rôle |
|---|---|
| `sa303_ref.py` | solveur de référence (géométrie, cinématique, statique multi-corps, tirette, enveloppe, modèle ressort) |
| `test_sa303_ref.py` | 13 tests du solveur de référence (cas à la main, invariances, refus, ressorts) : `python3 test_sa303_ref.py` |
| `gen_corpus.py` → `corpus-definitions.json` | 57 grappes (1 à 14 caissons, toute la grille, J/banane/CCA, assiettes −15° à +35°, tirette 150° à 315°, stacks 0/10/20, cas invalides) |
| `../../../crates/sa303-core/tests/audit_corpus.rs` → `corpus-export.json` | calcul du corpus par le solveur Rust (`cargo test -p sa303-core --test audit_corpus -- --ignored`, chemins absolus dans `SA303_CORPUS_IN/OUT`) |
| `compare.py` → `comparison.md`, `corpus-reference.json` | comparaison champ par champ et résultats de référence |
| `analysis.py` | enveloppe axiale, raideur Huth, cas à la main, invariances, décomposition des comportements §2.12, stack |
| `tests_proposes_sa303_core.rs` | tests Rust à intégrer (équilibre par corps, colinéarité, fermetures, pickup, invariances, cas à la main, refus, plus 4 tests `#[ignore]` qui figent les corrections C1/C2/C3/C6) |

---

## 8. Mise à jour du 26 septembre 2026 : accroche sur trous percés

Le solveur n'accroche plus à un x continu mais sur les trous réellement percés du bumper et de sa barre de déport (1 ou 2 points). Le corpus a été adapté pour rester comparable, sans changer ce qu'il valide (les efforts de jonction) :

- `sa303-bumper` : un seul trou, au centre, à l'ancienne hauteur de manille (`height + shackleHeightAboveBumper`) : même pendaison libre qu'avant ;
- `sa303-bumper-portee-650` : trous à −650, 0 et +650 mm (l'ancienne portée de barre). Grappes à assiette imposée : 2 points (assiette tenue exactement) si l'ancien calcul n'avait pas de pull-back, 1 point sinon (le pull-back part alors du trou en bout de portée et tient l'assiette exacte, comme avant) ;
- `tieAngle` est reporté dans `pullBackAngle` : l'ancien code l'ignorait (champ renommé) et tirait toujours à 180°.

**Équivalence vérifiée** : angles de pull-back remis à « non choisi », l'export du nouveau code est identique champ par champ (écart relatif < 10⁻⁶) à celui du commit précédent sur les 53 grappes, hors position de la manille en accroche 2 points. L'export commité avant cette mise à jour était lui-même périmé (51 grappes calculées au lieu de 53).

**À faire** : `sa303_ref.py` lit encore `maxDeportMm` et le modèle d'accroche continu ; `compare.py` ne tourne plus tant qu'il n'est pas adapté aux trous (`rigging.shackleHoles`, `rigging.points`). `tests_proposes_sa303_core.rs` est une archive de septembre, non compilée.

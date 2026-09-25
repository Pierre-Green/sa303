# Correction C4 : convention et nature du pull-back

Dates : 24 et 25 septembre 2026. Suite du rapport `rapport-validation.md` : §0 ligne C4, §2.6, §5 point 5, question Q7.

---

## 1. Le problème

Le solveur exprime la direction de la « tirette » en degrés, dans la convention §2 de `dir_from_angle`. Cette convention part de 0° vers le bas et tourne dans le sens horaire. Dans cette convention, **180° pointe vers le haut**.

Le code disait « 180° = vers l'arrière ». Les deux exports réels ont un angle de 180°. Le solveur y calcule donc un câble qui **monte à la verticale** depuis la couronne du dernier caisson. C'est un second point de levage au pied de la grappe, le « vertical pull-back » de Meyer Sound. Le calcul était juste, mais la documentation le décrivait mal.

Le 24 septembre, la première correction documentait aussi une « tirette au sol » tirant vers le bas. Pierre l'a écartée le 25 septembre, pour les raisons qui suivent.

## 2. Décision de Pierre (25 septembre 2026)

- **Un pull-back tire verticalement vers le haut.** Son intérêt est de porter le bas de la grappe et de mettre une partie de la chaîne en compression. Un câble tiré vers le bas s'ajoute au poids : il remet la chaîne en traction et charge la manille. Il n'a pas d'usage ici.
- **La direction est limitée à 180° ± 10°**, comme dans les instructions Meyer Sound. La tolérance de 10° est réglable.
- **La « tirette » devient « pull-back »** partout, et « pull-back (compression) » à l'écran.
- **Toute notion de pull-back vers le bas disparaît.**
- **Q7 est tranchée.** Les exports à 180° sont bien des pull-backs verticaux, et le recalcul vers 300° est sans objet.

Seule la pratique Meyer Sound est appuyée par un document primaire lu : *Pull Back Rigging Operating Instructions*, PN 05.083.008.01, p. 3. Aucune source lue ne décrit de pull-back vers le bas.

## 3. Ce qui a été fait

### 3.1 Solveur

- **Fenêtre verticale.** La plage annoncée est l'intersection de deux ensembles : la fenêtre 180° ± tolérance et le demi-cercle où le câble tire, avec une tension positive. Si cette intersection est vide, la configuration est refusée. Le calcul est fait par la fonction `pull_back_usable_range_deg`.
- **Angle saisi hors plage.** Il n'est pas refusé : il est ramené sur la borne la plus proche (`clamp_to_pull_back_range`). Le champ de saisie fait de même, et le solveur le refait pour une grappe enregistrée dont la plage a bougé avec l'assiette. **La seule erreur restante est la traversée d'une enceinte.**
- **Bornes statiques.** Là où la plage est coupée par la statique, elle s'arrête 0,1° avant la borne, où la tension serait infinie (`PULL_BACK_STATIC_MARGIN_DEG`).
- **Champ de saisie.** Il porte `min`, `max` et un pas de 0,5°. Pendant la frappe, seule une valeur dans la plage part au solveur. En sortie de champ, la valeur est ramenée dans la plage.
- **Défaut.** Le solveur suggère 180°. Si 180° n'est pas dans la plage, il prend la borne la plus proche.
- **Tolérance réglable.** Le nouveau champ `Settings::pull_back_tolerance_deg` vaut 10° par défaut, et `pullBackToleranceDeg` en JSON. Les anciens fichiers de réglages, qui n'ont pas ce champ, prennent 10°. Aucun écran ne modifie encore les réglages : on change la tolérance dans le fichier de réglages.
- **Suppressions.** La suggestion « vers l'arrière », le libellé automatique et le champ exporté qui le portait sont supprimés. Le libellé ne servait qu'à distinguer une tirette au sol qui n'existe plus.
- **Inchangés.** La tension, la plage statique et le test de collision ne changent pas. Les efforts d'une grappe à 180° sont identiques à ceux d'avant.

### 3.2 Convention

- **Doc de `vector.rs`.** Elle énonce les quatre directions cardinales, dont « 180° = vers le haut ».
- **Nouveau test.** `angle_convention_cardinal_directions` fige la convention.

### 3.3 Vocabulaire

- **Code et interface.** « Tirette » devient « pull-back » dans les commentaires, les messages d'erreur, les infobulles et les popups. Il reste « pull-back (compression) » comme titre à l'écran et dans l'étiquette du viewer.
- **Point d'accroche.** Il est nommé « trou de couronne 0° de l'enceinte du bas » au lieu de « point 0° arrière-bas ».
- **Identifiants renommés.** À la demande de Pierre, plus aucun identifiant ne contient « tie ». Le module s'appelle `pull_back`, et les champs Rust `pull_back_*`. Les champs JSON sont `pullBackAngle`, `pullBackTensionN`, `pullBackPointGlobal`, `pullBackDirectionGlobal`, `pullBackDirectionAngleDeg` et `pullBackAngleRangeDeg`. Côté front, la couche du viewer s'appelle `PullBackOverlay.vue`. Les grappes livrées dans `src-tauri/assets/clusters` ont été migrées. Les grappes enregistrées par l'utilisateur avant ce changement perdent leur angle de pull-back, et le solveur reprend alors 180°.

### 3.4 Tests

| Test | Changement |
|---|---|
| Grappes de `golden.rs` à −20° avec pull-back à 0° | Passées à +20° avec pull-back à 180° |
| Collision | Grappe nez en l'air à −25° : le pull-back à 180° traverse la grappe et le calcul est refusé |
| Hors plage | 0° et 90° sont ramenés à 170° ; 195°, 270° et 300° à 190° |
| L'ancien test « la direction ne pousse jamais », qui reposait sur un pull-back à 90° | Remplacé par `pull_back_default_is_vertical_and_pulls` |
| `pull_back_tolerance_comes_from_the_settings` | Nouveau : 195° ramené à 190° à ±10°, gardé à ±20° |
| `user_can_pick_their_own_angle_inside_the_reported_range` | N'est plus ignoré |
| Unités de `pull_back/tension.rs` | Sept tests : fenêtre, rognage, absence de plage vers le bas, tolérance, bornage, défaut, direction |

## 4. Vérification

```
cargo test --workspace      : tous les tests passent, 0 échec
                              (les tests encore ignorés sont les harnais d'export)
cargo build --workspace     : OK (2 avertissements préexistants dans src-tauri/src/commands.rs)
pnpm exec vue-tsc --noEmit  : OK
```

Les 28 grappes livrées se calculent toujours : le test `every_seeded_cluster_asset_describes_a_buildable_cluster` appelle le solveur sur chacune. Aucune n'avait de pull-back vers le bas. Les cinq qui ont un pull-back sont à 180°.

## 5. Ce qui n'a pas été modifié

- **Les pièces datées de l'audit** restent telles quelles, parce que ce sont des preuves figées : le corpus, le solveur de référence Python, `comparison.md`, le corps du rapport et l'export `docs/audit/sa303-audit-29-grappes-2026-09-16.json`. Elles emploient encore « tirette » et `tieAngle`. Le corpus contient des tirettes entre 150° et 315°. Relancé avec le solveur actuel, le harnais `audit_corpus.rs` ignorera le champ `tieAngle`, et tous les pull-backs y seront pris à 180°. Ses résultats ne seront donc plus comparables à ceux de l'audit.
- **Les occurrences de « au sol » qui restent** concernent le stack posé au sol, pas le pull-back.
- **Grappes nez en l'air.** La plage annoncée n'est pas encore filtrée par le test de collision. Une plage peut donc s'afficher alors que le câble vertical traverse la grappe. Le calcul refuse alors la configuration avec un message explicite.

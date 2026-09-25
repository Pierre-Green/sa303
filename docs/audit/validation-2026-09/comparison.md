# Comparaison code Rust / référence Python

53 grappes, 450 jonctions.

Écart max sur tout le corpus, par champ. `réf`/`code` sont les valeurs (ou normes) là où l'écart est maximal.

| Champ | écart max abs | écart rel | grappe | jonction | réf | code |
|---|---|---|---|---|---|---|
| `anchorHoleGlobal (mm)` | 1.137e-12 | 5.54e-16 | iso14 20 partout free | 10 | 2052.54 | 2052.54 |
| `barAxialN` | 4.115e-04 | 1.23e-09 | INVALIDE tirette traverse la grappe (140°) | 4 | 334083 | 334083 |
| `barMomentAtPairNm` | 3.693e-05 | 1.08e-09 | INVALIDE tirette traverse la grappe (140°) | 12 | 34289.2 | 34289.2 |
| `barMomentMaxNm` | 5.467e-01 | 5.52e-05 | INVALIDE tirette traverse la grappe (140°) | 7 | 9905.65 | 9905.1 |
| `barShearN` | 9.744e-05 | 1.08e-09 | INVALIDE tirette traverse la grappe (140°) | 12 | 90468.6 | 90468.6 |
| `bumper fPairAnchorGlobal` | 1.964e-04 | 1.32e-09 | INVALIDE tirette traverse la grappe (140°) |  | 148970 | 148970 |
| `bumper fPairAnchorN` | 1.958e-04 | 1.31e-09 | INVALIDE tirette traverse la grappe (140°) |  | 148970 | 148970 |
| `bumper fPairLatchN` | 1.614e-04 | 1.28e-09 | INVALIDE tirette traverse la grappe (140°) |  | 126460 | 126460 |
| `bumper orientationForceGlobal` | 2.531e-04 | 1.21e-09 | INVALIDE tirette traverse la grappe (140°) |  | 208982 | 208982 |
| `bumper orientationForceN` | 2.530e-04 | 1.21e-09 | INVALIDE tirette traverse la grappe (140°) |  | 208982 | 208982 |
| `bumper orientationPointGlobal (mm)` | 2.344e-13 | 5.12e-16 | iso11+cca3 free |  | 457.64 | 457.64 |
| `bumper pairAnchorHoleGlobal (mm)` | 1.025e-13 | 3.01e-16 | iso14 20 partout free |  | 340.463 | 340.463 |
| `bumper pairMomentNm` | 1.281e-05 | 1.42e-09 | INVALIDE tirette traverse la grappe (140°) |  | -9040.89 | -9040.89 |
| `bumper pinPairMomentNm` | 1.104e-04 | 1.21e-09 | INVALIDE tirette traverse la grappe (140°) |  | 91477.4 | 91477.4 |
| `bumper pinSpanMm` | 2.274e-13 | 3.46e-16 | iso2 droit free |  | 656.831 | 656.831 |
| `bumper pivotForceGlobal` | 8.863e-05 | 1.21e-09 | INVALIDE tirette traverse la grappe (140°) |  | 73021.6 | 73021.6 |
| `bumper pivotForceN` | 8.863e-05 | 1.21e-09 | INVALIDE tirette traverse la grappe (140°) |  | 73021.6 | 73021.6 |
| `bumper pivotPointGlobal (mm)` | 1.137e-13 | 2.41e-16 | iso14 20 partout free |  | 471.803 | 471.803 |
| `bumper supportForceGlobal` | 3.346e-04 | 1.21e-09 | INVALIDE tirette traverse la grappe (140°) |  | 275615 | 275615 |
| `bumper supportForceN` | 3.343e-04 | 1.21e-09 | INVALIDE tirette traverse la grappe (140°) |  | 275615 | 275615 |
| `cg (mm)` | 1.438e-12 | 4.21e-16 | J14 tilt +9 barre (proche max) |  | 3414.1 | 3414.1 |
| `colinéarité bielle |u×fPivot|/|fPivot|` | 2.561e-15 | 2.56e-06 | iso14 20 partout free | 4 | 0 | 2.56061e-15 |
| `couronne polaire vs barre (mm)` | 1.686e-03 | 1.00e+00 | iso11+cca3 free | 12 | 0 | 0.00168625 |
| `fAnchorGlobal (N)` | 4.299e-04 | 1.08e-09 | INVALIDE tirette traverse la grappe (140°) | 12 | 399162 | 399162 |
| `fAnchorN` | 4.299e-04 | 1.08e-09 | INVALIDE tirette traverse la grappe (140°) | 12 | 399162 | 399162 |
| `fLatchGlobal (N)` | 3.359e-04 | 1.08e-09 | INVALIDE tirette traverse la grappe (140°) | 12 | 311910 | 311910 |
| `fLatchN` | 3.359e-04 | 1.08e-09 | INVALIDE tirette traverse la grappe (140°) | 12 | 311910 | 311910 |
| `fOrientation (local) vs global tourné` | 2.439e+03 | 2.00e+00 | stack 4 cca bas 20 | 2 | 1219.49 | 1219.49 |
| `fOrientationAngleDeg` | 3.600e+02 | 1.00e+00 | droit 8 tilt 0 | 0 | 360 | 0 |
| `fOrientationGlobal (N)` | 2.439e+03 | 2.00e+00 | stack 4 cca bas 20 | 2 | 1219.49 | 1219.49 |
| `fOrientationN` | 4.115e-04 | 1.23e-09 | INVALIDE tirette traverse la grappe (140°) | 4 | 334097 | 334097 |
| `fPivotGlobal (N)` | 1.113e+03 | 2.00e+00 | stack 4 bas 20 | 2 | 556.719 | 556.719 |
| `fPivotN` | 2.414e-04 | 1.24e-09 | INVALIDE tirette traverse la grappe (140°) | 4 | 194542 | 194542 |
| `fermeture ht(bas)=pv(haut) (mm)` | 9.099e-13 | 9.10e-04 | J14 tilt -6 sans tirette | 9 | 0 | 9.09939e-13 |
| `hingeReversed` | 1.000e+00 | 1.00e+00 | iso8 20 partout free | 6 | 1 | 0 |
| `latchHoleGlobal (mm)` | 1.137e-12 | 5.66e-16 | iso14 20 partout free | 10 | 2009.37 | 2009.37 |
| `loadedOrientationHoleGlobal (mm)` | 9.095e-13 | 4.07e-16 | iso14 20 partout free | 9 | 2235.99 | 2235.99 |
| `loadedPivotHoleGlobal (mm)` | 9.095e-13 | 3.24e-16 | iso14 20 partout free | 9 | 2809.39 | 2809.39 |
| `moment global autour du pickup (N·mm)` | 3.725e-08 | 1.00e+00 | INVALIDE tirette traverse la grappe (140°) |  | 0 | -3.72529e-08 |
| `phiFreeHang (rad)` | 2.220e-16 | 1.97e-16 | iso14 20 partout free |  | -1.12588 | -1.12588 |
| `phiInitial (rad)` | 2.220e-16 | 1.97e-16 | iso14 20 partout free |  | -1.12588 | -1.12588 |
| `pickupGlobal (mm)` | 1.271e-13 | 2.43e-16 | J14 tilt -6 sans tirette |  | 523.973 | 523.973 |
| `row` | 0.000e+00 | 0.00e+00 | iso2 droit free | 0 | 0 | 0 |
| `résidu force corps (N)` | 2.277e-05 | 1.00e+00 | INVALIDE tirette traverse la grappe (140°) | 4 | 0 | 2.27729e-05 |
| `résidu moment corps (N·mm)` | 1.884e-05 | 1.00e+00 | INVALIDE tirette traverse la grappe (140°) | 9 | 0 | 1.88355e-05 |
| `speakers[].o (mm)` | 9.095e-13 | 3.70e-16 | iso14 20 partout free | 9 | 2459.17 | 2459.17 |
| `speakers[].phi (rad)` | 2.220e-16 | 1.97e-16 | iso14 20 partout free | 0 | -1.12588 | -1.12588 |
| `stack: réaction sol = W·k` | 2.416e-09 | 5.66e-13 | stack 4 cca bas 20 |  | 4267.99 | 4267.99 |
| `stack: supportForceN` | 0.000e+00 | 0.00e+00 | stack 3 droit 0 |  | 3200.99 | 3200.99 |
| `support = W·k − tirette` | 7.276e-12 | 4.98e-16 | J14 tilt +0 tie180 |  | 14602.4 | 14602.4 |
| `tieAngleRangeDeg[0]` | 2.842e-14 | 2.18e-16 | J14 tilt +35 tie180 |  | 130.293 | 130.293 |
| `tieAngleRangeDeg[1]` | 5.684e-14 | 1.83e-16 | J14 tilt +35 tie180 |  | 310.293 | 310.293 |
| `tieDirectionGlobal` | 1.570e-16 | 1.57e-16 | INVALIDE tirette traverse la grappe (140°) |  | 1 | 1 |
| `tiePointGlobal (mm)` | 0.000e+00 | 0.00e+00 | J14 tilt +0 tie180 |  | 6669.93 | 6669.93 |
| `tieTensionN` | 5.821e-11 | 2.03e-16 | INVALIDE tirette traverse la grappe (140°) |  | 287063 | 287063 |
| `totalMassKg` | 4.547e-13 | 3.88e-16 | iso14 droit free |  | 1171.73 | 1171.73 |

## Résidus de moment exportés > 1 N·mm

Le champ `momentResidualNmm` de l'export n'est pas un résidu d'équilibre dès qu'une tirette est active (voir rapport §2.6, correction C3).

| grappe | jonction | momentResidualNmm |
|---|---|---|
| J14 tilt +0 tie180 | 0 | 5.559e+05 |
| J14 tilt +0 tie180 | 1 | 5.357e+05 |
| J14 tilt +0 tie180 | 2 | 5.127e+05 |
| J14 tilt +0 tie180 | 3 | 4.867e+05 |
| J14 tilt +0 tie180 | 4 | 4.574e+05 |
| J14 tilt +0 tie180 | 5 | 4.249e+05 |
| J14 tilt +0 tie180 | 6 | 3.891e+05 |
| J14 tilt +0 tie180 | 7 | 3.498e+05 |
| J14 tilt +0 tie180 | 8 | 3.067e+05 |
| J14 tilt +0 tie180 | 9 | 2.602e+05 |
| J14 tilt +0 tie180 | 10 | 2.102e+05 |
| J14 tilt +0 tie180 | 11 | 1.570e+05 |
| J14 tilt +0 tie180 | 12 | 1.014e+05 |
| J14 tilt +3 tie180 | 0 | 3.028e+06 |
| J14 tilt +3 tie180 | 1 | 2.904e+06 |
| J14 tilt +3 tie180 | 2 | 2.766e+06 |
| J14 tilt +3 tie180 | 3 | 2.613e+06 |
| J14 tilt +3 tie180 | 4 | 2.444e+06 |
| J14 tilt +3 tie180 | 5 | 2.259e+06 |
| J14 tilt +3 tie180 | 6 | 2.057e+06 |
| J14 tilt +3 tie180 | 7 | 1.839e+06 |
| J14 tilt +3 tie180 | 8 | 1.602e+06 |
| J14 tilt +3 tie180 | 9 | 1.349e+06 |
| J14 tilt +3 tie180 | 10 | 1.078e+06 |
| J14 tilt +3 tie180 | 11 | 7.929e+05 |
| J14 tilt +3 tie180 | 12 | 4.972e+05 |
| J14 tilt +8 tie180 | 0 | 6.366e+06 |
| J14 tilt +8 tie180 | 1 | 6.064e+06 |
| J14 tilt +8 tie180 | 2 | 5.736e+06 |
| J14 tilt +8 tie180 | 3 | 5.381e+06 |
| J14 tilt +8 tie180 | 4 | 4.998e+06 |
| J14 tilt +8 tie180 | 5 | 4.586e+06 |
| J14 tilt +8 tie180 | 6 | 4.145e+06 |
| J14 tilt +8 tie180 | 7 | 3.673e+06 |
| J14 tilt +8 tie180 | 8 | 3.169e+06 |
| J14 tilt +8 tie180 | 9 | 2.636e+06 |
| J14 tilt +8 tie180 | 10 | 2.073e+06 |
| J14 tilt +8 tie180 | 11 | 1.486e+06 |
| J14 tilt +8 tie180 | 12 | 8.861e+05 |
| J14 tilt +15 tie180 | 0 | 1.014e+07 |
| J14 tilt +15 tie180 | 1 | 9.582e+06 |
| J14 tilt +15 tie180 | 2 | 8.996e+06 |
| J14 tilt +15 tie180 | 3 | 8.374e+06 |
| J14 tilt +15 tie180 | 4 | 7.715e+06 |
| J14 tilt +15 tie180 | 5 | 7.018e+06 |
| J14 tilt +15 tie180 | 6 | 6.284e+06 |
| J14 tilt +15 tie180 | 7 | 5.511e+06 |
| J14 tilt +15 tie180 | 8 | 4.698e+06 |
| J14 tilt +15 tie180 | 9 | 3.848e+06 |
| J14 tilt +15 tie180 | 10 | 2.963e+06 |
| J14 tilt +15 tie180 | 11 | 2.050e+06 |
| J14 tilt +15 tie180 | 12 | 1.133e+06 |
| J14 tilt +25 tie180 | 0 | 1.447e+07 |
| J14 tilt +25 tie180 | 1 | 1.357e+07 |
| J14 tilt +25 tie180 | 2 | 1.263e+07 |
| J14 tilt +25 tie180 | 3 | 1.165e+07 |
| J14 tilt +25 tie180 | 4 | 1.063e+07 |
| J14 tilt +25 tie180 | 5 | 9.574e+06 |
| J14 tilt +25 tie180 | 6 | 8.477e+06 |
| J14 tilt +25 tie180 | 7 | 7.341e+06 |
| J14 tilt +25 tie180 | 8 | 6.163e+06 |
| J14 tilt +25 tie180 | 9 | 4.948e+06 |
| J14 tilt +25 tie180 | 10 | 3.701e+06 |
| J14 tilt +25 tie180 | 11 | 2.432e+06 |
| J14 tilt +25 tie180 | 12 | 1.182e+06 |
| J14 tilt +35 tie180 | 0 | 1.790e+07 |
| J14 tilt +35 tie180 | 1 | 1.667e+07 |
| J14 tilt +35 tie180 | 2 | 1.541e+07 |
| J14 tilt +35 tie180 | 3 | 1.411e+07 |
| J14 tilt +35 tie180 | 4 | 1.278e+07 |
| J14 tilt +35 tie180 | 5 | 1.141e+07 |
| J14 tilt +35 tie180 | 6 | 1.000e+07 |
| J14 tilt +35 tie180 | 7 | 8.564e+06 |
| J14 tilt +35 tie180 | 8 | 7.089e+06 |
| J14 tilt +35 tie180 | 9 | 5.585e+06 |
| J14 tilt +35 tie180 | 10 | 4.058e+06 |
| J14 tilt +35 tie180 | 11 | 2.521e+06 |
| J14 tilt +35 tie180 | 12 | 1.033e+06 |
| J14 tilt +25 tie150 | 0 | 2.987e+07 |
| J14 tilt +25 tie150 | 1 | 2.908e+07 |
| J14 tilt +25 tie150 | 2 | 2.811e+07 |
| J14 tilt +25 tie150 | 3 | 2.694e+07 |
| J14 tilt +25 tie150 | 4 | 2.557e+07 |
| J14 tilt +25 tie150 | 5 | 2.399e+07 |
| J14 tilt +25 tie150 | 6 | 2.219e+07 |
| J14 tilt +25 tie150 | 7 | 2.017e+07 |
| J14 tilt +25 tie150 | 8 | 1.790e+07 |
| J14 tilt +25 tie150 | 9 | 1.540e+07 |
| J14 tilt +25 tie150 | 10 | 1.267e+07 |
| J14 tilt +25 tie150 | 11 | 9.728e+06 |
| J14 tilt +25 tie150 | 12 | 6.584e+06 |
| J14 tilt +25 tie180 | 0 | 1.447e+07 |
| J14 tilt +25 tie180 | 1 | 1.357e+07 |
| J14 tilt +25 tie180 | 2 | 1.263e+07 |
| J14 tilt +25 tie180 | 3 | 1.165e+07 |
| J14 tilt +25 tie180 | 4 | 1.063e+07 |
| J14 tilt +25 tie180 | 5 | 9.574e+06 |
| J14 tilt +25 tie180 | 6 | 8.477e+06 |
| J14 tilt +25 tie180 | 7 | 7.341e+06 |
| J14 tilt +25 tie180 | 8 | 6.163e+06 |
| J14 tilt +25 tie180 | 9 | 4.948e+06 |
| J14 tilt +25 tie180 | 10 | 3.701e+06 |
| J14 tilt +25 tie180 | 11 | 2.432e+06 |
| J14 tilt +25 tie180 | 12 | 1.182e+06 |
| J14 tilt +25 tie210 | 0 | 1.170e+07 |
| J14 tilt +25 tie210 | 1 | 1.078e+07 |
| J14 tilt +25 tie210 | 2 | 9.844e+06 |
| J14 tilt +25 tie210 | 3 | 8.901e+06 |
| J14 tilt +25 tie210 | 4 | 7.946e+06 |
| J14 tilt +25 tie210 | 5 | 6.983e+06 |
| J14 tilt +25 tie210 | 6 | 6.011e+06 |
| J14 tilt +25 tie210 | 7 | 5.035e+06 |
| J14 tilt +25 tie210 | 8 | 4.053e+06 |
| J14 tilt +25 tie210 | 9 | 3.068e+06 |
| J14 tilt +25 tie210 | 10 | 2.087e+06 |
| J14 tilt +25 tie210 | 11 | 1.120e+06 |
| J14 tilt +25 tie210 | 12 | 2.105e+05 |
| J14 tilt +25 tie240 | 0 | 9.906e+06 |
| J14 tilt +25 tie240 | 1 | 8.970e+06 |
| J14 tilt +25 tie240 | 2 | 8.041e+06 |
| J14 tilt +25 tie240 | 3 | 7.119e+06 |
| J14 tilt +25 tie240 | 4 | 6.206e+06 |
| J14 tilt +25 tie240 | 5 | 5.303e+06 |
| J14 tilt +25 tie240 | 6 | 4.413e+06 |
| J14 tilt +25 tie240 | 7 | 3.540e+06 |
| J14 tilt +25 tie240 | 8 | 2.685e+06 |
| J14 tilt +25 tie240 | 9 | 1.850e+06 |
| J14 tilt +25 tie240 | 10 | 1.042e+06 |
| J14 tilt +25 tie240 | 11 | 2.698e+05 |
| J14 tilt +25 tie240 | 12 | 4.189e+05 |
| J14 tilt +25 tie270 | 0 | 7.719e+06 |
| J14 tilt +25 tie270 | 1 | 6.767e+06 |
| J14 tilt +25 tie270 | 2 | 5.842e+06 |
| J14 tilt +25 tie270 | 3 | 4.947e+06 |
| J14 tilt +25 tie270 | 4 | 4.084e+06 |
| J14 tilt +25 tie270 | 5 | 3.255e+06 |
| J14 tilt +25 tie270 | 6 | 2.465e+06 |
| J14 tilt +25 tie270 | 7 | 1.718e+06 |
| J14 tilt +25 tie270 | 8 | 1.017e+06 |
| J14 tilt +25 tie270 | 9 | 3.653e+05 |
| J14 tilt +25 tie270 | 10 | 2.331e+05 |
| J14 tilt +25 tie270 | 11 | 7.668e+05 |
| J14 tilt +25 tie270 | 12 | 1.186e+06 |
| J14 tilt +25 tie300 | 0 | 1.503e+06 |
| J14 tilt +25 tie300 | 1 | 5.053e+05 |
| J14 tilt +25 tie300 | 2 | 4.075e+05 |
| J14 tilt +25 tie300 | 3 | 1.227e+06 |
| J14 tilt +25 tie300 | 4 | 1.947e+06 |
| J14 tilt +25 tie300 | 5 | 2.565e+06 |
| J14 tilt +25 tie300 | 6 | 3.073e+06 |
| J14 tilt +25 tie300 | 7 | 3.460e+06 |
| J14 tilt +25 tie300 | 8 | 3.722e+06 |
| J14 tilt +25 tie300 | 9 | 3.855e+06 |
| J14 tilt +25 tie300 | 10 | 3.856e+06 |
| J14 tilt +25 tie300 | 11 | 3.713e+06 |
| J14 tilt +25 tie300 | 12 | 3.367e+06 |
| J14 tilt +25 tie315 | 0 | 2.533e+07 |
| J14 tilt +25 tie315 | 1 | 2.653e+07 |
| J14 tilt +25 tie315 | 2 | 2.738e+07 |
| J14 tilt +25 tie315 | 3 | 2.788e+07 |
| J14 tilt +25 tie315 | 4 | 2.798e+07 |
| J14 tilt +25 tie315 | 5 | 2.769e+07 |
| J14 tilt +25 tie315 | 6 | 2.698e+07 |
| J14 tilt +25 tie315 | 7 | 2.581e+07 |
| J14 tilt +25 tie315 | 8 | 2.418e+07 |
| J14 tilt +25 tie315 | 9 | 2.208e+07 |
| J14 tilt +25 tie315 | 10 | 1.950e+07 |
| J14 tilt +25 tie315 | 11 | 1.643e+07 |
| J14 tilt +25 tie315 | 12 | 1.278e+07 |
| J14 tilt +12 tie270 (faible) | 0 | 4.444e+06 |
| J14 tilt +12 tie270 (faible) | 1 | 3.968e+06 |
| J14 tilt +12 tie270 (faible) | 2 | 3.500e+06 |
| J14 tilt +12 tie270 (faible) | 3 | 3.041e+06 |
| J14 tilt +12 tie270 (faible) | 4 | 2.593e+06 |
| J14 tilt +12 tie270 (faible) | 5 | 2.156e+06 |
| J14 tilt +12 tie270 (faible) | 6 | 1.732e+06 |
| J14 tilt +12 tie270 (faible) | 7 | 1.324e+06 |
| J14 tilt +12 tie270 (faible) | 8 | 9.314e+05 |
| J14 tilt +12 tie270 (faible) | 9 | 5.565e+05 |
| J14 tilt +12 tie270 (faible) | 10 | 2.018e+05 |
| J14 tilt +12 tie270 (faible) | 11 | 1.273e+05 |
| J14 tilt +12 tie270 (faible) | 12 | 4.065e+05 |
| J14 tilt +9 barre (proche max) | 0 | 6.956e+06 |
| J14 tilt +9 barre (proche max) | 1 | 6.618e+06 |
| J14 tilt +9 barre (proche max) | 2 | 6.253e+06 |
| J14 tilt +9 barre (proche max) | 3 | 5.859e+06 |
| J14 tilt +9 barre (proche max) | 4 | 5.435e+06 |
| J14 tilt +9 barre (proche max) | 5 | 4.980e+06 |
| J14 tilt +9 barre (proche max) | 6 | 4.494e+06 |
| J14 tilt +9 barre (proche max) | 7 | 3.977e+06 |
| J14 tilt +9 barre (proche max) | 8 | 3.425e+06 |
| J14 tilt +9 barre (proche max) | 9 | 2.842e+06 |
| J14 tilt +9 barre (proche max) | 10 | 2.228e+06 |
| J14 tilt +9 barre (proche max) | 11 | 1.589e+06 |
| J14 tilt +9 barre (proche max) | 12 | 9.378e+05 |
| INVALIDE tirette traverse la grappe (140°) | 0 | 2.793e+08 |
| INVALIDE tirette traverse la grappe (140°) | 1 | 2.803e+08 |
| INVALIDE tirette traverse la grappe (140°) | 2 | 2.788e+08 |
| INVALIDE tirette traverse la grappe (140°) | 3 | 2.746e+08 |
| INVALIDE tirette traverse la grappe (140°) | 4 | 2.675e+08 |
| INVALIDE tirette traverse la grappe (140°) | 5 | 2.575e+08 |
| INVALIDE tirette traverse la grappe (140°) | 6 | 2.444e+08 |
| INVALIDE tirette traverse la grappe (140°) | 7 | 2.279e+08 |
| INVALIDE tirette traverse la grappe (140°) | 8 | 2.081e+08 |
| INVALIDE tirette traverse la grappe (140°) | 9 | 1.847e+08 |
| INVALIDE tirette traverse la grappe (140°) | 10 | 1.580e+08 |
| INVALIDE tirette traverse la grappe (140°) | 11 | 1.279e+08 |
| INVALIDE tirette traverse la grappe (140°) | 12 | 9.409e+07 |
| Grappe 14u - 10m- 0-60m | 0 | 1.447e+07 |
| Grappe 14u - 10m- 0-60m | 1 | 1.357e+07 |
| Grappe 14u - 10m- 0-60m | 2 | 1.263e+07 |
| Grappe 14u - 10m- 0-60m | 3 | 1.165e+07 |
| Grappe 14u - 10m- 0-60m | 4 | 1.063e+07 |
| Grappe 14u - 10m- 0-60m | 5 | 9.574e+06 |
| Grappe 14u - 10m- 0-60m | 6 | 8.477e+06 |
| Grappe 14u - 10m- 0-60m | 7 | 7.341e+06 |
| Grappe 14u - 10m- 0-60m | 8 | 6.163e+06 |
| Grappe 14u - 10m- 0-60m | 9 | 4.948e+06 |
| Grappe 14u - 10m- 0-60m | 10 | 3.701e+06 |
| Grappe 14u - 10m- 0-60m | 11 | 2.432e+06 |
| Grappe 14u - 10m- 0-60m | 12 | 1.182e+06 |
| Grappe - 14u - 12m - 0-80m | 0 | 4.512e+06 |
| Grappe - 14u - 12m - 0-80m | 1 | 4.303e+06 |
| Grappe - 14u - 12m - 0-80m | 2 | 4.079e+06 |
| Grappe - 14u - 12m - 0-80m | 3 | 3.839e+06 |
| Grappe - 14u - 12m - 0-80m | 4 | 3.579e+06 |
| Grappe - 14u - 12m - 0-80m | 5 | 3.299e+06 |
| Grappe - 14u - 12m - 0-80m | 6 | 2.998e+06 |
| Grappe - 14u - 12m - 0-80m | 7 | 2.672e+06 |
| Grappe - 14u - 12m - 0-80m | 8 | 2.323e+06 |
| Grappe - 14u - 12m - 0-80m | 9 | 1.948e+06 |
| Grappe - 14u - 12m - 0-80m | 10 | 1.547e+06 |
| Grappe - 14u - 12m - 0-80m | 11 | 1.120e+06 |
| Grappe - 14u - 12m - 0-80m | 12 | 6.619e+05 |

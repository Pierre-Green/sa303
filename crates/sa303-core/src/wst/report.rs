//! Assemblage du rapport WST : convertit les unités d'affichage (mm, degrés,
//! kHz) vers le SI attendu par `super::formulas`, évalue chaque critère sur la
//! grille demandée, et rend le tout prêt à afficher — le front ne fait aucune
//! formule (brief §1), il met en page.
//!
//! Le pas entre centres acoustiques (`STEP`) peut venir de deux sources : une
//! saisie manuelle (hauteur de caisse + jour), ou la géométrie réelle d'une
//! enceinte. Dans ce second cas il **dépend de l'angle** : le pivot avant fait
//! s'ouvrir le jour en façade quand on incline, donc chaque splay de la grille
//! a son propre pas, et donc son propre ARF.

use super::formulas::*;
use crate::cluster::{build_cluster, ChainSpeaker};
use crate::speaker::{SpeakerModel, WaveguideFront};
use serde::{Deserialize, Serialize};

/// Fréquences d'analyse par défaut (Hz) : celles du tableau du papier,
/// complétées vers le bas pour voir le régime « pas de lobe possible ».
pub const ANALYSIS_FREQUENCIES_HZ: [f64; 8] = [
    500.0, 1000.0, 2000.0, 4000.0, 8000.0, 12_000.0, 16_000.0, 20_000.0,
];

/// Nature du front rayonné par un élément — c'est elle qui décide quels
/// critères s'appliquent, et c'est la distinction que le papier ne fait pas
/// explicitement parce qu'il ne traite que le premier cas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GuideKind {
    /// Front plan (guide isophase type DOSC). Angler deux fronts plans ouvre
    /// entre eux une zone sans énergie : c'est exactement la figure 23 du
    /// papier, donc le critère 5 est ici un vrai verdict.
    Isophase,
    /// Front courbé rayonnant déjà un secteur `coverage_deg`. Les secteurs
    /// voisins se juxtaposent : il n'y a plus de zone vide à refermer, donc le
    /// critère 5 ne s'applique pas. Ce qui le remplace est un critère de
    /// directivité — couvrir l'angle et se raccorder à plat.
    #[serde(rename_all = "camelCase")]
    Curved {
        /// Ouverture verticale effective du guide, degrés.
        coverage_deg: f64,
        /// Niveau du guide à la moitié du splay (dB, négatif), relevé en
        /// simulation ou en mesure. −6 dB donne un raccord plat.
        level_at_half_splay_db: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WstInputs {
    /// Célérité du son, m/s. `SPEED_OF_SOUND_PAPER` pour recouper les chiffres
    /// publiés, 343 pour la réalité.
    pub speed_of_sound: f64,
    /// Hauteur de caisse, mm. Ignorée si une enceinte est sélectionnée.
    pub box_height_mm: f64,
    /// Jour en façade, mm. Ignoré si une enceinte est sélectionnée : il est
    /// alors dérivé de la géométrie, angle par angle.
    pub gap_mm: f64,
    /// Hauteur rayonnante D, mm.
    pub radiating_height_mm: f64,
    /// Nombre de caisses de la ligne.
    pub speaker_count: usize,
    /// Angles entre caisses à évaluer, degrés.
    pub splays_deg: Vec<f64>,
    /// Distances d'auditeur à évaluer, m. La plus courte est le pire cas.
    pub distances_m: Vec<f64>,
    /// Fréquence haute de la bande utile, Hz.
    pub f_max_hz: f64,
    /// Nature du front rayonné : décide des critères applicables.
    pub guide: GuideKind,
}

/// Grandeurs dérivées à l'angle de référence (le premier de `splays_deg`).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WstDerived {
    pub step_mm: f64,
    pub gap_mm: f64,
    pub arf: f64,
    pub line_height_m: f64,
    /// Bouche de guide réellement utilisée, mm — celle de l'enceinte quand elle
    /// en déclare une, sinon la saisie.
    pub radiating_height_mm: f64,
    /// Guide réellement utilisé : le front décide des critères applicables, et
    /// l'écran doit pouvoir montrer lequel a servi.
    pub guide: GuideKind,
    /// `true` si le pas vient de la géométrie d'une enceinte (il varie alors
    /// avec l'angle), `false` s'il est saisi à la main.
    pub step_from_speaker: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Criterion1 {
    pub arf: f64,
    pub arf_min: f64,
    pub satisfied: bool,
    /// `None` à ARF ≥ 1 : ligne continue, pas de lobe de réseau.
    pub side_lobe_attenuation_db: Option<f64>,
    pub axial_loss_db: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Criterion2Sample {
    pub frequency_hz: f64,
    /// `true` tant que le pas reste sous λ/2 : aucun lobe possible.
    pub satisfied: bool,
    pub grating_lobe_deg: Option<f64>,
    pub first_dip_deg: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Criterion2 {
    pub frequency_limit_hz: f64,
    pub samples: Vec<Criterion2Sample>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Criterion3 {
    pub f_max_hz: f64,
    pub max_deviation_mm: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NearFieldSample {
    pub frequency_hz: f64,
    /// `None` quand λ ≥ H : pas de champ proche à cette fréquence.
    pub boundary_m: Option<f64>,
    pub boundary_fresnel_m: f64,
    pub first_dip_deg: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NearField {
    pub no_near_field_below_hz: f64,
    pub samples: Vec<NearFieldSample>,
}

/// Une ligne du tableau du critère 5 : un angle, le pas et l'ARF qu'il
/// implique, et la fréquence tenable pour chaque distance évaluée.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Criterion5Row {
    pub splay_deg: f64,
    pub step_mm: f64,
    pub arf: f64,
    /// Même ordre que `WstInputs::distances_m`. `None` : aucune fréquence
    /// tenable (angle nul ou négatif).
    pub f_max_by_distance_hz: Vec<Option<f64>>,
}

/// L'angle maximal admissible à une fréquence, pour chaque distance évaluée.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Criterion5AngleLimit {
    pub frequency_hz: f64,
    /// `None` : aucun angle n'est admissible — la caisse est trop grande pour
    /// ce premier rang à cette fréquence.
    pub max_splay_deg_by_distance: Vec<Option<f64>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Criterion5 {
    pub rows: Vec<Criterion5Row>,
    pub angle_limits: Vec<Criterion5AngleLimit>,
    /// Pas maximal laissant un angle positif possible à `f_max` pour la
    /// distance la plus courte évaluée.
    pub max_step_mm: Option<f64>,
    pub closest_distance_m: Option<f64>,
}

/// Critère 4 — courbure variable, évalué angle par angle.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvatureRow {
    pub splay_deg: f64,
    /// `None` à 0° : rayon infini, la ligne est plate.
    pub radius_m: Option<f64>,
    /// Produit `α·d` (rad·m) à la distance la plus courte : constant le long
    /// d'une ligne bien courbée.
    pub angle_distance_product: Option<f64>,
    pub relative_level_db: Option<f64>,
    /// En dessous de cet angle, la ligne se comporte comme plate à `f_max`.
    pub curved_model_min_splay_deg: f64,
}

/// Géométrie de l'arc en angle constant (CCA), angle par angle. Purement
/// géométrique : vraie quelle que soit la nature du guide.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CcaRow {
    pub splay_deg: f64,
    pub radius_m: Option<f64>,
    /// Flèche d'une corde plate sur l'arc d'une caisse.
    pub sagitta_mm: Option<f64>,
    /// Au-dessus de cette fréquence, la flèche dépasse λ/4 : la courbure du
    /// front doit être juste. En dessous, des cordes plates approximent l'arc
    /// et la courbure du guide est indifférente.
    pub curvature_matters_above_hz: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideDelaySample {
    /// Hauteur sur la bouche, mm (de −D/2 à +D/2).
    pub y_mm: f64,
    pub delay_mm: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cca {
    pub rows: Vec<CcaRow>,
}

/// Un angle évalué face à un guide à front courbé.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvedGuideRow {
    pub splay_deg: f64,
    /// `θ_guide ≥ α` : les secteurs voisins se juxtaposent sans laisser de
    /// trou. C'est le critère qui remplace le critère 5 pour ce type de guide.
    pub covered: bool,
    /// Rayon que le front du guide doit viser à cet angle : `STEP / α`.
    pub target_radius_m: Option<f64>,
    /// Fréquence au-dessus de laquelle ce rayon doit être juste.
    pub curvature_matters_above_hz: Option<f64>,
}

/// Verdicts propres à un guide à front courbé. Le critère 5 du papier ne
/// s'applique pas ici : un élément qui rayonne déjà un secteur n'ouvre aucune
/// zone vide avec son voisin, quelle que soit la distance d'écoute.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvedGuide {
    pub coverage_deg: f64,
    /// Splay maximal : l'ouverture du guide elle-même.
    pub max_splay_deg: f64,
    /// Niveau relevé du guide à la moitié du splay (dB, négatif).
    pub level_at_half_splay_db: f64,
    /// Niveau au raccord entre deux secteurs voisins : deux contributions
    /// égales s'additionnent, soit +6 dB. 0 dB = raccord plat ; négatif = creux
    /// au raccord ; positif = bosse.
    pub splice_level_db: f64,
    pub rows: Vec<CurvedGuideRow>,
    /// Raccord entre une caisse isophase et une caisse à guide courbé : les
    /// deux fronts sont tangents à θ/2.
    pub transition_splay_deg: f64,
    /// Profil de retard visé sur la bouche, à l'angle de référence — de quoi
    /// vérifier le guide contre sa cible.
    pub guide_delay_profile: Vec<GuideDelaySample>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WstReport {
    pub derived: WstDerived,
    pub criterion1: Criterion1,
    pub criterion2: Criterion2,
    pub criterion3: Criterion3,
    pub near_field: NearField,
    /// Renseigné **uniquement** pour un guide à front plan : c'est le cas que
    /// traite le papier. Pour un guide à front courbé, la géométrie qui fonde
    /// ce critère n'existe pas — le verdict est alors dans `curved_guide`.
    pub criterion5: Option<Criterion5>,
    pub curvature: Vec<CurvatureRow>,
    pub cca: Cca,
    /// Renseigné uniquement pour un guide à front courbé.
    pub curved_guide: Option<CurvedGuide>,
}

/// Jour en façade entre deux enceintes adjacentes à ce splay : distance entre
/// le coin avant-bas de celle du haut et le coin avant-haut de celle du bas.
///
/// La charnière est en façade mais légèrement en retrait de la face : incliner
/// fait donc bâiller la façade, d'autant plus que l'angle est grand. On mesure
/// ce jour sur deux enceintes réellement assemblées, en réutilisant la
/// cinématique déjà validée plutôt qu'une approximation trigonométrique.
pub fn front_gap_mm(model: &SpeakerModel, splay_deg: f64) -> f64 {
    let chain = [
        ChainSpeaker::from_model(model),
        ChainSpeaker::from_model(model),
    ];
    let speakers = build_cluster(&chain, &[splay_deg], 0.0);
    let upper = speakers[0];
    let lower = speakers[1];
    // Silhouette en repère enceinte : 0 = coin avant-haut, 3 = coin avant-bas.
    let upper_front_bottom = upper.o + upper.outline[3].rotate(upper.phi);
    let lower_front_top = lower.o + lower.outline[0].rotate(lower.phi);
    (lower_front_top - upper_front_bottom).norm()
}

/// Pas entre centres acoustiques de deux enceintes adjacentes, à ce splay :
/// `hauteur de caisse + jour en façade`. Ce n'est donc pas une constante — il
/// s'ouvre avec l'angle, et l'ARF baisse d'autant.
pub fn acoustic_step_mm(model: &SpeakerModel, splay_deg: f64) -> f64 {
    model.mechanical.height + front_gap_mm(model, splay_deg)
}

/// Le front, la bouche du guide et son secteur décrivent l'enceinte, pas
/// l'analyse : quand une enceinte est fournie, ce sont ses valeurs qui font
/// foi et les champs saisis correspondants sont ignorés. Sans enceinte (étude
/// d'une caisse qui n'est pas au catalogue), tout vient de la saisie.
fn resolve_guide(inputs: &WstInputs, speaker: Option<&SpeakerModel>) -> GuideKind {
    let Some(model) = speaker else {
        return inputs.guide.clone();
    };
    match model.acoustics.wg_front {
        WaveguideFront::Isophase => GuideKind::Isophase,
        WaveguideFront::ConstantCurvature => GuideKind::Curved {
            coverage_deg: model.acoustics.directivity_vertical,
            level_at_half_splay_db: model.acoustics.wg_level_at_half_coverage_db,
        },
    }
}

/// Rapport complet. `speaker` sélectionnée : le pas (et donc l'ARF) est dérivé
/// de sa géométrie pour chaque angle, et le guide vient de son modèle
/// acoustique. Sinon, pas constant `box_height + gap` et guide saisi.
pub fn wst_report(inputs: &WstInputs, speaker: Option<&SpeakerModel>) -> WstReport {
    let c = inputs.speed_of_sound;
    let guide = resolve_guide(inputs, speaker);
    // Une bouche à 0 mm n'est pas une enceinte : c'est un modèle dont le champ
    // n'a jamais été renseigné. On retombe alors sur la saisie plutôt que de
    // rendre un ARF nul qui ferait échouer tous les critères sans raison.
    let radiating_height_mm = match speaker {
        Some(model) if model.acoustics.wg_output_height > 0.0 => model.acoustics.wg_output_height,
        _ => inputs.radiating_height_mm,
    };
    let radiating_height_m = radiating_height_mm / 1000.0;
    let splays_deg = if inputs.splays_deg.is_empty() {
        vec![0.0]
    } else {
        inputs.splays_deg.clone()
    };

    let step_m_at = |splay_deg: f64| match speaker {
        Some(model) => acoustic_step_mm(model, splay_deg) / 1000.0,
        None => (inputs.box_height_mm + inputs.gap_mm) / 1000.0,
    };
    let box_height_m = match speaker {
        Some(model) => model.mechanical.height / 1000.0,
        None => inputs.box_height_mm / 1000.0,
    };

    // Référence : le premier angle de la grille — c'est lui qui décrit la ligne
    // pour les critères qui ne dépendent pas de l'angle.
    let reference_splay_deg = splays_deg[0];
    let step_m = step_m_at(reference_splay_deg);
    let reference_arf = arf(radiating_height_m, step_m);
    let line_height_m = inputs.speaker_count as f64 * step_m;

    let derived = WstDerived {
        step_mm: step_m * 1000.0,
        gap_mm: (step_m - box_height_m) * 1000.0,
        arf: reference_arf,
        line_height_m,
        radiating_height_mm,
        guide: guide.clone(),
        step_from_speaker: speaker.is_some(),
    };

    let arf_min_value = arf_min(inputs.speaker_count);
    let criterion1 = Criterion1 {
        arf: reference_arf,
        arf_min: arf_min_value,
        satisfied: reference_arf >= arf_min_value,
        side_lobe_attenuation_db: side_lobe_attenuation_db(reference_arf),
        axial_loss_db: axial_loss_db(reference_arf),
    };

    let frequency_limit_hz = frequency_limit_half_wavelength(c, step_m);
    let criterion2 = Criterion2 {
        frequency_limit_hz,
        samples: ANALYSIS_FREQUENCIES_HZ
            .iter()
            .map(|&frequency_hz| {
                let lambda = wavelength_m(c, frequency_hz);
                Criterion2Sample {
                    frequency_hz,
                    satisfied: frequency_hz <= frequency_limit_hz,
                    grating_lobe_deg: grating_lobe_angle_rad(lambda, step_m).map(f64::to_degrees),
                    first_dip_deg: first_dip_angle_rad(lambda, step_m).map(f64::to_degrees),
                }
            })
            .collect(),
    };

    let criterion3 = Criterion3 {
        f_max_hz: inputs.f_max_hz,
        max_deviation_mm: max_wavefront_deviation_m(c, inputs.f_max_hz) * 1000.0,
    };

    let near_field = NearField {
        no_near_field_below_hz: no_near_field_below_hz(c, line_height_m),
        samples: ANALYSIS_FREQUENCIES_HZ
            .iter()
            .map(|&frequency_hz| {
                let lambda = wavelength_m(c, frequency_hz);
                NearFieldSample {
                    frequency_hz,
                    boundary_m: near_field_boundary_m(line_height_m, lambda),
                    boundary_fresnel_m: near_field_boundary_fresnel_m(line_height_m, lambda),
                    first_dip_deg: line_first_dip_angle_rad(lambda, line_height_m)
                        .map(f64::to_degrees),
                }
            })
            .collect(),
    };

    let closest_distance_m = inputs
        .distances_m
        .iter()
        .copied()
        .filter(|d| *d > 0.0)
        .fold(None::<f64>, |acc, d| Some(acc.map_or(d, |a: f64| a.min(d))));

    // Le critère 5 naît de la zone sans énergie qui s'ouvre entre deux fronts
    // **plans** anglés (papier, fig. 23). Un guide qui rayonne déjà un secteur
    // n'a pas cette géométrie : ses secteurs voisins se juxtaposent, il n'y a
    // rien à refermer. Lui appliquer le critère répondrait à une autre question
    // que celle posée — d'où l'option plutôt qu'un chiffre trompeur.
    let criterion5 = matches!(guide, GuideKind::Isophase).then(|| Criterion5 {
        rows: splays_deg
            .iter()
            .map(|&splay_deg| {
                let row_step_m = step_m_at(splay_deg);
                let row_arf = arf(radiating_height_m, row_step_m);
                Criterion5Row {
                    splay_deg,
                    step_mm: row_step_m * 1000.0,
                    arf: row_arf,
                    f_max_by_distance_hz: inputs
                        .distances_m
                        .iter()
                        .map(|&distance_m| {
                            max_frequency_hz(
                                c,
                                splay_deg.to_radians(),
                                row_arf,
                                row_step_m,
                                distance_m,
                            )
                        })
                        .collect(),
                }
            })
            .collect(),
        angle_limits: ANALYSIS_FREQUENCIES_HZ
            .iter()
            .map(|&frequency_hz| Criterion5AngleLimit {
                frequency_hz,
                max_splay_deg_by_distance: inputs
                    .distances_m
                    .iter()
                    .map(|&distance_m| {
                        let splay =
                            max_splay_rad(c, frequency_hz, reference_arf, step_m, distance_m);
                        (splay > 0.0).then(|| splay.to_degrees())
                    })
                    .collect(),
            })
            .collect(),
        max_step_mm: closest_distance_m
            .map(|d| max_step_m(c, inputs.f_max_hz, reference_arf, d) * 1000.0),
        closest_distance_m,
    });

    let lambda_at_f_max = wavelength_m(c, inputs.f_max_hz);
    let curvature = splays_deg
        .iter()
        .map(|&splay_deg| {
            let splay_rad = splay_deg.to_radians();
            let row_step_m = step_m_at(splay_deg);
            CurvatureRow {
                splay_deg,
                radius_m: curvature_radius_m(row_step_m, splay_rad),
                angle_distance_product: closest_distance_m.map(|d| splay_rad * d),
                relative_level_db: closest_distance_m
                    .map(|d| curved_line_relative_level_db(splay_rad, d, row_step_m)),
                curved_model_min_splay_deg: curved_model_min_splay_rad(
                    row_step_m,
                    lambda_at_f_max,
                    line_height_m,
                )
                .to_degrees(),
            }
        })
        .collect();

    // Géométrie pure de l'arc : vraie quelle que soit la nature du guide.
    // `curvature_matters_above_hz` est la fréquence où la flèche d'une corde
    // plate sur l'arc dépasse λ/4. En dessous, des cordes plates approximent
    // l'arc et la courbure du front est indifférente ; au-dessus, elle doit
    // être juste.
    let reference_radius_m = curvature_radius_m(step_m, reference_splay_deg.to_radians());
    let arc_row = |splay_deg: f64| {
        let splay_rad = splay_deg.to_radians();
        let radius_m = curvature_radius_m(step_m_at(splay_deg), splay_rad);
        let sagitta_m = radius_m.map(|r| cca_sagitta_m(r, splay_rad));
        (radius_m, sagitta_m)
    };
    let cca = Cca {
        rows: splays_deg
            .iter()
            .map(|&splay_deg| {
                let (radius_m, sagitta_m) = arc_row(splay_deg);
                CcaRow {
                    splay_deg,
                    radius_m,
                    sagitta_mm: sagitta_m.map(|s| s * 1000.0),
                    curvature_matters_above_hz: sagitta_m
                        .and_then(|s| isophase_frequency_limit(c, s)),
                }
            })
            .collect(),
    };

    // Guide à front courbé : le verdict n'est plus « à quelle fréquence le trou
    // se referme » mais « le guide couvre-t-il l'angle, et se raccorde-t-il à
    // plat ». Deux secteurs voisins se recouvrent au raccord, donc deux
    // contributions égales s'y additionnent : +6 dB sur le niveau relevé à α/2.
    let curved_guide = match guide {
        GuideKind::Isophase => None,
        GuideKind::Curved {
            coverage_deg,
            level_at_half_splay_db,
        } => Some(CurvedGuide {
            coverage_deg,
            max_splay_deg: coverage_deg,
            level_at_half_splay_db,
            splice_level_db: level_at_half_splay_db + 6.0,
            rows: splays_deg
                .iter()
                .map(|&splay_deg| {
                    let (radius_m, sagitta_m) = arc_row(splay_deg);
                    CurvedGuideRow {
                        splay_deg,
                        covered: coverage_deg >= splay_deg,
                        target_radius_m: radius_m,
                        curvature_matters_above_hz: sagitta_m
                            .and_then(|s| isophase_frequency_limit(c, s)),
                    }
                })
                .collect(),
            transition_splay_deg: transition_splay_rad(coverage_deg.to_radians()).to_degrees(),
            guide_delay_profile: guide_delay_profile(reference_radius_m, radiating_height_m),
        }),
    };

    WstReport {
        derived,
        criterion1,
        criterion2,
        criterion3,
        near_field,
        criterion5,
        curvature,
        cca,
        curved_guide,
    }
}

/// Profil de retard sur la hauteur de bouche, échantillonné du bas au haut.
/// Vide si la ligne est plate (rayon infini) : il n'y a alors rien à courber.
fn guide_delay_profile(radius_m: Option<f64>, radiating_height_m: f64) -> Vec<GuideDelaySample> {
    const SAMPLES: usize = 9;
    let Some(radius_m) = radius_m else {
        return Vec::new();
    };
    (0..SAMPLES)
        .map(|i| {
            let t = i as f64 / (SAMPLES - 1) as f64;
            let y_m = radiating_height_m * (t - 0.5);
            GuideDelaySample {
                y_mm: y_m * 1000.0,
                delay_mm: guide_path_delay_m(radius_m, y_m) * 1000.0,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speaker::{Crown, Hinge, SpeakerAcousticsModel, SpeakerMechanicalModel};

    /// Le critère 5 n'est renseigné qu'en front plan ; ces tests-là sont tous
    /// en front plan, donc l'absence est un échec de test, pas un cas métier.
    fn criterion5(report: &WstReport) -> &Criterion5 {
        report
            .criterion5
            .as_ref()
            .expect("front plan : le critère 5 s'applique")
    }

    fn sa303() -> SpeakerModel {
        SpeakerModel {
            id: "sa303".into(),
            name: "SA303".into(),
            schema_version: 1,
            mechanical: SpeakerMechanicalModel {
                depth: 700.0,
                height: 550.0,
                total_vertical_angle: 20.0,
                mass_kg: 83.695,
                cg: [10.84, 11.91],
                hinge: Hinge {
                    x: -338.43,
                    y: 257.13,
                    joint_separation: 552.384,
                },
                crown: Crown {
                    radius: 680.0,
                    delta: 20.0,
                    anchor_angle: 2.5,
                    splay0_angle: 5.0,
                },
                splay_grid: vec![0.0, 5.0, 10.0],
                frame_hole_splay: 0.0,
            },
            acoustics: Default::default(),
            compatible_below: Vec::new(),
        }
    }

    #[test]
    fn acoustic_step_at_zero_splay_is_the_joint_separation() {
        // À plat, les deux faces avant sont parallèles : le pas vaut exactement
        // l'entraxe de charnière, soit la hauteur de caisse plus le jour.
        let model = sa303();
        let step = acoustic_step_mm(&model, 0.0);
        assert!(
            (step - model.mechanical.hinge.joint_separation).abs() < 1e-9,
            "obtenu {step}"
        );
        assert!(step > model.mechanical.height, "le jour doit être positif");
    }

    #[test]
    fn front_gap_opens_with_the_splay_because_the_pivot_sits_behind_the_face() {
        // La charnière est en façade mais en retrait de la face : incliner fait
        // bâiller la façade, de façon monotone.
        let model = sa303();
        let flat = front_gap_mm(&model, 0.0);
        let mid = front_gap_mm(&model, 5.0);
        let wide = front_gap_mm(&model, 20.0);
        assert!(flat < mid && mid < wide, "{flat} / {mid} / {wide}");
        // À plat, le jour vaut l'entraxe moins la hauteur de caisse.
        assert!(
            (flat - (model.mechanical.hinge.joint_separation - model.mechanical.height)).abs()
                < 1e-9
        );
        // Et ça reste un jour de quelques millimètres, pas une ouverture franche.
        assert!(wide < 20.0, "jour inattendu de {wide} mm à 20°");
    }

    #[test]
    fn selecting_a_speaker_derives_step_and_arf_per_angle() {
        let model = sa303();
        let inputs = WstInputs {
            speed_of_sound: SPEED_OF_SOUND_DEFAULT,
            box_height_mm: 0.0, // ignoré : l'enceinte fournit la géométrie
            gap_mm: 0.0,
            radiating_height_mm: 430.0,
            speaker_count: 6,
            splays_deg: vec![0.0, 5.0, 10.0],
            distances_m: vec![10.0, 25.0],
            f_max_hz: 16_000.0,
            guide: GuideKind::Isophase,
        };
        let report = wst_report(&inputs, Some(&model));

        assert!(report.derived.step_from_speaker);
        assert!(
            (report.derived.gap_mm - 2.384).abs() < 1e-6,
            "{}",
            report.derived.gap_mm
        );

        // Le pas s'ouvrant avec l'angle, l'ARF baisse : c'est exactement ce que
        // le sélecteur d'enceinte apporte par rapport à un pas saisi une fois.
        let arfs: Vec<f64> = criterion5(&report).rows.iter().map(|r| r.arf).collect();
        assert!(arfs[0] > arfs[1] && arfs[1] > arfs[2], "{arfs:?}");

        // Plus l'angle est grand, plus la fréquence tenable chute.
        let f = |row: usize| criterion5(&report).rows[row].f_max_by_distance_hz[0].unwrap();
        assert!(f(0) > f(1) && f(1) > f(2));
    }

    #[test]
    fn without_a_speaker_the_step_is_the_manual_height_plus_gap() {
        let inputs = WstInputs {
            speed_of_sound: SPEED_OF_SOUND_PAPER,
            box_height_mm: 550.0,
            gap_mm: 8.0,
            radiating_height_mm: 470.0,
            speaker_count: 6,
            splays_deg: vec![5.0],
            distances_m: vec![25.0],
            f_max_hz: 16_000.0,
            guide: GuideKind::Isophase,
        };
        let report = wst_report(&inputs, None);

        assert!(!report.derived.step_from_speaker);
        assert!((report.derived.step_mm - 558.0).abs() < 1e-9);
        assert!((report.derived.arf - 470.0 / 558.0).abs() < 1e-12);
        // Ligne de 6 caisses : H = 6 × 0,558.
        assert!((report.derived.line_height_m - 3.348).abs() < 1e-9);
        // Le pas ne bouge pas d'un angle à l'autre en saisie manuelle.
        assert!((criterion5(&report).rows[0].step_mm - 558.0).abs() < 1e-9);
    }

    #[test]
    fn an_angle_too_wide_for_the_front_row_reports_no_valid_splay() {
        // Premier rang très proche : à 20 kHz aucun angle ne passe.
        let inputs = WstInputs {
            speed_of_sound: SPEED_OF_SOUND_DEFAULT,
            box_height_mm: 550.0,
            gap_mm: 8.0,
            radiating_height_mm: 470.0,
            speaker_count: 6,
            splays_deg: vec![0.0],
            distances_m: vec![2.0],
            f_max_hz: 20_000.0,
            guide: GuideKind::Isophase,
        };
        let report = wst_report(&inputs, None);
        let top = criterion5(&report)
            .angle_limits
            .last()
            .expect("20 kHz est dans la grille");
        assert_eq!(top.frequency_hz, 20_000.0);
        assert!(
            top.max_splay_deg_by_distance[0].is_none(),
            "aucun angle ne devrait être admissible"
        );
    }

    /// Base commune aux deux tests de guide : une caisse type A15, exploitée à
    /// 20° et écoutée à 5 m — le cas qui rendait le critère 5 absurde.
    fn tight_cca_inputs(guide: GuideKind) -> WstInputs {
        WstInputs {
            speed_of_sound: SPEED_OF_SOUND_DEFAULT,
            box_height_mm: 372.0,
            gap_mm: 0.0,
            radiating_height_mm: 340.0,
            speaker_count: 12,
            splays_deg: vec![20.0],
            distances_m: vec![5.0],
            f_max_hz: 16_000.0,
            guide,
        }
    }

    #[test]
    fn a_flat_wavefront_guide_is_the_case_criterion5_actually_judges() {
        // Front plan à 20° : le critère s'applique et rend un verdict sévère,
        // qui est le bon — c'est la limite d'un guide *plan* monté en CCA.
        let report = wst_report(&tight_cca_inputs(GuideKind::Isophase), None);
        let f = criterion5(&report).rows[0].f_max_by_distance_hz[0]
            .expect("20° à 5 m donne une fréquence finie");
        assert!(f < 6_000.0, "obtenu {f} Hz");
        assert!(report.curved_guide.is_none());
    }

    #[test]
    fn a_curved_wavefront_guide_gets_no_criterion5_verdict() {
        // Même caisse, même angle, mais le guide rayonne déjà 20° : les secteurs
        // voisins se juxtaposent, aucune zone vide ne s'ouvre. Le critère 5 ne
        // s'applique pas — il ne doit donc pas produire de chiffre du tout.
        let report = wst_report(
            &tight_cca_inputs(GuideKind::Curved {
                coverage_deg: 20.0,
                level_at_half_splay_db: -6.0,
            }),
            None,
        );
        assert!(report.criterion5.is_none());

        let guide = report.curved_guide.expect("guide courbé renseigné");
        assert_eq!(guide.max_splay_deg, 20.0);
        assert!(guide.rows[0].covered, "20° de guide couvrent 20° de splay");
        // −6 dB à mi-angle : les deux secteurs voisins se somment à plat.
        assert!((guide.splice_level_db - 0.0).abs() < 1e-12);
    }

    #[test]
    fn splicing_two_sectors_adds_six_db_and_a_narrow_guide_leaves_a_hole() {
        // Guide de 20° exploité à 24° : l'angle dépasse la couverture, et le
        // niveau relevé plus bas creuse le raccord.
        let report = wst_report(
            &WstInputs {
                splays_deg: vec![24.0],
                ..tight_cca_inputs(GuideKind::Curved {
                    coverage_deg: 20.0,
                    level_at_half_splay_db: -8.5,
                })
            },
            None,
        );
        let guide = report.curved_guide.expect("guide courbé renseigné");
        assert!(!guide.rows[0].covered, "24° > 20° de couverture");
        assert!((guide.splice_level_db - (-2.5)).abs() < 1e-12);
        // Raccord vers une caisse isophase : tangence à la moitié du secteur.
        assert!((guide.transition_splay_deg - 10.0).abs() < 1e-9);
    }

    #[test]
    fn the_speaker_supplies_the_guide_and_overrides_what_was_typed() {
        // Le front et la bouche décrivent l'enceinte : une saisie contradictoire
        // ne doit pas pouvoir la faire passer pour ce qu'elle n'est pas.
        let mut model = sa303();
        model.acoustics = SpeakerAcousticsModel {
            fs: 60.0,
            directivity_horizontal: 90.0,
            directivity_vertical: 20.0,
            wg_front: WaveguideFront::ConstantCurvature,
            wg_output_height: 470.0,
            wg_level_at_half_coverage_db: -8.0,
        };

        let inputs = WstInputs {
            radiating_height_mm: 1.0,   // saisie absurde, à ignorer
            guide: GuideKind::Isophase, // contredit l'enceinte
            ..tight_cca_inputs(GuideKind::Isophase)
        };
        let report = wst_report(&inputs, Some(&model));

        assert!((report.derived.radiating_height_mm - 470.0).abs() < 1e-9);
        assert!(report.criterion5.is_none(), "l'enceinte est à front courbé");
        let guide = report.curved_guide.expect("guide courbé renseigné");
        assert_eq!(guide.coverage_deg, 20.0);
        assert!((guide.splice_level_db - (-2.0)).abs() < 1e-12);
    }

    #[test]
    fn a_speaker_without_a_declared_mouth_falls_back_to_the_typed_value() {
        // Modèle ancien ou incomplet : mieux vaut la saisie qu'un ARF nul qui
        // ferait échouer tous les critères pour une raison invisible.
        let model = sa303(); // acoustics par défaut : bouche à 0
        assert_eq!(model.acoustics.wg_output_height, 0.0);
        let report = wst_report(&tight_cca_inputs(GuideKind::Isophase), Some(&model));
        assert!((report.derived.radiating_height_mm - 340.0).abs() < 1e-9);
        assert!(report.derived.arf > 0.0);
    }

    #[test]
    fn arc_geometry_is_reported_whatever_the_guide_is() {
        // Rayon et flèche ne dépendent que de l'angle et du pas : la nature du
        // guide ne doit rien y changer. Seule leur lecture change.
        let flat = wst_report(&tight_cca_inputs(GuideKind::Isophase), None);
        let curved = wst_report(
            &tight_cca_inputs(GuideKind::Curved {
                coverage_deg: 20.0,
                level_at_half_splay_db: -6.0,
            }),
            None,
        );
        assert_eq!(flat.cca.rows.len(), 1);
        assert_eq!(
            flat.cca.rows[0].radius_m.map(f64::to_bits),
            curved.cca.rows[0].radius_m.map(f64::to_bits)
        );
        assert_eq!(
            flat.cca.rows[0]
                .curvature_matters_above_hz
                .map(f64::to_bits),
            curved.curved_guide.as_ref().unwrap().rows[0]
                .curvature_matters_above_hz
                .map(f64::to_bits)
        );
    }
}

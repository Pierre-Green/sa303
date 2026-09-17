//! Export d'audit : un JSON autoportant décrivant une sélection de grappes,
//! destiné à être relu par un tiers qui n'a pas le logiciel.
//!
//! « Autoportant » est l'exigence qui dicte la structure. Un rapport qui dirait
//! « jonction 3 : 14 kN au verrou » sans dire de quelle barre, de quelle
//! enceinte et sous quel coefficient de sécurité ne serait pas vérifiable : le
//! relecteur ne pourrait ni refaire le calcul, ni contester une cote. D'où
//! l'ordre du document — **les définitions d'abord, les résultats ensuite** —
//! et le fait que les réglages y figurent au même titre que les pièces.
//!
//! Ne sont exportés que les modèles **réellement référencés** par la sélection.
//! Livrer tout le catalogue noierait la pièce à auditer dans des modèles qui ne
//! participent pas, et laisserait croire qu'ils ont été pris en compte.
//!
//! Les grappes qui ne se résolvent pas ne sont pas silencieusement omises :
//! elles partent dans `impossible` avec leur raison. Un export où il manque
//! trois grappes sans explication est un export qu'on ne peut pas recouper avec
//! l'écran dont il sort.

use crate::bumper::{BumperBarModel, BumperModel};
use crate::checks::{check_bar, utilization, BarCheck, SandwichSpec};
use crate::cluster::{compute_cluster, BumperView, Cluster, ClusterResult, JointResult};
use crate::settings::Settings;
use crate::speaker::SpeakerModel;
use serde::Serialize;

/// Version du format d'export. Distincte des `schemaVersion` des modèles : ce
/// document-ci a sa propre vie, et un relecteur doit pouvoir dire quelle
/// structure il lit.
pub const EXPORT_SCHEMA_VERSION: u32 = 1;

/// Les cinq chemins de charge d'une jonction, chacun rapporté à son admissible.
/// Rendus séparément et non fondus en un seul taux : savoir *lequel* gouverne
/// est l'information qui dit quoi renforcer.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JointChecks {
    /// Sandwich à la goupille de couronne.
    pub utilization_crown: f64,
    /// Sandwich aux goupilles de bielle.
    pub utilization_bielle: f64,
    pub utilization_anchor: f64,
    pub utilization_latch: f64,
    /// Flexion composée de la barre arrière.
    pub utilization_bar: f64,
    pub utilization_worst: f64,
    /// Nom du chemin qui gouverne, pour n'avoir pas à recomparer cinq nombres.
    pub governing_path: &'static str,
    /// Détail par section de la barre, avec la section la plus sollicitée.
    pub bar: BarCheck,
}

impl JointChecks {
    fn of(joint: &JointResult, spec: &SandwichSpec) -> Self {
        let bar = check_bar(
            &joint.rear_bar,
            joint.splay_deg,
            &joint.bar_loads(),
            spec.safety_factor,
            Some(joint.rear_face_x),
            Some(joint.bar_rear_edge_max_x),
        );
        let utilization_crown = utilization(joint.f_orientation_n, spec);
        let utilization_bielle = utilization(joint.f_pivot_n, spec);
        let utilization_anchor = utilization(joint.f_anchor_n, spec);
        let utilization_latch = utilization(joint.f_latch_n, spec);
        let utilization_bar = bar.utilization();

        let paths = [
            ("couronne", utilization_crown),
            ("bielle", utilization_bielle),
            ("ancrage", utilization_anchor),
            ("verrou", utilization_latch),
            ("flexion de barre", utilization_bar),
        ];
        let (governing_path, utilization_worst) = paths
            .into_iter()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .expect("la liste des chemins n'est pas vide");

        Self {
            utilization_crown,
            utilization_bielle,
            utilization_anchor,
            utilization_latch,
            utilization_bar,
            utilization_worst,
            governing_path,
            bar,
        }
    }
}

/// Les chemins de charge de l'accrochage du bumper, chacun rapporté à son
/// admissible — le pendant de [`JointChecks`] pour la liaison qui porte la
/// grappe entière.
///
/// Sans eux le document se contredisait : il vérifiait les cinq chemins de
/// chaque jonction, mais pas ceux qui reprennent *tout* ce qui pend. Un
/// relecteur pouvait lire un coefficient de sécurité de grappe sans savoir si
/// les pions du bumper tenaient.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperChecks {
    /// Sandwich au pion avant du bumper.
    pub utilization_front_pin: Option<f64>,
    /// Sandwich au pion arrière.
    pub utilization_rear_pin: Option<f64>,
    /// Sandwich à l'ancrage et au verrou où la barre du bumper est boulonnée.
    /// `None` en stack : le bumper y est goupillé sans barre.
    pub utilization_bar_anchor: Option<f64>,
    pub utilization_bar_latch: Option<f64>,
    pub utilization_worst: f64,
    /// Nom du chemin qui gouverne, ou `None` si rien n'est renseigné.
    pub governing_path: Option<&'static str>,
}

impl BumperChecks {
    fn of(view: &BumperView, spec: &SandwichSpec) -> Self {
        let at = |force_n: Option<f64>| force_n.map(|n| utilization(n, spec));
        let utilization_front_pin = at(view.pivot_force_n);
        let utilization_rear_pin = at(view.orientation_force_n);
        let utilization_bar_anchor = at(view.f_pair_anchor_n);
        let utilization_bar_latch = at(view.f_pair_latch_n);

        let paths = [
            ("pion avant", utilization_front_pin),
            ("pion arrière", utilization_rear_pin),
            ("ancrage barre bumper", utilization_bar_anchor),
            ("verrou barre bumper", utilization_bar_latch),
        ];
        let governing = paths
            .into_iter()
            .filter_map(|(name, u)| u.map(|u| (name, u)))
            .max_by(|(_, a), (_, b)| a.total_cmp(b));

        Self {
            utilization_front_pin,
            utilization_rear_pin,
            utilization_bar_anchor,
            utilization_bar_latch,
            utilization_worst: governing.map_or(0.0, |(_, u)| u),
            governing_path: governing.map(|(name, _)| name),
        }
    }
}

/// Une grappe résolue : sa définition telle qu'enregistrée, le résultat complet
/// du solveur, et les vérifications jonction par jonction.
///
/// La définition accompagne le résultat plutôt que d'être seulement référencée :
/// c'est elle qui dit quels splays ont été retenus et quelle assiette imposée,
/// donc sans elle le résultat n'est pas reproductible.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterExport {
    pub definition: Cluster,
    pub result: ClusterResult,
    /// Un par jonction, même ordre que `result.joints`.
    pub joint_checks: Vec<JointChecks>,
    /// L'accrochage du bumper : pions, et paire de sa barre en vol.
    pub bumper_checks: BumperChecks,
    /// Le pire taux de la grappe, tous chemins confondus — jonctions **et**
    /// accrochage du bumper. Laisser ce dernier dehors donnait un coefficient
    /// de sécurité qui ignorait la liaison portant toute la grappe.
    pub utilization_worst: f64,
    /// Coefficient de sécurité réel de la grappe : l'inverse du pire taux. Le
    /// taux est rapporté à `R_m / sf`, donc un taux de 0,5 sous `sf = 4` veut
    /// dire qu'il reste un facteur 8 à la rupture.
    pub safety_factor: f64,
    /// Le même, recalculé à `k_dyn = 1,1` au lieu du facteur dynamique des
    /// réglages. C'est la valeur qu'on compare à Soundvision, qui ne dynamise
    /// pas de la même façon — les mettre côte à côte sans le dire ferait croire
    /// à un désaccord de calcul là où il n'y a qu'un coefficient différent.
    ///
    /// Les réglages ne sont **pas** modifiés pour autant : la grappe est
    /// recalculée à part.
    pub safety_factor_static: f64,
}

/// Facteur dynamique de la comparaison Soundvision. Figé : il ne suit pas les
/// réglages, sinon les deux colonnes finiraient par dire la même chose.
pub const STATIC_COMPARISON_K_DYN: f64 = 1.1;

/// Grappe écartée du calcul, avec la raison. Jamais omise en silence.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpossibleClusterExport {
    pub id: String,
    pub name: String,
    pub reason: String,
}

/// Les pièces réellement référencées par la sélection, et elles seules.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Definitions {
    pub speakers: Vec<SpeakerModel>,
    pub bumpers: Vec<BumperModel>,
    pub bumper_bars: Vec<BumperBarModel>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditExport {
    pub schema_version: u32,
    /// Horodatage fourni par l'appelant (ISO 8601). Le cœur de calcul n'a pas
    /// d'horloge — et ne doit pas en avoir, sinon deux exports du même état ne
    /// seraient plus comparables octet à octet en test.
    pub generated_at: String,
    /// Réglages sous lesquels tout ce qui suit a été calculé : coefficient de
    /// sécurité, facteur dynamique, gravité, part par flanc, goupille et tôles.
    /// Sans eux, aucun taux de travail du document n'est reproductible.
    pub settings: Settings,
    pub definitions: Definitions,
    pub clusters: Vec<ClusterExport>,
    pub impossible: Vec<ImpossibleClusterExport>,
}

/// Construit l'export pour la sélection de grappes donnée.
///
/// Les catalogues passés sont les catalogues **complets** : c'est cette
/// fonction qui en extrait les pièces référencées, pour que l'appelant n'ait
/// pas à refaire la résolution des identifiants — et n'ait pas l'occasion de la
/// faire différemment du solveur.
pub fn build_audit_export(
    generated_at: String,
    selection: &[Cluster],
    speakers: &[SpeakerModel],
    bumpers: &[BumperModel],
    bumper_bars: &[BumperBarModel],
    settings: &Settings,
) -> AuditExport {
    let spec = SandwichSpec::from_settings(settings);
    let mut clusters = Vec::new();
    let mut impossible = Vec::new();

    for cluster in selection {
        let Some(bumper) = bumpers.iter().find(|b| b.id == cluster.bumper_model_id) else {
            impossible.push(ImpossibleClusterExport {
                id: cluster.id.clone(),
                name: cluster.name.clone(),
                reason: format!("bumper « {} » introuvable", cluster.bumper_model_id),
            });
            continue;
        };
        match compute_cluster(speakers, cluster, settings, bumper, bumper_bars) {
            Err(e) => impossible.push(ImpossibleClusterExport {
                id: cluster.id.clone(),
                name: cluster.name.clone(),
                reason: e.reason,
            }),
            Ok(result) => {
                let joint_checks: Vec<JointChecks> = result
                    .joints
                    .iter()
                    .map(|j| JointChecks::of(j, &spec))
                    .collect();
                let bumper_checks = BumperChecks::of(&result.bumper_view, &spec);
                let utilization_worst = joint_checks
                    .iter()
                    .map(|c| c.utilization_worst)
                    .fold(bumper_checks.utilization_worst, f64::max);
                // Même grappe, même barre, seul `k_dyn` change. Recalculée
                // plutôt que mise à l'échelle : les efforts ne sont pas tous
                // proportionnels au poids — la tirette, elle, est résolue pour
                // tenir une assiette, donc la règle de trois serait fausse.
                let mut static_settings = settings.clone();
                static_settings.dynamic_factor = STATIC_COMPARISON_K_DYN;
                let static_worst =
                    compute_cluster(speakers, cluster, &static_settings, bumper, bumper_bars)
                        .ok()
                        .map(|r| {
                            let spec = SandwichSpec::from_settings(&static_settings);
                            // Même périmètre que `utilization_worst` ci-dessus,
                            // bumper compris : sinon les deux colonnes ne
                            // compareraient pas la même chose.
                            let bumper = BumperChecks::of(&r.bumper_view, &spec).utilization_worst;
                            r.joints
                                .iter()
                                .map(|j| JointChecks::of(j, &spec).utilization_worst)
                                .fold(bumper, f64::max)
                        })
                        .unwrap_or(utilization_worst);

                clusters.push(ClusterExport {
                    definition: cluster.clone(),
                    result,
                    joint_checks,
                    bumper_checks,
                    utilization_worst,
                    safety_factor: safety_from(utilization_worst, settings.safety_factor),
                    safety_factor_static: safety_from(static_worst, settings.safety_factor),
                });
            }
        }
    }

    AuditExport {
        schema_version: EXPORT_SCHEMA_VERSION,
        generated_at,
        settings: settings.clone(),
        definitions: referenced_definitions(selection, speakers, bumpers, bumper_bars),
        clusters,
        impossible,
    }
}

/// Coefficient de sécurité réel depuis un taux de travail. Le taux est rapporté
/// à `R_m / sf`, donc le facteur à la rupture est `sf / taux`.
fn safety_from(utilization: f64, safety_factor: f64) -> f64 {
    if utilization <= 0.0 {
        f64::INFINITY
    } else {
        safety_factor / utilization
    }
}

/// Pièces référencées par la sélection. Les grappes impossibles comptent aussi :
/// leurs modèles font partie de ce qu'il faut pour comprendre pourquoi elles
/// échouent.
fn referenced_definitions(
    selection: &[Cluster],
    speakers: &[SpeakerModel],
    bumpers: &[BumperModel],
    bumper_bars: &[BumperBarModel],
) -> Definitions {
    let used_speakers: Vec<&str> = selection
        .iter()
        .flat_map(|c| c.speaker_model_ids.iter())
        .map(String::as_str)
        .collect();
    let used_bumpers: Vec<&str> = selection
        .iter()
        .map(|c| c.bumper_model_id.as_str())
        .collect();

    let kept_bumpers: Vec<BumperModel> = bumpers
        .iter()
        .filter(|b| used_bumpers.contains(&b.id.as_str()))
        .cloned()
        .collect();

    Definitions {
        speakers: speakers
            .iter()
            .filter(|s| used_speakers.contains(&s.id.as_str()))
            .cloned()
            .collect(),
        // Une barre est retenue dès qu'elle est compatible avec un bumper de la
        // sélection : c'est le solveur qui décide ensuite laquelle sert
        // réellement, en fonction du déport calculé. Les exporter toutes, c'est
        // permettre au relecteur de refaire ce choix — n'en garder qu'une le
        // lui imposerait.
        bumper_bars: bumper_bars
            .iter()
            .filter(|bar| {
                bar.compatible_bumpers
                    .iter()
                    .any(|c| kept_bumpers.iter().any(|b| b.id == c.bumper_model_id))
            })
            .cloned()
            .collect(),
        bumpers: kept_bumpers,
    }
}

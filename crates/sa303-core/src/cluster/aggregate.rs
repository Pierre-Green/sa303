//! Agrégat des pires cas sur toutes les configurations (brief §9) : fait
//! tourner `super::solver::compute_cluster` pour chaque cluster, bucket les
//! joints obtenus par compartiment, et applique `worst_cases_selector` à
//! chaque compartiment séparément. Les grappes physiquement impossibles ne
//! sont jamais mélangées silencieusement au reste (brief §11.6).

use super::joint::JointResult;
use super::model::{Cluster, Compartment};
use super::solver::compute_cluster;
use super::worst_cases_selector::{select_block_a, select_block_b, LoadCase};
use crate::bumper::{BumperBarModel, BumperModel};
use crate::checks::{utilization, SandwichSpec};
use crate::settings::Settings;
use crate::speaker::SpeakerModel;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadCaseReport {
    pub cluster_name: String,
    pub joint_number: usize,
    /// Chemins de charge distincts satisfaits par ce cas (bloc A uniquement).
    pub labels: Vec<&'static str>,
    /// Bloc B seulement : même grappe, même joint qu'un cas déjà retenu au bloc A.
    pub duplicate: bool,
    pub result: JointResult,
    pub utilization_orientation: f64,
    pub utilization_pivot: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompartmentReport {
    pub block_a: Vec<LoadCaseReport>,
    pub block_b: Vec<LoadCaseReport>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpossibleClusterReport {
    pub cluster_name: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregateReport {
    pub flown: CompartmentReport,
    pub stacked: CompartmentReport,
    /// Grappes exclues de l'agrégat car leur configuration est physiquement
    /// impossible (brief §11.6).
    pub impossible_clusters: Vec<ImpossibleClusterReport>,
}

fn load_case_report(
    case: &LoadCase,
    labels: Vec<&'static str>,
    duplicate: bool,
    spec: &SandwichSpec,
) -> LoadCaseReport {
    LoadCaseReport {
        cluster_name: case.cluster_name.clone(),
        joint_number: case.joint_number,
        labels,
        duplicate,
        result: case.result,
        utilization_orientation: utilization(case.result.mag_orientation(), spec),
        utilization_pivot: utilization(case.result.mag_pivot(), spec),
    }
}

fn compartment_report(cases: &[LoadCase], spec: &SandwichSpec) -> CompartmentReport {
    let block_a = select_block_a(cases);
    let block_b = select_block_b(cases, &block_a);
    CompartmentReport {
        block_a: block_a
            .iter()
            .map(|b| load_case_report(&cases[b.case_index], b.labels.clone(), false, spec))
            .collect(),
        block_b: block_b
            .iter()
            .map(|b| load_case_report(&cases[b.case_index], Vec::new(), b.duplicate, spec))
            .collect(),
    }
}

/// Agrégat des pires cas sur toutes les configurations, stacks séparés des
/// grappes (brief §9). Chaque grappe référence une `SpeakerModel` par position
/// via `speaker_model_ids` (chaîne hétérogène, résolue contre `speakers`) et
/// son `BumperModel` obligatoire via `bumper_model_id` ;
/// la barre de déport, s'il y en a une, est dérivée automatiquement du bumper
/// actif parmi `bumper_bars`. Les grappes physiquement impossibles, y compris
/// une référence de bumper introuvable (brief §11.6), sont exclues des blocs
/// et listées à part, jamais mélangées silencieusement au reste.
pub fn compute_aggregate(
    speakers: &[SpeakerModel],
    clusters: &[Cluster],
    settings: &Settings,
    bumpers: &[BumperModel],
    bumper_bars: &[BumperBarModel],
) -> AggregateReport {
    let spec = SandwichSpec::from_settings(settings);
    let mut flown_cases: Vec<LoadCase> = Vec::new();
    let mut stacked_cases: Vec<LoadCase> = Vec::new();
    let mut impossible_clusters: Vec<ImpossibleClusterReport> = Vec::new();

    for cluster in clusters {
        // Un bumper est obligatoire (brief §11.6) : une référence qui ne
        // pointe vers rien est une configuration impossible à signaler,
        // jamais une panique qui ferait échouer tout l'agrégat pour une
        // seule grappe mal configurée.
        let Some(bumper_model) = bumpers.iter().find(|b| b.id == cluster.bumper_model_id) else {
            impossible_clusters.push(ImpossibleClusterReport {
                cluster_name: cluster.name.clone(),
                reason: format!(
                    "Bumper \"{}\" introuvable : un bumper est obligatoire pour chaque grappe.",
                    cluster.bumper_model_id
                ),
            });
            continue;
        };

        let result = match compute_cluster(speakers, cluster, settings, bumper_model, bumper_bars) {
            Ok(result) => result,
            Err(e) => {
                impossible_clusters.push(ImpossibleClusterReport {
                    cluster_name: cluster.name.clone(),
                    reason: e.reason,
                });
                continue;
            }
        };
        let bucket = match cluster.compartment {
            Compartment::Flown => &mut flown_cases,
            Compartment::Stacked => &mut stacked_cases,
        };
        for (i, joint) in result.joints.into_iter().enumerate() {
            bucket.push(LoadCase {
                cluster_name: cluster.name.clone(),
                joint_number: i + 1,
                result: joint,
            });
        }
    }

    AggregateReport {
        impossible_clusters,
        flown: compartment_report(&flown_cases, &spec),
        stacked: compartment_report(&stacked_cases, &spec),
    }
}

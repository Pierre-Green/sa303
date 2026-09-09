//! Commandes Tauri : fines, elles ne font qu'appeler `sa303-core` et la
//! persistance, puis sérialiser en JSON (brief §1).

use crate::persistence;
use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::Cluster;
use sa303_core::settings::Settings;
use sa303_core::speaker::{geometry_report, SpeakerGeometryReport, SpeakerModel};
use sa303_core::wst::{wst_report, WstInputs, WstReport};
use sa303_core::{compute_aggregate, compute_cluster, AggregateReport, ClusterResult};
use tauri::AppHandle;

#[tauri::command]
pub fn list_speaker_models(app: AppHandle) -> Result<Vec<SpeakerModel>, String> {
    persistence::list_speaker_models(&app)
}

#[tauri::command]
pub fn save_speaker_model(app: AppHandle, speaker_model: SpeakerModel) -> Result<(), String> {
    persistence::save_speaker_model(&app, &speaker_model)
}

#[tauri::command]
pub fn delete_speaker_model(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete_speaker_model(&app, &id)
}

#[tauri::command]
pub fn list_bumper_models(app: AppHandle) -> Result<Vec<BumperModel>, String> {
    persistence::list_bumper_models(&app)
}

#[tauri::command]
pub fn save_bumper_model(app: AppHandle, bumper_model: BumperModel) -> Result<(), String> {
    persistence::save_bumper_model(&app, &bumper_model)
}

#[tauri::command]
pub fn delete_bumper_model(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete_bumper_model(&app, &id)
}

#[tauri::command]
pub fn list_bumper_bar_models(app: AppHandle) -> Result<Vec<BumperBarModel>, String> {
    persistence::list_bumper_bar_models(&app)
}

#[tauri::command]
pub fn save_bumper_bar_model(
    app: AppHandle,
    bumper_bar_model: BumperBarModel,
) -> Result<(), String> {
    persistence::save_bumper_bar_model(&app, &bumper_bar_model)
}

#[tauri::command]
pub fn delete_bumper_bar_model(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete_bumper_bar_model(&app, &id)
}

#[tauri::command]
pub fn list_clusters(app: AppHandle) -> Result<Vec<Cluster>, String> {
    persistence::list_clusters(&app)
}

#[tauri::command]
pub fn save_cluster(app: AppHandle, cluster: Cluster) -> Result<(), String> {
    persistence::save_cluster(&app, &cluster)
}

#[tauri::command]
pub fn delete_cluster(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete_cluster(&app, &id)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    persistence::load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    persistence::save_settings(&app, &settings)
}

/// Prend la grappe en paramètre plutôt que son id : sert aussi bien à
/// prévisualiser une grappe pas encore enregistrée (édition en cours côté
/// front) qu'à recalculer une grappe sauvegardée.
#[tauri::command]
pub fn compute_cluster_result(app: AppHandle, cluster: Cluster) -> Result<ClusterResult, String> {
    // Une grappe est hétérogène : elle peut référencer plusieurs modèles
    // d'enceinte, donc on passe le catalogue complet plutôt qu'un modèle
    // unique — c'est le solveur qui résout chaque position.
    let speaker_models = persistence::list_speaker_models(&app)?;
    let settings = persistence::load_settings(&app)?;
    let bumper_model = persistence::load_bumper_model(&app, &cluster.bumper_model_id)?;
    let bumper_bars = persistence::list_bumper_bar_models(&app)?;

    compute_cluster(
        &speaker_models,
        &cluster,
        &settings,
        &bumper_model,
        &bumper_bars,
    )
    .map_err(|e| e.reason)
}

#[tauri::command]
pub fn get_speaker_geometry_report(
    app: AppHandle,
    speaker_model_id: String,
) -> Result<SpeakerGeometryReport, String> {
    let speaker_model = persistence::load_speaker_model(&app, &speaker_model_id)?;
    geometry_report(&speaker_model).map_err(|e| {
        format!(
            "incohérence géométrique au splay {}° : bras {:.1} mm vs recoupement {:.1} mm (écart {:.2} mm au-delà de la tolérance)",
            e.splay_deg, e.lever_mm, e.lever_check_mm, e.discrepancy_mm
        )
    })
}

/// Critères WST (brief : boîte à outils). `speaker_model_id` renseigné : le pas
/// entre centres acoustiques est dérivé de la géométrie réelle de l'enceinte,
/// angle par angle — le jour en façade s'ouvre avec l'inclinaison. Sinon, tout
/// vient des champs saisis.
#[tauri::command]
pub fn compute_wst_report(
    app: AppHandle,
    inputs: WstInputs,
    speaker_model_id: Option<String>,
) -> Result<WstReport, String> {
    let speaker_model = match &speaker_model_id {
        Some(id) => Some(persistence::load_speaker_model(&app, id)?),
        None => None,
    };
    Ok(wst_report(&inputs, speaker_model.as_ref()))
}

#[tauri::command]
pub fn compute_aggregate_report(app: AppHandle) -> Result<AggregateReport, String> {
    let speakers = persistence::list_speaker_models(&app)?;
    let clusters = persistence::list_clusters(&app)?;
    let settings = persistence::load_settings(&app)?;
    let bumpers = persistence::list_bumper_models(&app)?;
    let bumper_bars = persistence::list_bumper_bar_models(&app)?;
    Ok(compute_aggregate(
        &speakers,
        &clusters,
        &settings,
        &bumpers,
        &bumper_bars,
    ))
}

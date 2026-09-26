//! Commandes Tauri : fines, elles ne font qu'appeler `sa303-core` et la
//! persistance, puis sérialiser en JSON (brief §1).

use crate::persistence;
use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::Cluster;
use sa303_core::export::{build_audit_export, AuditExport};
use sa303_core::settings::Settings;
use sa303_core::speaker::{geometry_report, SpeakerGeometryReport, SpeakerModel};
use sa303_core::wst::{wst_report, WstInputs, WstReport};
use sa303_core::{compute_aggregate, compute_cluster, AggregateReport, ClusterResult};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Identifiants du catalogue livré avec le logiciel. L'interface s'en sert pour
/// interdire la modification et la suppression de ces éléments : la persistance
/// refuse déjà l'opération, mais mieux vaut ne pas proposer un bouton qui ne
/// peut qu'échouer.
#[tauri::command]
pub fn get_builtin_ids() -> persistence::BuiltinIds {
    persistence::builtin_ids()
}

#[tauri::command]
pub fn list_speaker_models(app: AppHandle) -> Result<Vec<SpeakerModel>, String> {
    persistence::list::<SpeakerModel>(&app)
}

#[tauri::command]
pub fn save_speaker_model(app: AppHandle, speaker_model: SpeakerModel) -> Result<(), String> {
    persistence::save(&app, &speaker_model)
}

#[tauri::command]
pub fn delete_speaker_model(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete::<SpeakerModel>(&app, &id)
}

#[tauri::command]
pub fn list_bumper_models(app: AppHandle) -> Result<Vec<BumperModel>, String> {
    persistence::list::<BumperModel>(&app)
}

#[tauri::command]
pub fn save_bumper_model(app: AppHandle, bumper_model: BumperModel) -> Result<(), String> {
    persistence::save(&app, &bumper_model)
}

#[tauri::command]
pub fn delete_bumper_model(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete::<BumperModel>(&app, &id)
}

#[tauri::command]
pub fn list_bumper_bar_models(app: AppHandle) -> Result<Vec<BumperBarModel>, String> {
    persistence::list::<BumperBarModel>(&app)
}

#[tauri::command]
pub fn save_bumper_bar_model(
    app: AppHandle,
    bumper_bar_model: BumperBarModel,
) -> Result<(), String> {
    persistence::save(&app, &bumper_bar_model)
}

#[tauri::command]
pub fn delete_bumper_bar_model(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete::<BumperBarModel>(&app, &id)
}

#[tauri::command]
pub fn list_clusters(app: AppHandle) -> Result<Vec<Cluster>, String> {
    persistence::list::<Cluster>(&app)
}

#[tauri::command]
pub fn save_cluster(app: AppHandle, cluster: Cluster) -> Result<(), String> {
    persistence::save(&app, &cluster)
}

#[tauri::command]
pub fn delete_cluster(app: AppHandle, id: String) -> Result<(), String> {
    persistence::delete::<Cluster>(&app, &id)
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
    let speaker_models = persistence::list::<SpeakerModel>(&app)?;
    let settings = persistence::load_settings(&app)?;
    let bumper_model = persistence::load::<BumperModel>(&app, &cluster.bumper_model_id)?;
    let bumper_bars = persistence::list::<BumperBarModel>(&app)?;

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
    let speaker_model = persistence::load::<SpeakerModel>(&app, &speaker_model_id)?;
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
        Some(id) => Some(persistence::load::<SpeakerModel>(&app, id)?),
        None => None,
    };
    Ok(wst_report(&inputs, speaker_model.as_ref()))
}

#[tauri::command]
pub fn compute_aggregate_report(app: AppHandle) -> Result<AggregateReport, String> {
    let speakers = persistence::list::<SpeakerModel>(&app)?;
    let clusters = persistence::list::<Cluster>(&app)?;
    let settings = persistence::load_settings(&app)?;
    let bumpers = persistence::list::<BumperModel>(&app)?;
    let bumper_bars = persistence::list::<BumperBarModel>(&app)?;
    Ok(compute_aggregate(
        &speakers,
        &clusters,
        &settings,
        &bumpers,
        &bumper_bars,
    ))
}

/// Export d'audit : un JSON autoportant décrivant une sélection de grappes.
///
/// `cluster_ids` absent ou vide = toutes les grappes enregistrées. Sinon, la
/// sélection, dans l'ordre où le front l'a donnée — c'est l'ordre d'affichage,
/// donc celui que le relecteur retrouvera à l'écran.
///
/// `generated_at` vient du front plutôt que d'une horloge côté Rust : le cœur
/// de calcul n'a pas de dépendance à une bibliothèque de dates, et lui en
/// ajouter une pour un champ d'en-tête serait cher payé.
#[tauri::command]
pub fn build_cluster_audit_export(
    app: AppHandle,
    cluster_ids: Option<Vec<String>>,
    generated_at: String,
) -> Result<AuditExport, String> {
    let all = persistence::list::<Cluster>(&app)?;
    let selection: Vec<Cluster> = match cluster_ids {
        Some(ids) if !ids.is_empty() => {
            // Résolus dans l'ordre demandé, et une absence est une erreur :
            // exporter en silence une sélection amputée donnerait un document
            // qui ne correspond à rien de ce que l'utilisateur a vu.
            ids.iter()
                .map(|id| {
                    all.iter()
                        .find(|c| &c.id == id)
                        .cloned()
                        .ok_or_else(|| format!("grappe « {id} » introuvable"))
                })
                .collect::<Result<_, _>>()?
        }
        _ => all,
    };
    if selection.is_empty() {
        return Err("aucune grappe à exporter".into());
    }

    Ok(build_audit_export(
        generated_at,
        &selection,
        &persistence::list::<SpeakerModel>(&app)?,
        &persistence::list::<BumperModel>(&app)?,
        &persistence::list::<BumperBarModel>(&app)?,
        &persistence::load_settings(&app)?,
    ))
}

/// Même export, écrit sur disque via la boîte de dialogue système.
///
/// Rend le chemin retenu, ou `None` si l'utilisateur a annulé — annuler n'est
/// pas une erreur, et remonter un `Err` ferait afficher un bandeau rouge pour
/// un geste délibéré.
#[tauri::command]
pub async fn export_clusters_for_audit(
    app: AppHandle,
    cluster_ids: Option<Vec<String>>,
    generated_at: String,
    suggested_file_name: String,
) -> Result<Option<String>, String> {
    let export = build_cluster_audit_export(app.clone(), cluster_ids, generated_at)?;
    // Indenté : ce fichier est fait pour être lu et commenté par un tiers, pas
    // seulement reparsé.
    let json = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;

    let Some(file) = app
        .dialog()
        .file()
        .set_title("Exporter les grappes pour audit")
        .set_file_name(&suggested_file_name)
        .add_filter("JSON", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("écriture impossible : {e}"))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn export_aggregate_report_for_shape_optimization_fem(
    app: AppHandle,
) -> Result<(), String> {
    let report = compute_aggregate_report(app.clone())?;
    // Implementation for exporting the report

    let file_path = app
        .dialog()
        .file()
        .set_title("Sauvegarder le json pour le systeme de shape optimization")
        .add_filter("JSON", &["json"])
        .blocking_save_file();

    Ok(())
}

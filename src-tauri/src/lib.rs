mod commands;
mod persistence;
mod seed;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Sans cette ligne, `app.dialog()` panique : le plugin est déclaré en
        // dépendance mais son état n'est enregistré qu'ici.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            persistence::ensure_seeded(&app.handle().clone())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_builtin_ids,
            commands::list_speaker_models,
            commands::save_speaker_model,
            commands::delete_speaker_model,
            commands::list_bumper_models,
            commands::save_bumper_model,
            commands::delete_bumper_model,
            commands::list_bumper_bar_models,
            commands::save_bumper_bar_model,
            commands::delete_bumper_bar_model,
            commands::list_clusters,
            commands::save_cluster,
            commands::delete_cluster,
            commands::get_settings,
            commands::save_settings,
            commands::compute_cluster_result,
            commands::get_speaker_geometry_report,
            commands::compute_aggregate_report,
            commands::compute_wst_report,
            commands::export_aggregate_report_for_shape_optimization_fem
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

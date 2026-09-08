mod commands;
mod rename;
mod tags;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::read_tracks_cmd,
            commands::read_track_details_cmd,
            commands::write_tags_cmd,
            commands::titles_from_filenames_cmd,
            commands::set_field_bulk_cmd,
            commands::plan_renames_cmd,
            commands::apply_renames_cmd,
            commands::rename_file_stem_cmd,
            commands::scan_folder_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

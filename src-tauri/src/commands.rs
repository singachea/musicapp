use crate::rename::{
    apply_renames, plan_renames_from_titles, rename_file_stem, RenameApplyResult, RenamePlan,
};
use crate::tags::{
    read_track_details, read_tracks, scan_folder, set_field_bulk, titles_from_filenames,
    write_tags, OpResult, TagUpdate, TrackDetails, TrackInfo,
};

#[tauri::command]
pub fn read_tracks_cmd(paths: Vec<String>) -> Vec<TrackInfo> {
    read_tracks(paths)
}

#[tauri::command]
pub fn read_track_details_cmd(path: String) -> TrackDetails {
    read_track_details(&path)
}

#[tauri::command]
pub fn write_tags_cmd(updates: Vec<TagUpdate>) -> Vec<OpResult> {
    write_tags(updates)
}

#[tauri::command]
pub fn titles_from_filenames_cmd(paths: Vec<String>) -> Vec<OpResult> {
    titles_from_filenames(paths)
}

#[tauri::command]
pub fn set_field_bulk_cmd(
    paths: Vec<String>,
    album: Option<String>,
    artist: Option<String>,
) -> Vec<OpResult> {
    set_field_bulk(paths, album, artist)
}

#[tauri::command]
pub fn plan_renames_cmd(paths: Vec<String>) -> RenamePlan {
    plan_renames_from_titles(paths)
}

#[tauri::command]
pub fn apply_renames_cmd(plan: RenamePlan) -> Vec<RenameApplyResult> {
    apply_renames(plan)
}

#[tauri::command]
pub fn rename_file_stem_cmd(path: String, new_stem: String) -> RenameApplyResult {
    rename_file_stem(&path, &new_stem)
}

#[tauri::command]
pub fn scan_folder_cmd(path: String, recursive: bool) -> Result<Vec<String>, String> {
    scan_folder(&path, recursive)
}

import { invoke } from "@tauri-apps/api/core";
import type {
  OpResult,
  RenameApplyResult,
  RenamePlan,
  TagUpdate,
  TrackDetails,
  TrackInfo,
} from "./types";

export function readTracks(paths: string[]): Promise<TrackInfo[]> {
  return invoke("read_tracks_cmd", { paths });
}

export function readTrackDetails(path: string): Promise<TrackDetails> {
  return invoke("read_track_details_cmd", { path });
}

export function writeTags(updates: TagUpdate[]): Promise<OpResult[]> {
  return invoke("write_tags_cmd", { updates });
}

export function titlesFromFilenames(paths: string[]): Promise<OpResult[]> {
  return invoke("titles_from_filenames_cmd", { paths });
}

export function setFieldBulk(
  paths: string[],
  album?: string | null,
  artist?: string | null,
): Promise<OpResult[]> {
  return invoke("set_field_bulk_cmd", { paths, album, artist });
}

export function planRenames(paths: string[]): Promise<RenamePlan> {
  return invoke("plan_renames_cmd", { paths });
}

export function applyRenames(plan: RenamePlan): Promise<RenameApplyResult[]> {
  return invoke("apply_renames_cmd", { plan });
}

export function renameFileStem(
  path: string,
  newStem: string,
): Promise<RenameApplyResult> {
  return invoke("rename_file_stem_cmd", { path, newStem });
}

export function scanFolder(path: string, recursive: boolean): Promise<string[]> {
  return invoke("scan_folder_cmd", { path, recursive });
}

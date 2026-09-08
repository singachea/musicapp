use crate::tags::{read_track, TrackInfo};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameItem {
    pub from: String,
    pub to: String,
    pub title: String,
    pub collision: bool,
    pub skipped: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamePlan {
    pub items: Vec<RenameItem>,
    pub collision_count: usize,
    pub applyable_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameApplyResult {
    pub from: String,
    pub to: Option<String>,
    pub ok: bool,
    pub error: Option<String>,
    pub track: Option<TrackInfo>,
}

/// Characters illegal on macOS/Windows filesystems or awkward in names.
pub fn sanitize_filename(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => out.push('_'),
            c if c.is_control() => out.push('_'),
            c => out.push(c),
        }
    }
    let trimmed = out.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn plan_renames_from_titles(paths: Vec<String>) -> RenamePlan {
    let mut items = Vec::with_capacity(paths.len());
    let mut target_counts: HashMap<String, usize> = HashMap::new();

    // First pass: compute intended destinations
    let mut draft: Vec<(String, String, String, Option<String>)> = Vec::new();
    for path in paths {
        let track = read_track(&path);
        let title = track.title.trim().to_string();
        if title.is_empty() {
            draft.push((
                path,
                String::new(),
                String::new(),
                Some("Empty title — cannot rename".into()),
            ));
            continue;
        }

        let src = PathBuf::from(&path);
        let parent = src.parent().unwrap_or_else(|| Path::new("."));
        let ext = src
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let safe = sanitize_filename(&title);
        let dest = parent.join(format!("{safe}{ext}"));
        let dest_str = dest.to_string_lossy().to_string();

        // Same path = no-op
        if dest_str == path {
            draft.push((
                path,
                dest_str,
                title,
                Some("Already named correctly".into()),
            ));
            continue;
        }

        *target_counts.entry(dest_str.clone()).or_insert(0) += 1;
        draft.push((path, dest_str, title, None));
    }

    // Paths that already exist on disk (and aren't the source of this rename)
    let existing: HashSet<String> = draft
        .iter()
        .filter_map(|(from, to, _, reason)| {
            if reason.is_some() || to.is_empty() {
                return None;
            }
            let p = Path::new(to);
            if p.exists() && to != from {
                Some(to.clone())
            } else {
                None
            }
        })
        .collect();

    let mut collision_count = 0usize;
    let mut applyable_count = 0usize;

    for (from, to, title, reason) in draft {
        if let Some(reason) = reason {
            let skipped = true;
            items.push(RenameItem {
                from,
                to,
                title,
                collision: false,
                skipped,
                reason: Some(reason),
            });
            continue;
        }

        let multi_target = target_counts.get(&to).copied().unwrap_or(0) > 1;
        let disk_collision = existing.contains(&to);
        let collision = multi_target || disk_collision;

        let (skipped, reason) = if collision {
            collision_count += 1;
            (
                true,
                Some(if multi_target {
                    "Multiple tracks would rename to the same path".into()
                } else {
                    "Destination already exists".into()
                }),
            )
        } else {
            applyable_count += 1;
            (false, None)
        };

        items.push(RenameItem {
            from,
            to,
            title,
            collision,
            skipped,
            reason,
        });
    }

    RenamePlan {
        items,
        collision_count,
        applyable_count,
    }
}

/// Rename a single file's stem (base name), keeping the original extension.
pub fn rename_file_stem(path: &str, new_stem: &str) -> RenameApplyResult {
    let src = PathBuf::from(path);
    if !src.exists() {
        return RenameApplyResult {
            from: path.to_string(),
            to: None,
            ok: false,
            error: Some("File not found".into()),
            track: None,
        };
    }

    let safe = sanitize_filename(new_stem);
    if safe.is_empty() {
        return RenameApplyResult {
            from: path.to_string(),
            to: None,
            ok: false,
            error: Some("Filename cannot be empty".into()),
            track: None,
        };
    }

    let parent = src.parent().unwrap_or_else(|| Path::new("."));
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let dest = parent.join(format!("{safe}{ext}"));
    let dest_str = dest.to_string_lossy().to_string();

    if dest_str == path {
        return RenameApplyResult {
            from: path.to_string(),
            to: Some(dest_str),
            ok: true,
            error: None,
            track: Some(read_track(path)),
        };
    }

    if dest.exists() {
        return RenameApplyResult {
            from: path.to_string(),
            to: Some(dest_str),
            ok: false,
            error: Some("A file with that name already exists".into()),
            track: None,
        };
    }

    match std::fs::rename(&src, &dest) {
        Ok(()) => RenameApplyResult {
            from: path.to_string(),
            to: Some(dest_str.clone()),
            ok: true,
            error: None,
            track: Some(read_track(&dest_str)),
        },
        Err(e) => RenameApplyResult {
            from: path.to_string(),
            to: Some(dest_str),
            ok: false,
            error: Some(e.to_string()),
            track: None,
        },
    }
}

pub fn apply_renames(plan: RenamePlan) -> Vec<RenameApplyResult> {
    let mut results = Vec::with_capacity(plan.items.len());

    for item in plan.items {
        if item.skipped || item.collision || item.to.is_empty() {
            results.push(RenameApplyResult {
                from: item.from,
                to: if item.to.is_empty() {
                    None
                } else {
                    Some(item.to)
                },
                ok: false,
                error: item
                    .reason
                    .or_else(|| Some("Skipped".into())),
                track: None,
            });
            continue;
        }

        let from = PathBuf::from(&item.from);
        let to = PathBuf::from(&item.to);

        if to.exists() {
            results.push(RenameApplyResult {
                from: item.from,
                to: Some(item.to),
                ok: false,
                error: Some("Destination already exists".into()),
                track: None,
            });
            continue;
        }

        match std::fs::rename(&from, &to) {
            Ok(()) => {
                let new_path = item.to.clone();
                results.push(RenameApplyResult {
                    from: item.from,
                    to: Some(new_path.clone()),
                    ok: true,
                    error: None,
                    track: Some(read_track(&new_path)),
                });
            }
            Err(e) => {
                results.push(RenameApplyResult {
                    from: item.from,
                    to: Some(item.to),
                    ok: false,
                    error: Some(e.to_string()),
                    track: None,
                });
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_illegal_chars() {
        assert_eq!(sanitize_filename("a/b:c*d?"), "a_b_c_d_");
        assert_eq!(sanitize_filename("   "), "untitled");
        assert_eq!(sanitize_filename("hello.world"), "hello.world");
    }
}

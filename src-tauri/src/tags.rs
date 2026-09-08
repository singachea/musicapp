use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, FileType, TaggedFileExt};
use lofty::picture::{MimeType, PictureType};
use lofty::probe::Probe;
use lofty::read_from_path;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "m4a", "m4b", "aac", "flac", "ogg", "oga", "opus", "wav", "aiff", "aif", "wma",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub path: String,
    pub filename: String,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub format: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdate {
    pub path: String,
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpResult {
    pub path: String,
    pub ok: bool,
    pub error: Option<String>,
    pub track: Option<TrackInfo>,
}

/// Full metadata for the detail panel (loaded on demand).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackDetails {
    pub path: String,
    pub filename: String,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub album_artist: String,
    pub genre: String,
    pub comment: String,
    pub year: String,
    pub date: String,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    pub format: String,
    pub duration_secs: f64,
    pub duration_label: String,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub bit_depth: Option<u8>,
    pub file_size_bytes: u64,
    pub file_size_label: String,
    pub cover_data_url: Option<String>,
    pub cover_type: Option<String>,
    pub error: Option<String>,
}

pub fn is_audio_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.iter().any(|ext| e.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

pub fn scan_folder(folder: &str, recursive: bool) -> Result<Vec<String>, String> {
    let root = PathBuf::from(folder);
    if !root.is_dir() {
        return Err(format!("Not a directory: {folder}"));
    }

    let mut paths = Vec::new();
    collect_audio_files(&root, recursive, &mut paths)?;
    paths.sort();
    Ok(paths
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

fn collect_audio_files(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("Cannot read {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_audio_files(&path, true, out)?;
            }
        } else if is_audio_path(&path) {
            out.push(path);
        }
    }
    Ok(())
}

pub fn read_tracks(paths: Vec<String>) -> Vec<TrackInfo> {
    paths.into_iter().map(|p| read_track(&p)).collect()
}

pub fn read_track(path: &str) -> TrackInfo {
    let path_buf = PathBuf::from(path);
    let filename = path_buf
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    let format = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match read_track_inner(&path_buf) {
        Ok((title, album, artist, format_label)) => TrackInfo {
            path: path.to_string(),
            filename,
            title,
            album,
            artist,
            format: if format_label.is_empty() {
                format
            } else {
                format_label
            },
            error: None,
        },
        Err(err) => TrackInfo {
            path: path.to_string(),
            filename,
            title: String::new(),
            album: String::new(),
            artist: String::new(),
            format,
            error: Some(err),
        },
    }
}

fn file_type_label(ft: FileType) -> String {
    match ft {
        FileType::Mpeg => "mp3".into(),
        FileType::Mp4 => "m4a".into(),
        FileType::Flac => "flac".into(),
        FileType::Opus => "opus".into(),
        FileType::Vorbis => "ogg".into(),
        FileType::Wav => "wav".into(),
        FileType::Aiff => "aiff".into(),
        other => format!("{other:?}").to_lowercase(),
    }
}

fn read_track_inner(path: &Path) -> Result<(String, String, String, String), String> {
    let tagged = read_from_path(path).map_err(|e| e.to_string())?;
    let format = file_type_label(tagged.file_type());

    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let (title, album, artist) = match tag {
        Some(t) => (
            t.title().map(|c| c.to_string()).unwrap_or_default(),
            t.album().map(|c| c.to_string()).unwrap_or_default(),
            t.artist().map(|c| c.to_string()).unwrap_or_default(),
        ),
        None => (String::new(), String::new(), String::new()),
    };

    // Touch properties so we confirm file is parseable as audio
    let _ = tagged.properties();

    Ok((title, album, artist, format))
}

fn format_duration(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "—".into();
    }
    let total = secs.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    let b = bytes as f64;
    if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

fn mime_to_str(mime: Option<&MimeType>) -> &'static str {
    match mime {
        Some(MimeType::Jpeg) => "image/jpeg",
        Some(MimeType::Png) => "image/png",
        Some(MimeType::Gif) => "image/gif",
        Some(MimeType::Bmp) => "image/bmp",
        Some(MimeType::Tiff) => "image/tiff",
        _ => "image/jpeg",
    }
}

fn picture_type_label(t: PictureType) -> String {
    format!("{t:?}")
}

pub fn read_track_details(path: &str) -> TrackDetails {
    let path_buf = PathBuf::from(path);
    let filename = path_buf
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    let file_size_bytes = std::fs::metadata(&path_buf)
        .map(|m| m.len())
        .unwrap_or(0);

    let format_guess = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let empty = || TrackDetails {
        path: path.to_string(),
        filename: filename.clone(),
        title: String::new(),
        album: String::new(),
        artist: String::new(),
        album_artist: String::new(),
        genre: String::new(),
        comment: String::new(),
        year: String::new(),
        date: String::new(),
        track_number: None,
        track_total: None,
        disc_number: None,
        disc_total: None,
        format: format_guess.clone(),
        duration_secs: 0.0,
        duration_label: "—".into(),
        bitrate_kbps: None,
        sample_rate_hz: None,
        channels: None,
        bit_depth: None,
        file_size_bytes,
        file_size_label: format_file_size(file_size_bytes),
        cover_data_url: None,
        cover_type: None,
        error: None,
    };

    let tagged = match read_from_path(&path_buf) {
        Ok(t) => t,
        Err(e) => {
            let mut d = empty();
            d.error = Some(e.to_string());
            return d;
        }
    };

    let props = tagged.properties();
    let duration_secs = props.duration().as_secs_f64();
    let format = file_type_label(tagged.file_type());

    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());

    let mut details = empty();
    details.format = format;
    details.duration_secs = duration_secs;
    details.duration_label = format_duration(duration_secs);
    details.bitrate_kbps = props.audio_bitrate().or(props.overall_bitrate());
    details.sample_rate_hz = props.sample_rate();
    details.channels = props.channels();
    details.bit_depth = props.bit_depth();

    if let Some(t) = tag {
        details.title = t.title().map(|c| c.to_string()).unwrap_or_default();
        details.album = t.album().map(|c| c.to_string()).unwrap_or_default();
        details.artist = t.artist().map(|c| c.to_string()).unwrap_or_default();
        details.genre = t.genre().map(|c| c.to_string()).unwrap_or_default();
        details.comment = t.comment().map(|c| c.to_string()).unwrap_or_default();
        details.track_number = t.track();
        details.track_total = t.track_total();
        details.disc_number = t.disk();
        details.disc_total = t.disk_total();
        details.album_artist = t
            .get_string(ItemKey::AlbumArtist)
            .unwrap_or_default()
            .to_string();

        if let Some(ts) = t.date() {
            details.date = ts.to_string();
            details.year = ts.year.to_string();
        } else if let Some(y) = t.get_string(ItemKey::Year) {
            details.year = y.to_string();
            details.date = y.to_string();
        }

        // Prefer front cover, else first picture
        let pictures = t.pictures();
        let pic = pictures
            .iter()
            .find(|p| p.pic_type() == PictureType::CoverFront)
            .or_else(|| pictures.first());

        if let Some(pic) = pic {
            let mime = mime_to_str(pic.mime_type());
            let data = pic.data();
            if !data.is_empty() {
                // Cap huge covers (~2MB) for IPC
                let slice = if data.len() > 2_000_000 {
                    &data[..2_000_000]
                } else {
                    data
                };
                details.cover_data_url = Some(format!(
                    "data:{mime};base64,{}",
                    B64.encode(slice)
                ));
                details.cover_type = Some(picture_type_label(pic.pic_type()));
            }
        }
    }

    details
}

pub fn write_tags(updates: Vec<TagUpdate>) -> Vec<OpResult> {
    updates.into_iter().map(write_one).collect()
}

fn write_one(update: TagUpdate) -> OpResult {
    let path = update.path.clone();
    match apply_tag_update(&update) {
        Ok(()) => OpResult {
            path: path.clone(),
            ok: true,
            error: None,
            track: Some(read_track(&path)),
        },
        Err(error) => OpResult {
            path,
            ok: false,
            error: Some(error),
            track: None,
        },
    }
}

fn apply_tag_update(update: &TagUpdate) -> Result<(), String> {
    let path = Path::new(&update.path);
    let mut tagged = Probe::open(path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    if tagged.primary_tag().is_none() {
        let tag_type = tagged.primary_tag_type();
        tagged.insert_tag(Tag::new(tag_type));
    }

    let tag = tagged
        .primary_tag_mut()
        .ok_or_else(|| "Could not create or open a primary tag".to_string())?;

    if let Some(ref title) = update.title {
        if title.is_empty() {
            tag.remove_title();
        } else {
            tag.set_title(title.clone());
        }
    }
    if let Some(ref album) = update.album {
        if album.is_empty() {
            tag.remove_album();
        } else {
            tag.set_album(album.clone());
        }
    }
    if let Some(ref artist) = update.artist {
        if artist.is_empty() {
            tag.remove_artist();
        } else {
            tag.set_artist(artist.clone());
        }
    }

    tag.save_to_path(path, WriteOptions::default())
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn titles_from_filenames(paths: Vec<String>) -> Vec<OpResult> {
    let updates: Vec<TagUpdate> = paths
        .into_iter()
        .map(|path| {
            let title = Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            TagUpdate {
                path,
                title: Some(title),
                album: None,
                artist: None,
            }
        })
        .collect();
    write_tags(updates)
}

pub fn set_field_bulk(
    paths: Vec<String>,
    album: Option<String>,
    artist: Option<String>,
) -> Vec<OpResult> {
    let updates: Vec<TagUpdate> = paths
        .into_iter()
        .map(|path| TagUpdate {
            path,
            title: None,
            album: album.clone(),
            artist: artist.clone(),
        })
        .collect();
    write_tags(updates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_audio_extensions() {
        assert!(is_audio_path(Path::new("/a/b/c.mp3")));
        assert!(is_audio_path(Path::new("/a/b/c.M4A")));
        assert!(!is_audio_path(Path::new("/a/b/c.txt")));
    }
}

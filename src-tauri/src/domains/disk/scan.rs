use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use jwalk::WalkDir;
use serde::Serialize;

/// A single file discovered during a scan.
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified_secs: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawScan {
    pub total_bytes: u64,
    pub scanned_files: u64,
    /// Files counted in pass 1 — the denominator for a partial snapshot's pct.
    pub total_files: u64,
    /// True if the walk stopped early because the caller requested cancellation.
    pub cancelled: bool,
    #[serde(skip)]
    pub files: Vec<FileEntry>,
}

/// Live progress emitted during a walk so the UI can show a bar + ETA.
/// `counting` = fast first pass (total unknown, bar indeterminate);
/// `scanning`  = metadata pass (percent + ETA are meaningful).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub phase: &'static str,
    pub scanned_files: u64,
    pub total_files: u64,
    pub total_bytes: u64,
    pub current_path: String,
    pub eta_ms: u64,
    pub elapsed_ms: u128,
}

fn modified_secs(meta: &std::fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Emit at most ~6×/sec to keep the IPC channel from flooding the UI.
const EMIT_EVERY_MS: u128 = 150;

/// Persist a partial snapshot at most this often. Bounds how much progress a
/// crash / app-close can lose while keeping classify+DB writes cheap.
const CHECKPOINT_EVERY_MS: u128 = 2500;

/// Walks `root` in parallel and collects every file with its size and mtime.
/// Symlinks are not followed to avoid escaping the scanned subtree.
/// The walk checks `cancel` per entry and stops early when it is set.
///
/// Runs in two passes so the caller can render a real ETA:
///   1. a cheap count pass (no metadata) establishes the total file count;
///   2. the metadata pass reads sizes/mtimes and reports percent + ETA.
/// `on_progress` is invoked throttled to ~6×/sec, plus once at the end.
///
/// `on_checkpoint` is invoked at most every `CHECKPOINT_EVERY_MS` during the
/// metadata pass with the files gathered so far, so the caller can persist a
/// partial snapshot that survives an interrupt / crash / app close. On
/// cancellation the walk returns the partial file list with `cancelled = true`
/// (rather than discarding it) so the caller can save a final checkpoint.
pub fn walk<F, C>(
    root: &str,
    cancel: &AtomicBool,
    mut on_progress: F,
    mut on_checkpoint: C,
) -> RawScan
where
    F: FnMut(Progress),
    C: FnMut(&[FileEntry], u64, u64, u128),
{
    // ---- Pass 1: count files (no metadata read — cheap). ----
    let count_started = Instant::now();
    let mut total_files = 0u64;
    let mut last_emit = Instant::now();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .skip_hidden(false)
        .into_iter()
        .flatten()
    {
        if cancel.load(Ordering::Relaxed) {
            // No metadata gathered yet in the count pass — nothing to snapshot.
            return cancelled_scan();
        }
        if entry.file_type().is_file() {
            total_files += 1;
        }
        if last_emit.elapsed().as_millis() >= EMIT_EVERY_MS {
            last_emit = Instant::now();
            on_progress(Progress {
                phase: "counting",
                scanned_files: total_files,
                total_files: 0,
                total_bytes: 0,
                current_path: entry.path().to_string_lossy().into_owned(),
                eta_ms: 0,
                elapsed_ms: count_started.elapsed().as_millis(),
            });
        }
    }

    // ---- Pass 2: read metadata, report percent + ETA. ----
    let mut files = Vec::with_capacity(total_files as usize);
    let mut total_bytes = 0u64;
    let scan_started = Instant::now();
    last_emit = Instant::now();
    let mut last_checkpoint = Instant::now();
    let mut cancelled = false;

    for entry in WalkDir::new(root)
        .follow_links(false)
        .skip_hidden(false)
        .into_iter()
        .flatten()
    {
        if cancel.load(Ordering::Relaxed) {
            // Keep what we have — the caller saves it as a restorable snapshot.
            cancelled = true;
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let size = meta.len();
        total_bytes += size;
        let path = entry.path();
        let done = files.len() as u64 + 1;

        if last_emit.elapsed().as_millis() >= EMIT_EVERY_MS {
            last_emit = Instant::now();
            let elapsed = scan_started.elapsed().as_millis();
            // Linear extrapolation from current throughput. total_files can only
            // grow stale (files appearing mid-scan), so clamp remaining at 0.
            let eta_ms = if done > 0 && total_files >= done {
                (elapsed as u64).saturating_mul(total_files - done) / done
            } else {
                0
            };
            on_progress(Progress {
                phase: "scanning",
                scanned_files: done,
                total_files,
                total_bytes,
                current_path: path.to_string_lossy().into_owned(),
                eta_ms,
                elapsed_ms: elapsed,
            });
        }

        files.push(FileEntry {
            path,
            size,
            modified_secs: modified_secs(&meta),
        });

        if last_checkpoint.elapsed().as_millis() >= CHECKPOINT_EVERY_MS {
            last_checkpoint = Instant::now();
            on_checkpoint(&files, total_files, total_bytes, scan_started.elapsed().as_millis());
        }
    }

    RawScan {
        scanned_files: files.len() as u64,
        total_bytes,
        total_files,
        cancelled,
        files,
    }
}

fn cancelled_scan() -> RawScan {
    RawScan {
        total_bytes: 0,
        scanned_files: 0,
        total_files: 0,
        cancelled: true,
        files: Vec::new(),
    }
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

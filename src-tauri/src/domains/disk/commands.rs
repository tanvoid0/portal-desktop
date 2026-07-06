//! Tauri command surface for the disk-cleanup domain. Ported from the standalone
//! `portal_disk_utility` app's `lib.rs`. State (`Db`, `ScanControl`,
//! `VerifyControl`) is managed in the desktop `lib.rs` setup; see
//! `docs/development/DISK_UTILITY_MIGRATION.md`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::domains::disk::classify::{self, Proposal};
use crate::domains::disk::db::{AuditEntry, Db};
use crate::domains::disk::disk::{self, DiskUsage};
use crate::domains::disk::locations::{self, Location};
use crate::domains::disk::projects::{self, ProjectScan};
use crate::domains::disk::quarantine::{self, QuarantineItem, QuarantineResult};
use crate::domains::disk::scan;
use crate::domains::disk::verify::{self, AiConfig, TeamOption, VerificationResult, VerifyProgress, VerifyTask};
use crate::domains::disk::verify_ai;
use crate::domains::ai::services::AIService;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    root: String,
    total_bytes: u64,
    scanned_files: u64,
    elapsed_ms: u128,
    proposals: Vec<Proposal>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedScan {
    ts: i64,
    /// "complete" = a finished scan; "partial" = an interrupted one that can be
    /// resumed (re-scanned) or loaded as-is.
    status: String,
    scanned_files: i64,
    total_files: i64,
    summary: ScanSummary,
}

/// Shared cancellation flag for the in-flight scan. Set by `cancel_scan`,
/// polled by the walker, and reset at the start of every new scan.
#[derive(Default)]
pub struct ScanControl {
    cancel: Arc<AtomicBool>,
}

/// Shared cancellation flag for the in-flight AI verification. Set by
/// `cancel_verify`, polled by the verify poll loop, reset at each run's start.
#[derive(Default)]
pub struct VerifyControl {
    cancel: Arc<AtomicBool>,
}

/// Read-only: walks the tree and returns cleanup proposals. Deletes nothing.
/// Custom protected paths are applied, and the result is cached for instant reload.
#[tauri::command]
pub async fn scan_directory(
    root: String,
    app: AppHandle,
    db: State<'_, Arc<Db>>,
    control: State<'_, ScanControl>,
) -> Result<ScanSummary, String> {
    let started = Instant::now();
    let extra_protected = db.list_protected()?;
    let root_clone = root.clone();
    // Handles moved into the blocking walk so it can checkpoint partial results
    // straight to the DB (surviving a crash / app close, not just a Stop click).
    let db_ck = db.inner().clone();
    let ck_root = root.clone();
    let ck_protected = extra_protected.clone();
    // Fresh flag for this run — clears any leftover request from a prior scan.
    let cancel = control.cancel.clone();
    cancel.store(false, Ordering::Relaxed);
    // Heavy IO off the main thread.
    let summary = tauri::async_runtime::spawn_blocking(move || {
        // Forward throttled walk progress to the UI. Fire-and-forget: a dropped
        // event just means one skipped frame, never a failed scan.
        let raw = scan::walk(
            &root_clone,
            &cancel,
            |p| {
                let _ = app.emit("scan://progress", p);
            },
            |files, total_files, _bytes, elapsed_ms| {
                // Classify what we have so far and persist it as a partial
                // snapshot the restore card can offer on the next run.
                let proposals = classify::classify(&ck_root, files, &ck_protected);
                let partial = ScanSummary {
                    root: ck_root.clone(),
                    total_bytes: files.iter().map(|f| f.size).sum(),
                    scanned_files: files.len() as u64,
                    elapsed_ms,
                    proposals,
                };
                if let Ok(json) = serde_json::to_string(&partial) {
                    let _ = db_ck.cache_partial(
                        &ck_root,
                        &json,
                        files.len() as u64,
                        total_files,
                        elapsed_ms,
                    );
                }
            },
        );
        let proposals = classify::classify(&root_clone, &raw.files, &extra_protected);
        (
            raw.total_bytes,
            raw.scanned_files,
            raw.total_files,
            raw.cancelled,
            proposals,
        )
    })
    .await
    .map_err(|e| e.to_string())?;

    let (total_bytes, scanned_files, total_files, cancelled, proposals) = summary;
    let out = ScanSummary {
        root: root.clone(),
        total_bytes,
        scanned_files,
        elapsed_ms: started.elapsed().as_millis(),
        proposals,
    };

    if let Ok(json) = serde_json::to_string(&out) {
        if cancelled {
            // Final partial snapshot keeps the real pass-1 total so the restore
            // card can show how far the interrupted scan actually got.
            let _ = db.cache_partial(&root, &json, scanned_files, total_files, out.elapsed_ms);
        } else {
            let _ = db.cache_scan(&root, &json, scanned_files, scanned_files, out.elapsed_ms);
        }
    }
    if cancelled {
        return Err("cancelled".to_string());
    }
    Ok(out)
}

/// Read-only project-aware scan: detects project roots by their marker files
/// (package.json, pom.xml, Cargo.toml, …) and groups each project's regenerable
/// temp dirs (node_modules, target, build, .venv, …). Deletes nothing; reuses
/// the same walk + progress event as `scan_directory`.
#[tauri::command]
pub async fn scan_projects(
    root: String,
    app: AppHandle,
    db: State<'_, Arc<Db>>,
    control: State<'_, ScanControl>,
) -> Result<ProjectScan, String> {
    let extra_protected = db.list_protected()?;
    let cancel = control.cancel.clone();
    cancel.store(false, Ordering::Relaxed);
    let scan = tauri::async_runtime::spawn_blocking(move || {
        let raw = scan::walk(
            &root,
            &cancel,
            |p| {
                let _ = app.emit("scan://progress", p);
            },
            // Projects tab has no restore flow — nothing to checkpoint.
            |_files, _total, _bytes, _elapsed| {},
        );
        if raw.cancelled {
            return None;
        }
        Some(projects::detect(&root, &raw.files, &extra_protected))
    })
    .await
    .map_err(|e| e.to_string())?;

    scan.ok_or_else(|| "cancelled".to_string())
}

/// Requests cancellation of the in-flight scan. The walker stops at its next
/// per-entry check and `scan_directory` returns an `Err("cancelled")`.
#[tauri::command]
pub fn cancel_scan(control: State<'_, ScanControl>) {
    control.cancel.store(true, Ordering::Relaxed);
}

/// Returns the last cached scan for a root, if any — instant reload without re-walking.
#[tauri::command]
pub fn get_cached_scan(root: String, db: State<'_, Arc<Db>>) -> Result<Option<CachedScan>, String> {
    match db.cached_scan(&root)? {
        Some(row) => {
            let summary: ScanSummary =
                serde_json::from_str(&row.summary_json).map_err(|e| e.to_string())?;
            Ok(Some(CachedScan {
                ts: row.ts,
                status: row.status,
                scanned_files: row.scanned_files,
                total_files: row.total_files,
                summary,
            }))
        }
        None => Ok(None),
    }
}

/// Drops a root's cached scan — backs the restore card's "Discard" button.
#[tauri::command]
pub fn remove_cached_scan(root: String, db: State<'_, Arc<Db>>) -> Result<(), String> {
    db.remove_cached_scan(&root)
}

/// Destructive but reversible: moves the given items to the Recycle Bin.
/// Only ever called with items the user explicitly ticked and confirmed.
/// Every outcome is recorded in the audit log.
#[tauri::command]
pub async fn quarantine_paths(
    items: Vec<QuarantineItem>,
    app: AppHandle,
    db: State<'_, Arc<Db>>,
) -> Result<QuarantineResult, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        quarantine::quarantine(items, |p| {
            let _ = app.emit("quarantine://progress", p);
        })
    })
    .await
    .map_err(|e| e.to_string())?;

    for m in &result.moved {
        let _ = db.log_action("quarantine", &m.path, m.size_bytes, &m.kind, "moved");
    }
    for f in &result.failed {
        let _ = db.log_action("quarantine", &f.path, 0, &f.kind, "failed");
    }
    Ok(result)
}

/// Full history of every quarantine action, newest first.
#[tauri::command]
pub fn get_audit_log(db: State<'_, Arc<Db>>) -> Result<Vec<AuditEntry>, String> {
    db.audit_log(500)
}

#[tauri::command]
pub fn list_protected(db: State<'_, Arc<Db>>) -> Result<Vec<String>, String> {
    db.list_protected()
}

#[tauri::command]
pub fn add_protected(path: String, db: State<'_, Arc<Db>>) -> Result<(), String> {
    db.add_protected(&path)
}

#[tauri::command]
pub fn remove_protected(path: String, db: State<'_, Arc<Db>>) -> Result<(), String> {
    db.remove_protected(&path)
}

/// Suggested scan starting points — mounted drives and common user folders.
/// Read-only enumeration; lets the UI propose targets instead of forcing a pick.
#[tauri::command]
pub fn list_locations() -> Vec<Location> {
    locations::suggested_locations()
}

/// Mounted-volume capacity readout for the Dashboard. Read-only.
#[tauri::command]
pub fn disk_usage() -> Vec<DiskUsage> {
    disk::disk_usage()
}

/// Advisory: asks the configured AI provider (via the desktop `ai` domain) for a
/// second opinion on the current proposals. Deletes nothing and never changes a
/// proposal's risk — the returned notes/verdicts inform the user's own review.
///
/// `_config` is accepted for wire-compatibility with the old Agent-Platform path
/// (the frontend still sends it) but is unused here — the provider is whatever the
/// user configured under AI settings. See DISK_UTILITY_MIGRATION.md.
#[tauri::command]
pub async fn verify_proposals(
    root: String,
    proposals: Vec<Proposal>,
    #[allow(non_snake_case)] config: AiConfig,
    app: AppHandle,
    ai: State<'_, Arc<AIService>>,
    control: State<'_, VerifyControl>,
) -> Result<VerificationResult, String> {
    let _config = config;
    // Fresh flag for this run — clears any leftover request from a prior verify.
    let cancel = control.cancel.clone();
    cancel.store(false, Ordering::Relaxed);

    // The AI-domain call is a single non-cancellable generation; honour a Stop
    // requested before it starts, and drive the UI panel with two synthetic
    // progress frames (planning → reviewing) since there is no poll loop.
    if cancel.load(Ordering::Relaxed) {
        return Err("cancelled".to_string());
    }
    let _ = app.emit(
        "verify://progress",
        VerifyProgress { process_id: 0, status: "planning".to_string(), tasks: vec![] },
    );
    let _ = app.emit(
        "verify://progress",
        VerifyProgress {
            process_id: 0,
            status: "running".to_string(),
            tasks: vec![VerifyTask { role: "AI Reviewer".to_string(), status: "in_progress".to_string() }],
        },
    );

    verify_ai::verify_with_ai(root, proposals, ai.inner().as_ref()).await
}

/// Requests cancellation of the in-flight AI verification. The poll loop stops
/// at its next check and `verify_proposals` returns `Err("cancelled")`.
#[tauri::command]
pub fn cancel_verify(control: State<'_, VerifyControl>) {
    control.cancel.store(true, Ordering::Relaxed);
}

/// Lists the platform's team templates for the Settings picker. Each carries an
/// `isAppTeam` flag so the UI can preselect our own roster or warn if it's absent.
#[tauri::command]
pub async fn list_ai_teams(config: AiConfig) -> Result<Vec<TeamOption>, String> {
    tauri::async_runtime::spawn_blocking(move || verify::list_teams(config))
        .await
        .map_err(|e| e.to_string())?
}

/// Creates this app's dedicated "Deletion Verifier" team on the platform on demand
/// (Settings button). Idempotent — returns the existing one if already provisioned.
#[tauri::command]
pub async fn provision_ai_team(config: AiConfig) -> Result<TeamOption, String> {
    tauri::async_runtime::spawn_blocking(move || verify::provision_team(config))
        .await
        .map_err(|e| e.to_string())?
}

/// Opens the OS Recycle Bin so the user can restore anything moved. This is the
/// undo path — the app never force-restores; the user decides in their trash.
#[tauri::command]
pub fn open_recycle_bin() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg("shell:RecycleBinFolder")
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Open your system Trash to restore items.".to_string())
    }
}

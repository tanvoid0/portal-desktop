//! Tauri adapter for project discovery.
//!
//! Progress is emitted on `discovery://progress`, matching the disk domain's
//! `scan://progress` convention, and throttled here rather than in the engine so
//! the walk stays a plain synchronous function that is easy to test.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tauri::State;

use crate::database::DatabaseManager;
use crate::domains::projects::discovery::{DiscoveryConfig, DiscoveryReport};
use crate::domains::projects::services::discovery_service::{
    DiscoveryImportResult, DiscoveryService,
};
use crate::events::{EventSinkExt, Events};

const EMIT_EVERY_MS: u128 = 120;

/// Cancellation flag for the in-flight discovery walk, mirroring `ScanControl`
/// in the disk domain.
#[derive(Default)]
pub struct DiscoveryControl {
    pub cancel: Arc<AtomicBool>,
}

/// Walks the roots without touching the database. Use this to preview what a
/// scan would import before committing to it.
#[portal_rpc_macro::rpc]
pub async fn discover_projects(
    config: DiscoveryConfig,
    events: State<'_, Events>,
    control: State<'_, DiscoveryControl>,
) -> Result<DiscoveryReport, String> {
    run_scan(config, &events, &control).await
}

/// Walks the roots and writes the result into the projects registry.
#[portal_rpc_macro::rpc]
pub async fn discover_and_import_projects(
    config: DiscoveryConfig,
    mark_missing: Option<bool>,
    events: State<'_, Events>,
    control: State<'_, DiscoveryControl>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<DiscoveryImportResult, String> {
    let report = run_scan(config, &events, &control).await?;
    let service = DiscoveryService::new(&db_manager);
    let outcome = service
        .import(&report, mark_missing.unwrap_or(false))
        .await?;
    Ok(DiscoveryImportResult::new(report, outcome))
}

/// Imports a report produced earlier by `discover_projects`, so the user can
/// review the preview and then commit it without walking the disk twice.
#[portal_rpc_macro::rpc]
pub async fn import_discovered_projects(
    report: DiscoveryReport,
    mark_missing: Option<bool>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<DiscoveryImportResult, String> {
    let service = DiscoveryService::new(&db_manager);
    let outcome = service
        .import(&report, mark_missing.unwrap_or(false))
        .await?;
    Ok(DiscoveryImportResult::new(report, outcome))
}

#[portal_rpc_macro::rpc]
pub async fn cancel_discovery(control: State<'_, DiscoveryControl>) -> Result<(), String> {
    control.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

#[portal_rpc_macro::rpc]
pub async fn set_project_triage(
    id: i32,
    decision: Option<String>,
    note: Option<String>,
    successor: Option<String>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<bool, String> {
    let service = DiscoveryService::new(&db_manager);
    service.set_triage(id, decision, note, successor).await
}

#[portal_rpc_macro::rpc]
pub async fn set_project_triage_bulk(
    ids: Vec<i32>,
    decision: Option<String>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<u64, String> {
    let service = DiscoveryService::new(&db_manager);
    service.set_triage_bulk(ids, decision).await
}

async fn run_scan(
    config: DiscoveryConfig,
    events: &State<'_, Events>,
    control: &State<'_, DiscoveryControl>,
) -> Result<DiscoveryReport, String> {
    if config.roots.is_empty() {
        return Err("no roots configured".to_string());
    }
    let sink = events.sink();
    let cancel = control.cancel.clone();
    cancel.store(false, Ordering::Relaxed);

    let report = tauri::async_runtime::spawn_blocking(move || {
        let mut last = Instant::now();
        let mut first = true;
        DiscoveryService::scan(&config, |p| {
            // Always emit the first and the final tick so the bar starts and
            // finishes cleanly; throttle everything in between.
            let finished = p.total > 0 && p.done == p.total;
            if first || finished || last.elapsed().as_millis() >= EMIT_EVERY_MS {
                last = Instant::now();
                first = false;
                let _ = sink.emit("discovery://progress", p);
            }
            !cancel.load(Ordering::Relaxed)
        })
    })
    .await
    .map_err(|e| e.to_string())?;

    if report.cancelled {
        return Err("cancelled".to_string());
    }
    Ok(report)
}

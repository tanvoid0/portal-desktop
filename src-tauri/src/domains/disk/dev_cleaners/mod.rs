//! Pluggable dev-tool cleaners — each implementation discovers reclaimable
//! resources via its own mechanism (CLI, filesystem walk, …). Register new
//! cleaners in `registry()` below.

pub mod container_runtime;
pub mod projects;

use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};

use crate::domains::disk::classify::Risk;
use crate::domains::disk::scan;

/// Scan roots and protected paths shared by filesystem-walk cleaners.
#[derive(Clone, Default)]
pub struct DevCleanerScanContext {
    pub roots: Vec<String>,
    pub extra_protected: Vec<String>,
}

/// Progress + cancellation hooks for long filesystem walks.
pub struct DevCleanerWalkHooks<'a> {
    pub cancel: &'a AtomicBool,
    pub on_progress: &'a dyn Fn(scan::Progress),
}

/// One reclaimable resource surfaced for human review.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevCleanerItem {
    pub id: String,
    pub cleaner_id: String,
    pub path: String,
    pub kind: String,
    pub reason: String,
    pub size_bytes: u64,
    pub file_count: u64,
    pub risk: Risk,
    /// Parent grouping key — e.g. project root for nested UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_label: Option<String>,
}

/// Scan result from a single cleaner implementation.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevCleanerGroup {
    pub cleaner_id: String,
    pub label: String,
    pub available: bool,
    pub unavailable_reason: Option<String>,
    pub total_bytes: u64,
    pub items: Vec<DevCleanerItem>,
}

/// Combined scan across every registered cleaner.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevCleanerScan {
    pub total_bytes: u64,
    pub item_count: usize,
    pub groups: Vec<DevCleanerGroup>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevCleanerCleanItem {
    pub cleaner_id: String,
    pub id: String,
    pub path: String,
    pub kind: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanedItem {
    pub path: String,
    pub kind: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedCleanItem {
    pub path: String,
    pub kind: String,
    pub error: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevCleanerCleanResult {
    pub cleaned: Vec<CleanedItem>,
    pub failed: Vec<FailedCleanItem>,
    pub reclaimed_bytes: u64,
}

/// Live progress while dev-cleaner items are removed.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevCleanerCleanProgress {
    pub done: usize,
    pub total: usize,
    pub current_path: String,
}

/// Strategy interface for dev-tool disk cleaners.
pub trait DevCleaner: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    /// True when `scan` needs filesystem roots in the context (e.g. projects).
    fn needs_roots(&self) -> bool;
    fn scan(
        &self,
        ctx: &DevCleanerScanContext,
        hooks: Option<&DevCleanerWalkHooks<'_>>,
    ) -> Result<DevCleanerGroup, String>;
    fn clean(&self, items: &[DevCleanerItem]) -> (Vec<CleanedItem>, Vec<FailedCleanItem>);
}

fn registry() -> Vec<Box<dyn DevCleaner>> {
    vec![
        Box::new(projects::ProjectDevCleaner),
        Box::new(container_runtime::ContainerRuntimeCleaner::docker()),
        Box::new(container_runtime::ContainerRuntimeCleaner::podman()),
    ]
}

/// Runs every registered cleaner and merges results.
pub fn scan_all(
    ctx: DevCleanerScanContext,
    hooks: Option<&DevCleanerWalkHooks<'_>>,
) -> Result<DevCleanerScan, String> {
    let mut groups = Vec::new();
    for cleaner in registry() {
        let group = if cleaner.needs_roots() && ctx.roots.is_empty() {
            DevCleanerGroup {
                cleaner_id: cleaner.id().to_string(),
                label: cleaner.label().to_string(),
                available: false,
                unavailable_reason: Some(
                    "Pick one or more drives or folders to scan for project junk".into(),
                ),
                total_bytes: 0,
                items: vec![],
            }
        } else {
            cleaner.scan(&ctx, if cleaner.needs_roots() { hooks } else { None })?
        };
        groups.push(group);
    }

    let total_bytes = groups.iter().map(|g| g.total_bytes).sum();
    let item_count = groups.iter().map(|g| g.items.len()).sum();
    Ok(DevCleanerScan {
        total_bytes,
        item_count,
        groups,
    })
}

/// Removes user-confirmed items, routing each to its owning cleaner.
pub fn clean_items<F: FnMut(DevCleanerCleanProgress)>(
    items: Vec<DevCleanerCleanItem>,
    mut on_progress: F,
) -> DevCleanerCleanResult {
    let cleaners = registry();
    let mut cleaned = Vec::new();
    let mut failed = Vec::new();
    let mut reclaimed_bytes = 0u64;
    let total = items.len();

    for (i, item) in items.into_iter().enumerate() {
        on_progress(DevCleanerCleanProgress {
            done: i,
            total,
            current_path: item.path.clone(),
        });

        let Some(cleaner) = cleaners.iter().find(|c| c.id() == item.cleaner_id) else {
            failed.push(FailedCleanItem {
                path: item.path,
                kind: item.kind,
                error: format!("unknown cleaner: {}", item.cleaner_id),
            });
            continue;
        };

        let dev_item = DevCleanerItem {
            id: item.id,
            cleaner_id: item.cleaner_id,
            path: item.path.clone(),
            kind: item.kind,
            reason: String::new(),
            size_bytes: 0,
            file_count: 0,
            risk: Risk::Safe,
            group_label: None,
        };

        let (mut ok, mut err) = cleaner.clean(&[dev_item]);
        for c in &ok {
            reclaimed_bytes += c.size_bytes;
        }
        cleaned.append(&mut ok);
        failed.append(&mut err);

        on_progress(DevCleanerCleanProgress {
            done: i + 1,
            total,
            current_path: String::new(),
        });
    }

    DevCleanerCleanResult {
        cleaned,
        failed,
        reclaimed_bytes,
    }
}

//! Orchestrates a discovery run and its import into the registry.

use std::sync::Arc;

use crate::database::DatabaseManager;
use crate::domains::projects::discovery::{
    discover, state, DiscoveryChange, DiscoveryConfig, DiscoveryProgress, DiscoveryReport,
    DiscoveryState,
};
use crate::domains::projects::repositories::discovery_repository::{
    DiscoveryRepository, ImportOutcome,
};

/// Result of a scan-and-import cycle. Lives here rather than in the Tauri
/// adapter because ts-rs only exports from this crate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryImportResult {
    pub report: DiscoveryReport,
    pub inserted: Vec<String>,
    pub updated: Vec<String>,
    /// Rows that already existed at the same path and have now been linked to a
    /// discovery id.
    pub adopted: Vec<String>,
    /// Previously discovered projects the walk did not find this time.
    pub missing: Vec<String>,
    /// What this scan changed against the previous one. Empty on a preview
    /// import, which has no scan of its own to compare.
    #[serde(default)]
    pub changes: Vec<DiscoveryChange>,
}

impl DiscoveryImportResult {
    pub fn new(report: DiscoveryReport, outcome: ImportOutcome) -> Self {
        Self {
            report,
            inserted: outcome.inserted,
            updated: outcome.updated,
            adopted: outcome.adopted,
            missing: outcome.missing,
            changes: Vec::new(),
        }
    }

    pub fn with_changes(mut self, changes: Vec<DiscoveryChange>) -> Self {
        self.changes = changes;
        self
    }
}

pub struct DiscoveryService {
    repository: DiscoveryRepository,
}

impl DiscoveryService {
    pub fn new(db_manager: &Arc<DatabaseManager>) -> Self {
        Self {
            repository: DiscoveryRepository::new(db_manager.clone()),
        }
    }

    /// Walks the configured roots. Blocking and CPU/IO heavy — callers run it on
    /// a blocking thread and forward `on_progress` to the event sink.
    /// `on_progress` returns `false` to stop the walk early; the report then
    /// carries `cancelled: true` and a partial project list.
    pub fn scan<F>(config: &DiscoveryConfig, on_progress: F) -> DiscoveryReport
    where
        F: FnMut(DiscoveryProgress) -> bool,
    {
        discover(config, on_progress)
    }

    pub async fn import(
        &self,
        report: &DiscoveryReport,
        mark_missing: bool,
    ) -> Result<ImportOutcome, String> {
        self.repository.import(report, mark_missing).await
    }

    /// Audit state — roots, last report, change log. Read on view open so the
    /// page shows the previous scan instead of an empty table.
    pub fn state() -> DiscoveryState {
        state::load()
    }

    pub fn save_config(config: DiscoveryConfig) -> Result<DiscoveryState, String> {
        state::save_config(config)
    }

    /// Stores the completed scan and returns what it changed.
    pub fn record_scan(config: &DiscoveryConfig, report: &DiscoveryReport) -> Vec<DiscoveryChange> {
        state::record_scan(config, report)
    }

    pub async fn set_triage(
        &self,
        id: i32,
        decision: Option<String>,
        note: Option<String>,
        successor: Option<String>,
    ) -> Result<bool, String> {
        self.repository
            .set_triage(id, normalise_decision(decision)?, note, successor)
            .await
    }

    pub async fn set_triage_bulk(
        &self,
        ids: Vec<i32>,
        decision: Option<String>,
    ) -> Result<u64, String> {
        self.repository
            .set_triage_bulk(ids, normalise_decision(decision)?)
            .await
    }
}

/// Rejects unknown decisions rather than storing them: the UI filters on these
/// values, and a typo would make a project silently unreachable in every view.
fn normalise_decision(decision: Option<String>) -> Result<Option<String>, String> {
    match decision.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(d) if matches!(d, "keep" | "archive" | "delete" | "review") => Ok(Some(d.to_string())),
        Some(other) => Err(format!(
            "unknown triage decision `{other}` (expected keep, archive, delete or review)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_decisions_and_clears_on_empty() {
        assert_eq!(
            normalise_decision(Some("keep".into())).unwrap(),
            Some("keep".into())
        );
        assert_eq!(normalise_decision(Some("  ".into())).unwrap(), None);
        assert_eq!(normalise_decision(None).unwrap(), None);
    }

    #[test]
    fn rejects_unknown_decisions() {
        assert!(normalise_decision(Some("maybe".into())).is_err());
    }
}

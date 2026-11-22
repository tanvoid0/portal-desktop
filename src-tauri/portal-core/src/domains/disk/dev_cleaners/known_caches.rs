//! Well-known tool caches (npm, pnpm, Gradle, Cargo registry, pip, NuGet, editor
//! caches …) resolved from the user's home dir and sized *fast*.
//!
//! Sizing is the point: a `jwalk` pass reads metadata per file, which takes
//! minutes on a 100 GB tree. On Windows, `robocopy /L` in list-only mode reads
//! NTFS directory records and prints the byte total without opening a file, so
//! a multi-GB cache sizes in seconds. Other platforms (or a failed parse) fall
//! back to the walk.

use std::path::{Path, PathBuf};

use crate::domains::disk::classify::Risk;
use crate::domains::disk::dev_cleaners::{
    CleanedItem, DevCleaner, DevCleanerGroup, DevCleanerItem, DevCleanerScanContext,
    DevCleanerWalkHooks, FailedCleanItem,
};
use crate::domains::disk::quarantine::{self, QuarantineItem};

/// (label, path relative to home, risk, why it is reclaimable)
const TARGETS: &[(&str, &str, Risk, &str)] = &[
    ("npm cache", "AppData\\Local\\npm-cache", Risk::Safe, "Re-downloaded on next install"),
    ("pnpm store", "AppData\\Local\\pnpm\\store", Risk::Review, "Shared by every pnpm project; re-downloaded on next install"),
    ("Gradle caches", ".gradle\\caches", Risk::Safe, "Re-downloaded on next build"),
    ("Cargo registry", ".cargo\\registry", Risk::Review, "Crates re-downloaded on next build"),
    ("pip cache", "AppData\\Local\\pip\\Cache", Risk::Safe, "Re-downloaded on next install"),
    ("NuGet packages", ".nuget\\packages", Risk::Review, "Packages restored on next build"),
    ("User .cache", ".cache", Risk::Review, "Mixed tool caches (models, builds); contents vary"),
    ("Cursor cache", "AppData\\Roaming\\Cursor\\Cache", Risk::Safe, "Editor cache"),
    ("Cursor cached data", "AppData\\Roaming\\Cursor\\CachedData", Risk::Safe, "Editor cache"),
];

pub struct KnownCachesCleaner;

impl DevCleaner for KnownCachesCleaner {
    fn id(&self) -> &'static str {
        "known_caches"
    }

    fn label(&self) -> &'static str {
        "Tool caches"
    }

    fn needs_roots(&self) -> bool {
        false
    }

    fn scan(
        &self,
        _ctx: &DevCleanerScanContext,
        _hooks: Option<&DevCleanerWalkHooks<'_>>,
    ) -> Result<DevCleanerGroup, String> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map(PathBuf::from)
            .map_err(|_| "no home directory".to_string())?;

        let mut items: Vec<DevCleanerItem> = TARGETS
            .iter()
            .filter_map(|(label, rel, risk, reason)| {
                let path = home.join(rel);
                if !path.is_dir() {
                    return None;
                }
                let size_bytes = fast_dir_size(&path);
                (size_bytes > 0).then(|| DevCleanerItem {
                    id: format!("known_caches:{rel}"),
                    cleaner_id: "known_caches".into(),
                    path: path.to_string_lossy().into_owned(),
                    kind: (*label).into(),
                    reason: (*reason).into(),
                    size_bytes,
                    file_count: 0, // not available from the fast path
                    risk: *risk,
                    group_label: None,
                })
            })
            .collect();

        items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        Ok(DevCleanerGroup {
            cleaner_id: self.id().into(),
            label: self.label().into(),
            available: true,
            unavailable_reason: None,
            total_bytes: items.iter().map(|i| i.size_bytes).sum(),
            items,
        })
    }

    fn clean(
        &self,
        items: &[DevCleanerItem],
        permanent: bool,
    ) -> (Vec<CleanedItem>, Vec<FailedCleanItem>) {
        let batch = items
            .iter()
            .map(|i| QuarantineItem {
                path: i.path.clone(),
                kind: i.kind.clone(),
                permanent,
            })
            .collect();
        let r = quarantine::quarantine(batch, |_| {});
        (
            r.moved
                .into_iter()
                .map(|m| CleanedItem { path: m.path, kind: m.kind, size_bytes: m.size_bytes })
                .collect(),
            r.failed
                .into_iter()
                .map(|f| FailedCleanItem { path: f.path, kind: f.kind, error: f.error })
                .collect(),
        )
    }
}

/// Total bytes under `dir`. Fast path on Windows, `jwalk` elsewhere / on failure.
pub fn fast_dir_size(dir: &Path) -> u64 {
    #[cfg(windows)]
    if let Some(b) = robocopy_size(dir) {
        return b;
    }
    walk_size(dir)
}

fn walk_size(dir: &Path) -> u64 {
    jwalk::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

#[cfg(windows)]
fn robocopy_size(dir: &Path) -> Option<u64> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    // /L list only (nothing copied), /XJ skip junctions (no loops / double count),
    // /BYTES exact sizes. The destination is never written because of /L.
    let out = std::process::Command::new("robocopy")
        .arg(dir)
        .arg(std::env::temp_dir().join("portal-size-probe"))
        .args(["/L", "/E", "/XJ", "/NJH", "/NFL", "/NDL", "/NP", "/R:0", "/W:0", "/BYTES"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    parse_robocopy_bytes(&String::from_utf8_lossy(&out.stdout))
}

/// Pulls the total from the summary line `   Bytes :   <total> <copied> ...`.
/// Only English-locale robocopy prints "Bytes :"; anything else returns None so
/// the caller falls back to the walk.
fn parse_robocopy_bytes(stdout: &str) -> Option<u64> {
    let line = stdout.lines().find(|l| l.trim_start().starts_with("Bytes :"))?;
    line.split_whitespace().nth(2)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_robocopy_summary() {
        let s = "               Total    Copied   Skipped\n   Dirs :       4         4         0\n  Files :      10        10         0\n  Bytes : 19753983180 19753983180         0\n";
        assert_eq!(parse_robocopy_bytes(s), Some(19_753_983_180));
        assert_eq!(parse_robocopy_bytes("garbage"), None);
    }

    #[test]
    fn fast_size_matches_walk() {
        let d = std::env::temp_dir().join("portal-known-caches-test");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("a")).unwrap();
        std::fs::write(d.join("a").join("x"), vec![0u8; 1000]).unwrap();
        std::fs::write(d.join("y"), vec![0u8; 234]).unwrap();
        assert_eq!(fast_dir_size(&d), 1234);
        assert_eq!(walk_size(&d), 1234);
        let _ = std::fs::remove_dir_all(&d);
    }
}

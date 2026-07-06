use serde::{Deserialize, Serialize};

/// One item the user ticked for removal. `kind` is carried through purely so the
/// audit log can record what class of thing was moved.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineItem {
    pub path: String,
    pub kind: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovedItem {
    pub path: String,
    pub kind: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedItem {
    pub path: String,
    pub kind: String,
    pub error: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineResult {
    pub moved: Vec<MovedItem>,
    pub failed: Vec<FailedItem>,
    pub reclaimed_bytes: u64,
}

/// Live progress emitted once per item as the batch is moved to the trash.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineProgress {
    /// Items processed so far (moved + failed).
    pub done: usize,
    pub total: usize,
    pub current_path: String,
}

fn path_size(p: &std::path::Path) -> u64 {
    if p.is_file() {
        return std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    }
    jwalk::WalkDir::new(p)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Moves each path to the OS Recycle Bin / Trash. Never a hard delete — every
/// action is reversible by the user from their trash. This is the only place
/// the app touches user files destructively.
/// `on_progress` is invoked once after each item is processed, plus once up
/// front so the UI can render a full bar before the first (possibly slow) move.
pub fn quarantine<F: FnMut(QuarantineProgress)>(
    items: Vec<QuarantineItem>,
    mut on_progress: F,
) -> QuarantineResult {
    let mut moved = Vec::new();
    let mut failed = Vec::new();
    let mut reclaimed_bytes = 0u64;
    let total = items.len();

    for (i, item) in items.into_iter().enumerate() {
        on_progress(QuarantineProgress {
            done: i,
            total,
            current_path: item.path.clone(),
        });
        let path = std::path::Path::new(&item.path);
        let size = path_size(path);
        match trash::delete(path) {
            Ok(_) => {
                reclaimed_bytes += size;
                moved.push(MovedItem {
                    path: item.path,
                    kind: item.kind,
                    size_bytes: size,
                });
            }
            Err(e) => failed.push(FailedItem {
                path: item.path,
                kind: item.kind,
                error: e.to_string(),
            }),
        }
        on_progress(QuarantineProgress {
            done: i + 1,
            total,
            current_path: String::new(),
        });
    }

    QuarantineResult {
        moved,
        failed,
        reclaimed_bytes,
    }
}

//! Read-only discovery of suggested scan starting points: mounted drives and
//! common user folders. Purely enumerative — touches no file contents, deletes
//! nothing. Feeds the "suggest instead of make-them-pick" UX on the Cleanup tab.

use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    /// Absolute path to scan.
    pub path: String,
    /// Short display label, e.g. "Downloads" or "C:".
    pub label: String,
    /// "drive" | "folder" — lets the UI group/iconify.
    pub kind: String,
}

/// Existing drive roots (Windows) or `/` (unix), followed by common user
/// folders that actually exist. Never fabricates paths that aren't present.
pub fn suggested_locations() -> Vec<Location> {
    let mut out = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for c in b'A'..=b'Z' {
            let root = format!("{}:\\", c as char);
            if Path::new(&root).exists() {
                out.push(Location {
                    path: root.clone(),
                    label: format!("{}:", c as char),
                    kind: "drive".into(),
                });
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        out.push(Location {
            path: "/".into(),
            label: "/".into(),
            kind: "drive".into(),
        });
    }

    // Common user folders, resolved from the home dir plus %TEMP%.
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok();

    if let Some(home) = home {
        // The home root itself, e.g. C:\Users\{username}.
        if Path::new(&home).exists() {
            let label = Path::new(&home)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Home".into());
            out.push(Location {
                path: home.clone(),
                label,
                kind: "folder".into(),
            });
        }
        for name in [
            "Downloads",
            "Desktop",
            "Documents",
            "Pictures",
            "Videos",
            "AppData\\Local\\Temp",
        ] {
            let p = Path::new(&home).join(name);
            if p.exists() {
                out.push(Location {
                    path: p.to_string_lossy().into_owned(),
                    label: name.rsplit(['\\', '/']).next().unwrap_or(name).into(),
                    kind: "folder".into(),
                });
            }
        }
    }

    // OS temp dir (may differ from the per-user one above).
    let tmp = std::env::temp_dir();
    if tmp.exists() {
        let tmp_str = tmp.to_string_lossy().into_owned();
        if !out.iter().any(|l| l.path == tmp_str) {
            out.push(Location {
                path: tmp_str,
                label: "Temp".into(),
                kind: "folder".into(),
            });
        }
    }

    out
}

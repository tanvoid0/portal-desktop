//! Filesystem locations derived from the app's bundle identifier, and the one-time
//! migration out of the pre-rename locations.
//!
//! v0.9.0 renamed the app to "Portal Desktop" and the bundle identifier to
//! `com.tanvoid0.portal_desktop`. Both the Tauri-managed app data dir (keyed off the
//! identifier) and the config dir moved as a result, so existing installs need their
//! files carried across on first run of the renamed build.

use std::path::{Path, PathBuf};

use crate::{log_info, log_warn};

/// Bundle identifier from tauri.conf.json. Tauri derives the app data dir from it,
/// and the OS keychain entry is keyed off it.
pub const APP_IDENTIFIER: &str = "com.tanvoid0.portal_desktop";

/// Bundle identifier used before the v0.9.0 rename. Remove once no pre-rename
/// installs remain in the wild.
pub const LEGACY_APP_IDENTIFIER: &str = "com.tan.portal-desktop";

const CONFIG_DIR: &str = "portal_desktop";
const LEGACY_CONFIG_DIR: &str = "portal-desktop";

/// Per-user config directory (`settings.json`, `ai-settings.json`).
///
/// Migrates the pre-rename directory across on first call, so callers get a
/// populated directory without having to know the rename happened.
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join(CONFIG_DIR);

    if !dir.exists() {
        let legacy = base.join(LEGACY_CONFIG_DIR);
        if legacy.is_dir() {
            match std::fs::create_dir_all(&dir) {
                Ok(()) => copy_files_into(&legacy, &dir),
                Err(e) => log_warn!(
                    "AppPaths",
                    "Failed to create config dir {}: {}",
                    dir.display(),
                    e
                ),
            }
        }
    }

    dir
}

/// Copy every file in `src` into `dst`, skipping ones that already exist.
///
/// Shallow on purpose — both dirs hold flat files. The source is left in place so a
/// partial migration can be retried by hand.
pub fn copy_files_into(src: &Path, dst: &Path) {
    let entries = match std::fs::read_dir(src) {
        Ok(entries) => entries,
        Err(e) => {
            log_warn!("AppPaths", "Failed to read {}: {}", src.display(), e);
            return;
        }
    };

    for entry in entries.flatten() {
        let from = entry.path();
        if !from.is_file() {
            continue;
        }
        let to = dst.join(entry.file_name());
        if to.exists() {
            continue;
        }
        if let Err(e) = std::fs::copy(&from, &to) {
            log_warn!("AppPaths", "Failed to migrate {}: {}", from.display(), e);
        }
    }

    log_info!(
        "AppPaths",
        "Migrated files from legacy directory {}",
        src.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_legacy_files_without_clobbering_existing_ones() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();

        std::fs::write(src.path().join("portal_desktop.db"), b"old-main").unwrap();
        std::fs::write(src.path().join("disk_utility.db"), b"old-disk").unwrap();
        std::fs::create_dir(src.path().join("subdir")).unwrap();
        std::fs::write(dst.path().join("disk_utility.db"), b"already-here").unwrap();

        copy_files_into(src.path(), dst.path());

        // Missing file is carried over, existing one is left untouched.
        assert_eq!(
            std::fs::read(dst.path().join("portal_desktop.db")).unwrap(),
            b"old-main"
        );
        assert_eq!(
            std::fs::read(dst.path().join("disk_utility.db")).unwrap(),
            b"already-here"
        );

        // Shallow: directories are skipped, not recursed into.
        assert!(!dst.path().join("subdir").exists());

        // Source is left in place so a failed migration can be retried by hand.
        assert!(src.path().join("portal_desktop.db").exists());
    }
}

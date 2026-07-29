//! Filesystem locations derived from the app's bundle identifier, and the one-time
//! migration out of the pre-rename locations.
//!
//! v0.9.0 renamed the app to "Portal Desktop" and the bundle identifier to
//! `com.tanvoid0.portal_desktop`. Both the Tauri-managed app data dir (keyed off the
//! identifier) and the config dir moved as a result, so existing installs need their
//! files carried across on first run of the renamed build.
//!
//! v0.10.1 renamed the identifier again, to `com.tanvoid0.portal-desktop`: Tauri's
//! bundler rejects underscores in an identifier, so every v0.9.0/v0.10.0 platform
//! build failed and neither tag published binaries. Only locally-built installs ever
//! ran under the underscore form, but the data dir it created still has to be found,
//! hence a chain of legacy identifiers rather than a single one.

use std::path::{Path, PathBuf};

use crate::{log_info, log_warn};

/// Bundle identifier from tauri.conf.json. Tauri derives the app data dir from it,
/// and the OS keychain entry is keyed off it.
pub const APP_IDENTIFIER: &str = "com.tanvoid0.portal-desktop";

/// Identifiers this app has shipped under before, newest first. Data dirs and
/// keychain entries are looked up in this order and the first hit wins. Remove an
/// entry once no install can still be sitting on it.
pub const LEGACY_APP_IDENTIFIERS: &[&str] = &[
    // v0.9.0 / v0.10.0: never published — the bundler rejects the underscore — but
    // local builds from those tags created a data dir under it.
    "com.tanvoid0.portal_desktop",
    // Pre-v0.9.0, the last identifier with released builds behind it.
    "com.tan.portal-desktop",
];

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

    /// The bundler only validates the identifier during `tauri build`, which the
    /// release smoke job does not run — so a bad identifier gets caught by all four
    /// platform builds at once and the tag publishes nothing. That is what killed
    /// v0.9.0 and v0.10.0. Assert the rule here, where `cargo test` sees it.
    #[test]
    fn bundle_identifier_is_valid_and_matches_tauri_conf() {
        let conf: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json"))
                .unwrap(),
        )
        .unwrap();
        let configured = conf["identifier"].as_str().unwrap();

        assert_eq!(
            configured, APP_IDENTIFIER,
            "tauri.conf.json identifier and APP_IDENTIFIER have drifted; the app data \
             dir and keychain service would not match the bundle"
        );

        // Tauri: alphanumerics, hyphens and periods only. An underscore fails the build.
        assert!(
            configured
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.'),
            "identifier {configured:?} has characters the Tauri bundler rejects"
        );

        assert!(
            !LEGACY_APP_IDENTIFIERS.contains(&APP_IDENTIFIER),
            "current identifier is also listed as legacy; migration would read its own dir"
        );
    }

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

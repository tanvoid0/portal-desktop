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

/// True for debug builds — `tauri dev`, `cargo build`, `pnpm portal:ensure`'s old
/// path. A dev build runs under its own identity ([`app_identifier`]): own data dir,
/// config dir, keychain entry, WebView storage and daemon port. It used to share the
/// installed release's, ran its newer migrations on the release's database, and left
/// the release unable to start ("migration file ... is missing").
pub const DEV: bool = cfg!(debug_assertions);

const DEV_APP_IDENTIFIER: &str = "com.tanvoid0.portal-desktop.dev";

/// The identifier this build runs under. `lib.rs` writes it into the Tauri config at
/// startup, so Tauri's own paths and the single-instance lock follow it too.
pub fn app_identifier() -> &'static str {
    if DEV {
        DEV_APP_IDENTIFIER
    } else {
        APP_IDENTIFIER
    }
}

/// Where a first run copies data and the keychain master key from, newest first.
/// A dev build starts from a copy of the release's data, so it has real projects to
/// work against and never writes to the release's own files.
pub fn seed_identifiers() -> impl Iterator<Item = &'static str> {
    std::iter::once(APP_IDENTIFIER)
        .filter(|_| DEV)
        .chain(LEGACY_APP_IDENTIFIERS.iter().copied())
}

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
const DEV_CONFIG_DIR: &str = "portal_desktop-dev";
const LEGACY_CONFIG_DIR: &str = "portal-desktop";

/// Per-user config directory (`settings.json`, `ai-settings.json`).
///
/// Seeds itself on first call from the first older directory that exists — the
/// release's for a dev build, the pre-rename one otherwise — so callers get a
/// populated directory without having to know either happened.
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join(if DEV { DEV_CONFIG_DIR } else { CONFIG_DIR });

    if !dir.exists() {
        let seeds: &[&str] = if DEV {
            &[CONFIG_DIR, LEGACY_CONFIG_DIR]
        } else {
            &[LEGACY_CONFIG_DIR]
        };
        if let Some(seed) = seeds.iter().map(|d| base.join(d)).find(|d| d.is_dir()) {
            match std::fs::create_dir_all(&dir) {
                Ok(()) => copy_files_into(&seed, &dir),
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

    /// The half of the identifier contract that needs nothing outside this crate.
    ///
    /// The other half — that `APP_IDENTIFIER` still matches `tauri.conf.json`, and
    /// that the bundler will accept it — lives in `src-tauri/tests/bundle_identifier.rs`,
    /// because `tauri.conf.json` belongs to the shell and this crate must not
    /// reach up into it.
    #[test]
    fn current_identifier_is_not_also_listed_as_legacy() {
        assert!(
            !LEGACY_APP_IDENTIFIERS.contains(&APP_IDENTIFIER),
            "current identifier is also listed as legacy; migration would read its own dir"
        );
    }

    /// The whole point of the dev identity: a debug build (which is what runs this
    /// test) must never resolve the release's identifier, and must never seed from
    /// itself.
    #[test]
    #[cfg(debug_assertions)]
    fn dev_build_never_runs_as_the_release() {
        assert_ne!(app_identifier(), APP_IDENTIFIER);
        assert_eq!(seed_identifiers().next(), Some(APP_IDENTIFIER));
        assert!(!seed_identifiers().any(|id| id == app_identifier()));
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

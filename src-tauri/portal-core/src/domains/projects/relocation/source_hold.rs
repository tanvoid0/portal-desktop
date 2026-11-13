//! `#217`: for the duration of a move's copy, hold every file and directory
//! in the source tree open with a deny-write share mode — Windows only. An
//! other process trying to open one for write gets an OS-level sharing
//! violation instead of quietly racing the copy; today's only defense is
//! [`super::usage::check_in_use`]'s point-in-time probe, which this makes
//! stronger (actual denial, not just detection) without replacing it — the
//! in-use check still runs first, and the post-copy re-walk in
//! [`super::mover::final_verify`] is still the backstop for everything this
//! can't catch.
//!
//! ## What this does not do
//! - Does **not** stop a *new* file being created inside the source tree
//!   during the hold — a hold is per-path, taken on the move's manifest, not
//!   a directory watch. `final_verify`'s re-walk is still what catches that.
//! - Holds nothing across a Portal crash/restart: these are plain OS handles
//!   that close the moment the process exits. There is no persisted hold
//!   state to restore — a resumed move re-acquires a fresh hold over
//!   whatever is still on disk ([`acquire`] is called again).
//! - Unix has no enforced equivalent: there is no portable "deny other
//!   writers" open mode. [`acquire`] is a no-op there; the existing in-use
//!   checks are detection-only, same as before this module existed.
//! - Deliberately does **not** fall back to the read-only file attribute:
//!   that mutates the source (would need restoring after a crash, and most
//!   editors clear it on save anyway), so it's not an option here at all.

use std::fs::File;
use std::path::{Path, PathBuf};

use super::usage::Signal;

/// Live deny-write handles on every file/dir in a move's source tree.
/// Dropping it closes every handle — that's the only "release" there is.
#[derive(Debug)]
pub struct SourceHold {
    #[allow(dead_code)] // held only for the Drop side effect of closing them
    handles: Vec<File>,
}

/// Acquire the hold over every file in `rel_files` (manifest-relative, forward
/// slashes) plus every directory that contains one, under `source`. On
/// Windows, a sharing violation on any path — something else already has it
/// open for write — becomes a [`Signal`] so it slots into [`UsageReport`]'s
/// `blockers` list [`super::usage::UsageReport`] the same as any other
/// preflight blocker. Any handles already opened before the failure are
/// dropped (closed) along with the error.
#[cfg(target_os = "windows")]
pub fn acquire(source: &Path, rel_files: &[String]) -> Result<SourceHold, Signal> {
    use std::io;
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

    fn open_deny_write(path: &Path, is_dir: bool) -> io::Result<File> {
        let mut opts = std::fs::OpenOptions::new();
        opts.read(true).share_mode(FILE_SHARE_READ);
        if is_dir {
            // The standard trick to get a handle on a directory at all.
            opts.custom_flags(FILE_FLAG_BACKUP_SEMANTICS);
        }
        opts.open(path)
    }

    fn sharing_signal(path: &Path, e: io::Error) -> Signal {
        Signal {
            kind: "source_in_use".into(),
            detail: format!(
                "{} is already open elsewhere for writing, so it can't be held for the move: {e}",
                path.display()
            ),
        }
    }

    // Directories to hold: the source root plus every ancestor dir implied by
    // a file's relative path. build_manifest doesn't record empty dirs, so
    // there is nothing to hold that isn't an ancestor of some file in the set.
    let mut rel_dirs = std::collections::HashSet::new();
    for rel in rel_files {
        let parts: Vec<&str> = rel.split('/').collect();
        for i in 0..parts.len().saturating_sub(1) {
            rel_dirs.insert(parts[..=i].join("/"));
        }
    }

    let mut handles = Vec::with_capacity(rel_files.len() + rel_dirs.len() + 1);
    handles.push(open_deny_write(source, true).map_err(|e| sharing_signal(source, e))?);
    for rel in &rel_dirs {
        let path = os_join(source, rel);
        handles.push(open_deny_write(&path, true).map_err(|e| sharing_signal(&path, e))?);
    }
    for rel in rel_files {
        let path = os_join(source, rel);
        handles.push(open_deny_write(&path, false).map_err(|e| sharing_signal(&path, e))?);
    }
    Ok(SourceHold { handles })
}

#[cfg(target_os = "windows")]
fn os_join(base: &Path, rel: &str) -> PathBuf {
    base.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
}

/// ponytail: no portable "deny other writers" open mode exists on unix, so
/// there is no enforced equivalent to build here — this stays a genuine
/// no-op rather than a hold that only looks like one. Detection is still
/// `usage::check_in_use`, run by the caller before this.
#[cfg(not(target_os = "windows"))]
pub fn acquire(_source: &Path, _rel_files: &[String]) -> Result<SourceHold, Signal> {
    Ok(SourceHold { handles: Vec::new() })
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn acquire_holds_files_and_dirs_without_blocking_read() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "x").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/b.txt"), "y").unwrap();
        let rels = vec!["a.txt".to_string(), "sub/b.txt".to_string()];

        let hold = acquire(dir.path(), &rels).unwrap();

        // Held for read/delete-deny too, but a second read-only open and a
        // plain `fs::copy` (what the mover actually does) must still work —
        // this is the exact worry the task asks to verify.
        fs::read(dir.path().join("a.txt")).unwrap();
        fs::copy(dir.path().join("a.txt"), dir.path().join("a.copy")).unwrap();

        // A write open, in contrast, must be denied while the hold is up.
        let err = fs::OpenOptions::new()
            .write(true)
            .open(dir.path().join("a.txt"))
            .unwrap_err();
        assert_eq!(err.raw_os_error(), Some(32)); // ERROR_SHARING_VIOLATION

        drop(hold);
        // Released: the same write open now succeeds.
        fs::OpenOptions::new().write(true).open(dir.path().join("a.txt")).unwrap();
    }

    #[test]
    fn acquire_reports_a_sharing_violation_as_a_blocker_signal() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("locked.txt");
        fs::write(&path, "x").unwrap();
        // Hold it for write ourselves first, denying our own later open.
        let _busy = fs::OpenOptions::new().write(true).open(&path).unwrap();

        let err = acquire(dir.path(), &["locked.txt".to_string()]).unwrap_err();
        assert_eq!(err.kind, "source_in_use");
        assert!(err.detail.contains("locked.txt"));
    }
}

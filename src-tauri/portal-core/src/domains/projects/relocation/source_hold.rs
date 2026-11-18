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
use std::path::Path;

use super::mover::{os_join, MoveError};
use super::usage::{Signal, UsageReport};

/// Live deny-write handles on every file/dir in a move's source tree.
/// Dropping it closes every handle — that's the only "release" there is.
#[derive(Debug)]
pub struct SourceHold {
    #[allow(dead_code)] // held only for the Drop side effect of closing them
    handles: Vec<File>,
}

/// Acquire the hold over every file in `rel_files` (manifest-relative, forward
/// slashes) plus every directory that contains one, under `source`. On
/// Windows, a sharing or lock violation on any path — something else already
/// has it open for write — becomes a [`MoveError::Blocked`] signal, the same
/// as any other preflight blocker. Any other I/O error (the path vanished,
/// was denied by an ACL, ...) becomes a plain [`MoveError::Failed`] instead,
/// since "close whatever has it open" would be the wrong thing to tell the
/// user. The opens run on a small thread pool (#1110). On failure every handle
/// opened is dropped (closed) along with the error, and the error is the first
/// failing path in order — the source root, its dirs (sorted), then `rel_files`
/// — however the threads interleave.
#[cfg(target_os = "windows")]
pub fn acquire(source: &Path, rel_files: &[String]) -> Result<SourceHold, MoveError> {
    use std::io;
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::OnceLock;

    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    /// Threads opening handles at once. Open latency under an AV scan overlaps;
    /// past this the scanner, not the opens, is the limit.
    const HOLD_THREADS: usize = 16;

    fn open_deny_write(path: &Path, is_dir: bool) -> io::Result<File> {
        let mut opts = std::fs::OpenOptions::new();
        opts.read(true).share_mode(FILE_SHARE_READ);
        if is_dir {
            // The standard trick to get a handle on a directory at all.
            opts.custom_flags(FILE_FLAG_BACKUP_SEMANTICS);
        }
        opts.open(path)
    }

    // ERROR_SHARING_VIOLATION / ERROR_LOCK_VIOLATION mean something else has
    // the path open for write — an actionable, user-facing blocker. Anything
    // else (deleted between the manifest walk and here, ACL denial, ...) is
    // a real failure, not "close the other program".
    fn hold_error(path: &Path, e: io::Error) -> MoveError {
        match e.raw_os_error() {
            Some(32) | Some(33) => MoveError::Blocked(UsageReport {
                blockers: vec![Signal {
                    kind: "source_in_use".into(),
                    detail: format!(
                        "{} is already open elsewhere for writing, so it can't be held for the move: {e}",
                        path.display()
                    ),
                }],
                warnings: vec![],
            }),
            _ => MoveError::Failed(format!("hold {}: {e}", path.display())),
        }
    }

    // Directories to hold: the source root plus every ancestor dir implied by
    // a file's relative path. build_manifest doesn't record empty dirs, so
    // there is nothing to hold that isn't an ancestor of some file in the set.
    // A BTreeSet so the order below is the same on every run.
    let mut rel_dirs = std::collections::BTreeSet::new();
    for rel in rel_files {
        let parts: Vec<&str> = rel.split('/').collect();
        for i in 0..parts.len().saturating_sub(1) {
            rel_dirs.insert(parts[..=i].join("/"));
        }
    }

    // Root, then dirs, then files: the first path in this order that fails is
    // the error reported, however the opens below interleave.
    let paths: Vec<(PathBuf, bool)> = std::iter::once((source.to_path_buf(), true))
        .chain(rel_dirs.iter().map(|r| (os_join(source, r), true)))
        .chain(rel_files.iter().map(|r| (os_join(source, r), false)))
        .collect();

    // #1110: each open is latency-bound under a real-time AV scan (~7ms, CPU
    // idle), so 5k files opened one by one stalled the start of a move for
    // 20-55s. Open on HOLD_THREADS threads instead; each result lands in its
    // own slot by index, so they read back in `paths` order.
    // ponytail: a failure still finishes the pass (seconds); an AtomicUsize of
    // the lowest failing index would let workers stop early, keeping the order.
    let next = AtomicUsize::new(0);
    let slots: Vec<OnceLock<io::Result<File>>> = paths.iter().map(|_| OnceLock::new()).collect();
    std::thread::scope(|s| {
        for _ in 0..HOLD_THREADS.min(paths.len()) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((path, is_dir)) = paths.get(i) else {
                    break;
                };
                let _ = slots[i].set(open_deny_write(path, *is_dir));
            });
        }
    });

    // First failure in `paths` order wins. Returning drops `handles` and the
    // rest of `slots`, which closes every handle opened so far.
    let mut handles = Vec::with_capacity(paths.len());
    for (slot, (path, _)) in slots.into_iter().zip(&paths) {
        let opened = slot.into_inner().expect("scope joined every worker");
        handles.push(opened.map_err(|e| hold_error(path, e))?);
    }
    Ok(SourceHold { handles })
}

/// ponytail: no portable "deny other writers" open mode exists on unix, so
/// there is no enforced equivalent to build here — this stays a genuine
/// no-op rather than a hold that only looks like one. Detection is still
/// `usage::check_in_use`, run by the caller before this.
#[cfg(not(target_os = "windows"))]
pub fn acquire(_source: &Path, _rel_files: &[String]) -> Result<SourceHold, MoveError> {
    Ok(SourceHold {
        handles: Vec::new(),
    })
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

        // The directory hold must block a rename/delete too, not just a file
        // write open.
        let sub = dir.path().join("sub");
        let sub2 = dir.path().join("sub2");
        assert_eq!(
            fs::rename(&sub, &sub2).unwrap_err().raw_os_error(),
            Some(32)
        );
        assert_eq!(
            fs::remove_file(dir.path().join("a.txt"))
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );

        drop(hold);
        // Released: the same write open now succeeds.
        fs::OpenOptions::new()
            .write(true)
            .open(dir.path().join("a.txt"))
            .unwrap();
    }

    #[test]
    fn acquire_reports_a_sharing_violation_as_a_blocker_signal() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("locked.txt");
        fs::write(&path, "x").unwrap();
        // Hold it for write ourselves first, denying our own later open.
        let _busy = fs::OpenOptions::new().write(true).open(&path).unwrap();

        let err = acquire(dir.path(), &["locked.txt".to_string()]).unwrap_err();
        let MoveError::Blocked(report) = err else {
            panic!("expected Blocked, got {err:?}")
        };
        assert_eq!(report.blockers[0].kind, "source_in_use");
        assert!(report.blockers[0].detail.contains("locked.txt"));
    }

    #[test]
    fn acquire_propagates_a_missing_path_as_failed_not_blocked() {
        let dir = tempdir().unwrap();
        // Named in rel_files but never created — NotFound, not a sharing
        // violation, so this must not read as "close the other program".
        let err = acquire(dir.path(), &["missing.txt".to_string()]).unwrap_err();
        assert!(
            matches!(err, MoveError::Failed(_)),
            "expected Failed, got {err:?}"
        );
    }

    /// `n` small files over five dirs, as sorted manifest-style rels.
    fn many_files(root: &Path, n: usize) -> Vec<String> {
        let mut rels: Vec<String> = (0..n).map(|i| format!("d{}/f{i:03}.txt", i % 5)).collect();
        rels.sort();
        for rel in &rels {
            let path = root.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x").unwrap();
        }
        rels
    }

    fn open_for_write(path: &Path) -> std::io::Result<fs::File> {
        fs::OpenOptions::new().write(true).open(path)
    }

    /// #1110: the opens run on several threads — every one must still end up in
    /// the hold, and all of them must close when it drops.
    #[test]
    fn acquire_holds_every_file_when_opens_are_spread_over_threads() {
        let dir = tempdir().unwrap();
        let rels = many_files(dir.path(), 200);
        let hold = acquire(dir.path(), &rels).unwrap();
        for rel in &rels {
            let err = open_for_write(&dir.path().join(rel)).unwrap_err();
            assert_eq!(err.raw_os_error(), Some(32), "{rel} was not held");
        }
        drop(hold);
        for rel in &rels {
            open_for_write(&dir.path().join(rel)).unwrap();
        }
    }

    /// #1110: when one open fails, every handle the other threads already took
    /// is closed again, and the error reported is the first failing path in
    /// manifest order — here the write-locked file (Blocked), not the later
    /// missing one (Failed), whichever thread gets to its failure first.
    #[test]
    fn acquire_failure_releases_every_handle_and_reports_the_first_failure_in_order() {
        let dir = tempdir().unwrap();
        let rels = many_files(dir.path(), 200);
        let (locked, gone) = (&rels[20], &rels[150]);
        let _busy = open_for_write(&dir.path().join(locked)).unwrap();
        fs::remove_file(dir.path().join(gone)).unwrap();
        // A dir holding neither failing file: its handle must be released too.
        let spare = (0..5)
            .map(|k| format!("d{k}"))
            .find(|d| !locked.starts_with(d) && !gone.starts_with(d))
            .unwrap();

        for _ in 0..10 {
            let err = acquire(dir.path(), &rels).unwrap_err();
            let MoveError::Blocked(report) = err else {
                panic!("expected the earlier, Blocked failure, got {err:?}")
            };
            let detail = &report.blockers[0].detail;
            assert!(detail.contains(&locked.replace('/', "\\")), "{detail}");
            for rel in rels.iter().filter(|r| *r != locked && *r != gone) {
                open_for_write(&dir.path().join(rel)).unwrap();
            }
            fs::rename(dir.path().join(&spare), dir.path().join("moved")).unwrap();
            fs::rename(dir.path().join("moved"), dir.path().join(&spare)).unwrap();
        }
    }
}

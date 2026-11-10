//! Process-wide registry of project roots currently mid-relocation. The move
//! engine (step 4+ of the "safe project relocation" plan) locks the source
//! (and later the destination) for the duration of a move; anything that
//! would touch that tree in the meantime — coder, terminal, pipelines —
//! calls `ensure_unlocked` first.
//!
//! Sync and DB-free on purpose: sync commands (`coder_read_file` etc.) need
//! to call `is_locked`/`ensure_unlocked` without going async.

use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

static LOCKED_ROOTS: Lazy<RwLock<Vec<PathBuf>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Windows paths differ only by case and separator; comparing them raw would
/// miss matches. Moved here from `discovery_repository::paths_equal` (its
/// only other caller) so the move lock and discovery share one rule.
pub fn norm(s: &str) -> String {
    s.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

fn components(s: &str) -> Vec<String> {
    norm(s)
        .split('/')
        .filter(|c| !c.is_empty())
        .map(str::to_string)
        .collect()
}

/// True if `a` and `b` are the same directory, or one is nested inside the
/// other — component-wise, so `D:\a\foo` never matches the sibling
/// `D:\a\foobar`. Either side may be absolute or relative; empty input never
/// matches anything.
pub fn is_under(a: &str, b: &str) -> bool {
    let a = components(a);
    let b = components(b);
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let n = a.len().min(b.len());
    a[..n] == b[..n]
}

/// Lock `paths` for the duration of a move. Idempotent-ish: callers are
/// expected to `unlock` the same paths when done, in pairs.
pub fn lock(paths: &[PathBuf]) {
    let mut roots = LOCKED_ROOTS.write().unwrap();
    roots.extend(paths.iter().cloned());
}

pub fn unlock(paths: &[PathBuf]) {
    let targets: Vec<String> = paths.iter().map(|p| norm(&p.to_string_lossy())).collect();
    LOCKED_ROOTS
        .write()
        .unwrap()
        .retain(|r| !targets.contains(&norm(&r.to_string_lossy())));
}

/// Empty or relative paths are never locked — there is nothing meaningful to
/// compare them against.
pub fn is_locked(path: &Path) -> bool {
    let s = path.to_string_lossy();
    if s.is_empty() || !path.is_absolute() {
        return false;
    }
    LOCKED_ROOTS
        .read()
        .unwrap()
        .iter()
        .any(|root| is_under(&root.to_string_lossy(), &s))
}

pub fn ensure_unlocked(path: &Path) -> Result<(), String> {
    if is_locked(path) {
        Err(format!(
            "{} is locked by an in-progress project move",
            path.display()
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test uses its own root name — `LOCKED_ROOTS` is a process-wide
    // static and `cargo test` runs tests in parallel within one binary.
    #[test]
    fn prefix_matches_descendant() {
        lock(&[PathBuf::from(r"D:\mvlock1\foo")]);
        assert!(is_locked(Path::new(r"D:\mvlock1\foo\bar.txt")));
        unlock(&[PathBuf::from(r"D:\mvlock1\foo")]);
        assert!(!is_locked(Path::new(r"D:\mvlock1\foo\bar.txt")));
    }

    #[test]
    fn case_and_slash_insensitive() {
        lock(&[PathBuf::from(r"D:\mvlock2\AAA\Foo")]);
        assert!(is_locked(Path::new("d:/mvlock2/aaa/foo/bar")));
        unlock(&[PathBuf::from(r"D:\mvlock2\AAA\Foo")]);
    }

    #[test]
    fn sibling_with_shared_prefix_not_locked() {
        lock(&[PathBuf::from(r"D:\mvlock3\foo")]);
        assert!(!is_locked(Path::new(r"D:\mvlock3\foobar")));
        unlock(&[PathBuf::from(r"D:\mvlock3\foo")]);
    }

    #[test]
    fn empty_and_relative_never_locked() {
        lock(&[PathBuf::from(r"D:\mvlock4\foo")]);
        assert!(!is_locked(Path::new("")));
        assert!(!is_locked(Path::new(r"mvlock4\foo")));
        unlock(&[PathBuf::from(r"D:\mvlock4\foo")]);
    }
}

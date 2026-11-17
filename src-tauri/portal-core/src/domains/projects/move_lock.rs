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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

static LOCKED_ROOTS: Lazy<RwLock<Vec<PathBuf>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Count of moves whose copy/verify/promote/commit is *running right now* in
/// this process. Distinct from `LOCKED_ROOTS`: boot recovery re-arms a lock for
/// an interrupted move without running anything, so a held lock alone does not
/// mean work is in flight. The close guard uses this to tell "a move is
/// actually copying" from "a move is journaled, waiting for the user".
static ACTIVE_MOVES: AtomicUsize = AtomicUsize::new(0);

/// Mark an engine operation as running. Paired with [`end_active`]; the engine's
/// lock guard calls these so every path (success, error, `?`) balances.
pub fn begin_active() {
    ACTIVE_MOVES.fetch_add(1, Ordering::SeqCst);
}

pub fn end_active() {
    ACTIVE_MOVES.fetch_sub(1, Ordering::SeqCst);
}

/// True while a move's copy/verify/promote/commit is running in this process.
pub fn any_active() -> bool {
    ACTIVE_MOVES.load(Ordering::SeqCst) > 0
}

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

/// True if `child` is `root` or nested inside it — one direction only, unlike
/// [`is_under`]. For "is this process running from the project": a process
/// whose cwd is a shallow ancestor (the user's home, `C:\`) is not in the
/// project (board #431).
pub fn is_within(child: &str, root: &str) -> bool {
    let c = components(child);
    let r = components(root);
    !r.is_empty() && c.len() >= r.len() && c[..r.len()] == r[..]
}

/// True if `dest` lives on the disk mounted at `mount`. Unlike [`is_within`],
/// the Unix root `/` (zero components once normalised) mounts every absolute
/// path (board #782).
pub fn is_mount_of(mount: &str, dest: &str) -> bool {
    if norm(mount).is_empty() && mount.starts_with(['/', '\\']) {
        return dest.starts_with(['/', '\\']);
    }
    is_within(dest, mount)
}

/// Lock `paths` for the duration of a move. Idempotent-ish: callers are
/// expected to `unlock` the same paths when done, in pairs.
pub fn lock(paths: &[PathBuf]) {
    let mut roots = LOCKED_ROOTS.write().unwrap();
    roots.extend(paths.iter().cloned());
}

/// Like [`lock`], but refuses (returning the colliding root) when any of
/// `paths` is nested in, contains, or equals an already-locked root. The check
/// and the insert share one write lock, so two moves of overlapping trees can't
/// both pass — unlike the journal-row check, which only sees a move seconds
/// after it starts (board #413). Paths within `paths` may overlap each other.
pub fn try_lock(paths: &[PathBuf]) -> Result<(), PathBuf> {
    let mut roots = LOCKED_ROOTS.write().unwrap();
    for p in paths {
        let p_s = p.to_string_lossy();
        if let Some(hit) = roots.iter().find(|r| is_under(&r.to_string_lossy(), &p_s)) {
            return Err(hit.clone());
        }
    }
    roots.extend(paths.iter().cloned());
    Ok(())
}

pub fn unlock(paths: &[PathBuf]) {
    let targets: Vec<String> = paths.iter().map(|p| norm(&p.to_string_lossy())).collect();
    LOCKED_ROOTS
        .write()
        .unwrap()
        .retain(|r| !targets.contains(&norm(&r.to_string_lossy())));
}

fn locked_by(path: &Path, matches: fn(&str, &str) -> bool) -> bool {
    let s = path.to_string_lossy();
    if s.is_empty() || !path.is_absolute() {
        return false;
    }
    LOCKED_ROOTS
        .read()
        .unwrap()
        .iter()
        .any(|root| matches(&root.to_string_lossy(), &s))
}

/// Two-way: locked when `path` is inside a locked root *or* a locked root is
/// inside `path`. Use when the caller reads, writes, walks or runs git on the
/// tree rooted at `path` — an ancestor can reach the locked subtree.
///
/// Empty or relative paths are never locked — there is nothing meaningful to
/// compare them against.
pub fn is_locked(path: &Path) -> bool {
    locked_by(path, is_under)
}

/// One-way: locked only when `path` IS a locked root or inside one. Use when
/// `path` is merely a spawned process's cwd — a shell at an ancestor (the
/// user's home) holds no handle on the locked tree (board #781, cf. #431).
pub fn is_locked_within(path: &Path) -> bool {
    locked_by(path, |root, p| is_within(p, root))
}

pub fn ensure_unlocked_within(path: &Path) -> Result<(), String> {
    ensure(is_locked_within(path), path)
}

pub fn ensure_unlocked(path: &Path) -> Result<(), String> {
    ensure(is_locked(path), path)
}

fn ensure(locked: bool, path: &Path) -> Result<(), String> {
    if locked {
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
    // Windows path literals: `D:\…` is not absolute on Unix, so is_locked
    // short-circuits there.
    #[cfg(windows)]
    #[test]
    fn prefix_matches_descendant() {
        lock(&[PathBuf::from(r"D:\mvlock1\foo")]);
        assert!(is_locked(Path::new(r"D:\mvlock1\foo\bar.txt")));
        unlock(&[PathBuf::from(r"D:\mvlock1\foo")]);
        assert!(!is_locked(Path::new(r"D:\mvlock1\foo\bar.txt")));
    }

    #[cfg(windows)]
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
    fn try_lock_refuses_nested_and_ancestor_and_equal() {
        let base = if cfg!(windows) {
            r"D:\mvlock6"
        } else {
            "/mvlock6"
        };
        let app = PathBuf::from(base).join("app");
        let pkg = app.join("pkg");
        let other = PathBuf::from(base).join("other");
        assert!(try_lock(std::slice::from_ref(&app)).is_ok());
        assert!(try_lock(std::slice::from_ref(&pkg)).is_err()); // nested
        assert!(try_lock(std::slice::from_ref(&app)).is_err()); // equal
        assert!(try_lock(std::slice::from_ref(&other)).is_ok()); // disjoint
        unlock(&[app.clone(), other]);
        assert!(try_lock(std::slice::from_ref(&pkg)).is_ok());
        assert!(try_lock(std::slice::from_ref(&app)).is_err()); // ancestor
        unlock(&[pkg]);
    }

    #[test]
    fn is_within_is_one_directional() {
        assert!(is_within(r"C:\Users\me\proj\src", r"C:\Users\me\proj"));
        assert!(is_within(r"C:\Users\me\proj", r"c:/users/me/proj/"));
        assert!(!is_within(r"C:\Users\me", r"C:\Users\me\proj"));
        assert!(!is_within(r"C:\Users\me\projX", r"C:\Users\me\proj"));
        assert!(!is_within(r"C:\x", ""));
        assert!(!is_within("", r"C:\x"));
    }

    #[test]
    fn root_mount_matches_every_absolute_path() {
        assert!(is_mount_of("/", "/home/me/new"));
        assert!(is_mount_of("/home", "/home/me"));
        assert!(!is_mount_of("/home", "/homework"));
        assert!(!is_mount_of("/", ""));
        assert!(is_mount_of(r"C:\", r"C:\a\b"));
        assert!(!is_mount_of(r"C:\", r"D:\a"));
    }

    #[test]
    fn one_way_check_ignores_ancestor() {
        // Absolute on both platforms (`D:\…` is not absolute on Unix).
        let base = if cfg!(windows) {
            r"D:\mvlock5"
        } else {
            "/mvlock5"
        };
        let base = PathBuf::from(base).join("home");
        let root = base.join("proj");
        lock(std::slice::from_ref(&root));
        assert!(is_locked(&base));
        assert!(!is_locked_within(&base));
        assert!(ensure_unlocked_within(&base).is_ok());
        for p in [root.clone(), root.join("src")] {
            assert!(is_locked(&p));
            assert!(is_locked_within(&p));
            assert!(ensure_unlocked_within(&p).is_err());
        }
        unlock(&[root]);
    }

    #[test]
    fn empty_and_relative_never_locked() {
        lock(&[PathBuf::from(r"D:\mvlock4\foo")]);
        assert!(!is_locked(Path::new("")));
        assert!(!is_locked(Path::new(r"mvlock4\foo")));
        unlock(&[PathBuf::from(r"D:\mvlock4\foo")]);
    }
}

//! Real git in temp dirs. Replicas land at `<drive>/<temp path minus root>`.

use super::*;
use DriveSyncState as S;

fn sh(dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn init(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    sh(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("user.name", "t"),
        ("user.email", "t@t"),
        ("commit.gpgsign", "false"),
        ("core.autocrlf", "false"),
    ] {
        sh(dir, &["config", k, v]);
    }
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap()
}

fn commit(dir: &Path, file: &str, text: &str) {
    write(dir, file, text);
    sh(dir, &["add", file]);
    sh(dir, &["commit", "-q", "-m", file]);
}

struct Fx {
    _src: tempfile::TempDir,
    drive: tempfile::TempDir,
    local: PathBuf,
}

impl Fx {
    fn plain() -> Self {
        let src = tempfile::tempdir().unwrap();
        let local = src.path().join("proj");
        fs::create_dir(&local).unwrap();
        Self {
            _src: src,
            drive: tempfile::tempdir().unwrap(),
            local,
        }
    }

    fn git() -> Self {
        let f = Self::plain();
        init(&f.local);
        write(&f.local, ".gitignore", "node_modules/\n.env\n");
        sh(&f.local, &["add", "."]);
        commit(&f.local, "a.txt", "one");
        write(&f.local, ".env", "SECRET=1");
        write(&f.local, "node_modules/dep/i.js", "x");
        f
    }

    fn replica(&self) -> PathBuf {
        join(self.drive.path(), &rel_key(&self.local).unwrap())
    }

    fn state(&self) -> DriveProjectStatus {
        status(&self.local, self.drive.path()).unwrap()
    }

    fn push(&self) -> DriveSyncResult {
        let r = to_drive(&self.local, self.drive.path(), "proj", false, u64::MAX).unwrap();
        assert!(r.failures.is_empty(), "{:?}", r.failures);
        r
    }

    fn push_err(&self) -> String {
        to_drive(&self.local, self.drive.path(), "proj", false, u64::MAX).unwrap_err()
    }

    fn pull(&self) -> DriveSyncResult {
        let r = to_local(&self.local, self.drive.path()).unwrap();
        assert!(r.failures.is_empty(), "{:?}", r.failures);
        r
    }
}

/// #2177: a real file called `nul` used to fail with "Incorrect function".
#[cfg(windows)]
#[test]
fn a_file_with_a_device_name_is_copied() {
    let f = Fx::plain();
    fs::write(verbatim(&f.local.join("nul")), "data").unwrap();
    f.push();
    assert_eq!(fs::read_to_string(join(&f.replica(), "nul")).unwrap(), "data");
    assert_eq!(f.state().state, S::InSync);
}

#[test]
fn mirror_then_local_commit_round_trip() {
    let f = Fx::git();
    assert_eq!(f.state().state, S::NotOnDrive);
    let r = f.push();
    assert_eq!(r.status.state, S::InSync, "{:?}", r.status);
    assert!(r.files_copied > 0 && r.bytes_copied > 0);
    let rep = f.replica();
    assert_eq!(read(&rep, ".env"), "SECRET=1");
    assert!(!rep.join("node_modules").exists());

    commit(&f.local, "b.txt", "two");
    let s = f.state();
    assert_eq!((s.state, s.local_ahead), (S::DriveBehind, 1), "{s:?}");
    assert_eq!(f.push().status.state, S::InSync);
    assert_eq!(read(&rep, "b.txt"), "two");

    // A deletion on the PC is mirrored; a build folder made on the drive stays.
    write(&rep, "node_modules/built.js", "y");
    sh(&f.local, &["rm", "-q", "b.txt"]);
    sh(&f.local, &["commit", "-q", "-m", "rm"]);
    assert_eq!(f.push().status.state, S::InSync);
    assert!(!rep.join("b.txt").exists());
    assert!(rep.join("node_modules/built.js").exists());
}

#[test]
fn drive_commit_blocks_push_and_pulls() {
    let f = Fx::git();
    f.push();
    commit(&f.replica(), "d.txt", "drive");
    let s = f.state();
    assert_eq!((s.state, s.drive_ahead), (S::LocalBehind, 1), "{s:?}");
    assert!(f.push_err().contains("changes this PC does not have"));
    let r = f.pull();
    assert_eq!(r.status.state, S::InSync, "{r:?}");
    assert_eq!(read(&f.local, "d.txt"), "drive");
}

#[test]
fn diverged_merges_then_converges() {
    let f = Fx::git();
    f.push();
    commit(&f.replica(), "d.txt", "drive");
    commit(&f.local, "l.txt", "local");
    assert_eq!(f.state().state, S::Diverged);
    let r = f.pull();
    assert!(r.conflicts.is_empty(), "{r:?}");
    assert_eq!(r.status.state, S::DriveBehind, "{r:?}");
    assert_eq!(f.push().status.state, S::InSync);
    assert_eq!(read(&f.replica(), "l.txt"), "local");
}

/// Both sides edit one committed file; the merge differs from the drive's copy.
#[test]
fn auto_merged_file_still_converges() {
    let f = Fx::git();
    commit(&f.local, "m.txt", "1\n2\n3\n4\n5\n6\n7\n8\n9\n");
    f.push();
    commit(&f.replica(), "m.txt", "drive\n2\n3\n4\n5\n6\n7\n8\n9\n");
    commit(&f.local, "m.txt", "1\n2\n3\n4\n5\n6\n7\n8\nlocal\n");
    let r = f.pull();
    assert_eq!(r.status.state, S::DriveBehind, "{r:?}");
    assert_eq!(
        read(&f.local, "m.txt"),
        "drive\n2\n3\n4\n5\n6\n7\n8\nlocal\n"
    );
    assert_eq!(f.push().status.state, S::InSync);
}

#[test]
fn uncommitted_drive_edit_arrives() {
    let f = Fx::git();
    f.push();
    write(&f.replica(), "a.txt", "edited on the drive");
    assert_eq!(f.state().state, S::LocalBehind);
    let r = f.pull();
    assert_eq!(r.status.state, S::InSync, "{r:?}");
    assert_eq!(read(&f.local, "a.txt"), "edited on the drive");
    // The next one-sided edit must not look two-sided.
    write(&f.local, "a.txt", "then edited here, longer");
    assert_eq!(f.state().state, S::DriveBehind);
}

/// Work in progress goes to the drive, is committed there, and comes back.
#[test]
fn work_in_progress_committed_on_drive_fast_forwards() {
    let f = Fx::git();
    write(&f.local, "a.txt", "in progress");
    f.push();
    sh(&f.replica(), &["commit", "-q", "-am", "done on the drive"]);
    assert_eq!(f.state().state, S::LocalBehind);
    let r = f.pull();
    assert_eq!(r.status.state, S::InSync, "{r:?}");
    assert_eq!(
        sh(&f.local, &["rev-parse", "HEAD"]),
        sh(&f.replica(), &["rev-parse", "HEAD"])
    );
}

#[test]
fn drive_stash_and_detached_commit_are_carried() {
    let f = Fx::git();
    f.push();
    let rep = f.replica();
    write(&rep, "a.txt", "stashed on the drive");
    sh(&rep, &["stash", "push", "-q"]);
    sh(&rep, &["checkout", "-q", "--detach"]);
    commit(&rep, "detached.txt", "d");
    assert!(f.state().drive_ahead > 0);
    let r = f.pull();
    assert!(sh(&f.local, &["stash", "list"]).contains("from external drive"));
    assert!(sh(&f.local, &["branch"]).contains("drive-detached-"));
    assert_eq!(r.status.drive_ahead, 0, "{r:?}");
}

#[test]
fn non_git_conflict_leaves_local_alone() {
    let f = Fx::plain();
    write(&f.local, "n.txt", "base");
    write(&f.local, "node_modules/x", "x");
    f.push();
    assert!(!f.replica().join("node_modules").exists());
    write(&f.replica(), "n.txt", "drive version");
    write(&f.local, "n.txt", "the local version");
    assert_eq!(f.state().state, S::Diverged);
    let r = f.pull();
    assert!(r.conflicts.iter().any(|c| c.starts_with("n.txt")), "{r:?}");
    assert_eq!(read(&f.local, "n.txt"), "the local version");
    assert!(f.push_err().contains("changes this PC does not have"));
}

#[test]
fn nested_repo_drive_commit_blocks_push() {
    let f = Fx::git();
    let nested = f.local.join("libs/inner");
    init(&nested);
    commit(&nested, "i.txt", "inner");
    assert_eq!(f.push().status.state, S::InSync);

    let there = f.replica().join("libs/inner");
    commit(&there, "j.txt", "only on the drive");
    let sha = sh(&there, &["rev-parse", "HEAD"]);
    let s = f.state();
    assert!(s.drive_ahead >= 1, "{s:?}");
    assert!(f.push_err().contains("changes this PC does not have"));
    assert_eq!(sh(&there, &["rev-parse", "HEAD"]), sha);

    f.pull();
    assert_eq!(sh(&nested, &["rev-parse", "HEAD"]), sha);
    assert_eq!(read(&nested, "j.txt"), "only on the drive");
    assert_eq!(f.push().status.state, S::InSync);
}

#[test]
fn repo_added_or_removed_on_one_side() {
    let f = Fx::git();
    f.push();
    // Created on the drive: adopted here.
    let there = f.replica().join("tool");
    init(&there);
    commit(&there, "t.txt", "tool");
    assert!(f.state().drive_ahead >= 1);
    f.pull();
    assert_eq!(
        sh(&f.local.join("tool"), &["rev-parse", "HEAD"]),
        sh(&there, &["rev-parse", "HEAD"])
    );
    assert_eq!(f.push().status.state, S::InSync);

    // Removed here after a sync: not mistaken for new drive work.
    let mine = f.local.join("tool");
    for e in walkdir_files(&mine) {
        make_writable(&e);
    }
    fs::remove_dir_all(&mine).unwrap();
    assert_eq!(f.state().state, S::DriveBehind);
    f.push();
    assert!(!there.join(".git/HEAD").exists());
}

fn walkdir_files(dir: &Path) -> Vec<PathBuf> {
    jwalk::WalkDir::new(dir)
        .skip_hidden(false)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path())
        .collect()
}

#[test]
fn generic_folder_names_skip_only_when_ignored() {
    let f = Fx::plain();
    init(&f.local);
    write(&f.local, ".gitignore", "vendor/\n");
    write(&f.local, "vendor/v.js", "ignored");
    write(&f.local, "bin/run.sh", "not ignored");
    write(&f.local, "build/keep.txt", "strict name, not ignored");
    f.push();
    let rep = f.replica();
    assert!(!rep.join("vendor").exists());
    assert!(rep.join("bin/run.sh").exists());
    assert!(rep.join("build/keep.txt").exists());

    // Outside a repo the generic names are ordinary folders.
    let f = Fx::plain();
    write(&f.local, "vendor/v.js", "x");
    write(&f.local, "build/b.js", "x");
    f.push();
    assert!(f.replica().join("vendor/v.js").exists());
    assert!(!f.replica().join("build").exists());
}

#[test]
fn cachedir_tag_folder_is_skipped() {
    let f = Fx::plain();
    write(&f.local, "keep.txt", "k");
    write(&f.local, "out-of-tree/CACHEDIR.TAG", "Signature: x");
    write(&f.local, "out-of-tree/debug/obj.o", "o");
    f.push();
    assert!(f.replica().join("keep.txt").exists());
    assert!(!f.replica().join("out-of-tree").exists());
}

#[test]
fn unmanaged_folder_on_drive_is_refused() {
    let f = Fx::git();
    let rep = f.replica();
    write(&rep, "precious.txt", "mine");
    assert!(f.state().note.is_some());
    assert!(f.push_err().contains("was not created by Portal"));
    assert_eq!(read(&rep, "precious.txt"), "mine");
    assert!(!rep.join("a.txt").exists());
    // An empty folder is fine.
    fs::remove_file(rep.join("precious.txt")).unwrap();
    f.push();
}

#[test]
fn emptied_replica_is_not_a_mass_delete() {
    let f = Fx::plain();
    write(&f.local, "n.txt", "n");
    f.push();
    fs::remove_file(f.replica().join("n.txt")).unwrap();
    assert_eq!(f.state().state, S::NotOnDrive);
    assert!(to_local(&f.local, f.drive.path()).is_err());
    assert!(f.local.join("n.txt").exists());
}

#[test]
fn project_on_the_drive_itself_is_refused() {
    let f = Fx::plain();
    write(&f.local, "n.txt", "n");
    let around = f.local.parent().unwrap();
    assert!(to_drive(&f.local, around, "p", true, u64::MAX).is_err());
    assert!(to_drive(&f.local, &f.local, "p", true, u64::MAX).is_err());
    assert_eq!(read(&f.local, "n.txt"), "n");
}

#[test]
fn not_enough_space_writes_nothing() {
    let f = Fx::plain();
    write(&f.local, "n.txt", "more than one byte");
    let e = to_drive(&f.local, f.drive.path(), "p", false, 1).unwrap_err();
    assert!(e.contains("Not enough space"), "{e}");
    assert!(!f.replica().exists());
}

#[cfg(windows)]
#[test]
fn uncopyable_file_is_reported_without_leaving_the_project_half_done() {
    use std::os::windows::fs::OpenOptionsExt;
    let f = Fx::plain();
    write(&f.local, "locked.db", "v1");
    write(&f.local, "gone.txt", "g");
    f.push();
    fs::remove_file(f.local.join("gone.txt")).unwrap();
    write(&f.local, "locked.db", "version two");
    write(&f.local, "z.txt", "z");
    let hold = fs::File::options()
        .read(true)
        .share_mode(0)
        .open(f.local.join("locked.db"))
        .unwrap();
    let r = to_drive(&f.local, f.drive.path(), "p", false, u64::MAX).unwrap();
    assert_eq!(r.failures.len(), 1, "{r:?}");
    assert_eq!(read(&f.replica(), "z.txt"), "z");
    assert_eq!(read(&f.replica(), "locked.db"), "v1");
    assert!(f.replica().join("gone.txt").exists());
    assert!(load_manifest(f.drive.path())
        .unwrap()
        .projects
        .values()
        .all(|e| !e.in_progress && e.synced_at.is_some() && e.failed.len() == 1));
    // Not cleanly in sync, and nothing to pull back.
    assert_eq!(
        (r.status.state, r.status.drive_only_files),
        (S::DriveBehind, 0),
        "{r:?}"
    );
    assert!(r.status.note.unwrap().contains("locked.db"));
    to_local(&f.local, f.drive.path()).unwrap();
    assert!(!f.local.join("gone.txt").exists());
    drop(hold);
    let s = f.push().status;
    assert_eq!((s.state, s.note), (S::InSync, None));
    assert!(!f.replica().join("gone.txt").exists());
}

#[test]
fn nested_project_is_covered_unless_the_walk_skips_it() {
    let f = Fx::git();
    write(&f.local, "apps/web/x", "x");
    let parent = vec![("outer".to_string(), f.local.clone())];
    let inner = f.local.join("apps/web");
    assert_eq!(covered_by(&inner, &parent), Some("outer".into()));
    let skipped = f.local.join("node_modules/dep");
    assert_eq!(covered_by(&skipped, &parent), None);
    assert_eq!(covered_by(&f.local, &parent), None);
}

#[test]
fn hostile_manifest_folder_is_rejected() {
    let f = Fx::git();
    for bad in ["../x", "a/../../x", ".portal-sync/y", "/abs"] {
        write(
            f.drive.path(),
            ".portal-sync/portal-sync.json",
            &format!(r#"{{"version":1,"projects":{{"{bad}":{{"name":"p","sourcePath":"x"}}}}}}"#),
        );
        let e = status(&f.local, f.drive.path()).unwrap_err();
        assert!(e.contains("unsafe project folder"), "{bad}: {e}");
        assert!(to_drive(&f.local, f.drive.path(), "p", true, u64::MAX).is_err());
    }
    // A corrupt manifest is reported, never overwritten.
    write(f.drive.path(), ".portal-sync/portal-sync.json", "{nope");
    assert!(to_drive(&f.local, f.drive.path(), "p", true, u64::MAX).is_err());
    assert_eq!(
        read(f.drive.path(), ".portal-sync/portal-sync.json"),
        "{nope"
    );
}

#[test]
fn another_pcs_base_is_not_used() {
    let f = Fx::plain();
    write(&f.local, "n.txt", "one");
    f.push();
    write(&f.replica(), "n.txt", "changed elsewhere");
    // Pretend the base was written by a different machine.
    let key = rel_key(&f.local).unwrap();
    let mut state = load_state(f.drive.path(), &key);
    let base = state.remove(&owner(&f.local)).unwrap();
    state.insert("other-pc|x".into(), base);
    write_meta(f.drive.path(), &state_file(&key), &state).unwrap();
    assert_eq!(f.state().state, S::Diverged);
    assert!(f.push_err().contains("changes this PC does not have"));
}

#[test]
fn a_vanished_drive_or_source_deletes_nothing() {
    let f = Fx::plain();
    write(&f.local, "n.txt", "n");
    f.push();
    let gone = f.drive.path().join("not-there");
    assert!(compare(&f.local, &gone, &[], None).is_err());
    // An emptied project folder never clears the drive, even when forced.
    fs::remove_file(f.local.join("n.txt")).unwrap();
    assert!(to_drive(&f.local, f.drive.path(), "p", true, u64::MAX).is_err());
    assert_eq!(read(&f.replica(), "n.txt"), "n");
}

/// A submodule's `.git` is a file with a relative target.
#[test]
fn submodule_drive_commit_blocks_push() {
    let f = Fx::git();
    let other = tempfile::tempdir().unwrap();
    init(other.path());
    commit(other.path(), "s.txt", "sub");
    sh(
        &f.local,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &slashes(other.path()),
            "sub",
        ],
    );
    sh(&f.local, &["commit", "-q", "-m", "add sub"]);
    let sub = f.local.join("sub");
    assert!(sub.join(".git").is_file());
    assert_eq!(f.push().status.state, S::InSync);

    let there = f.replica().join("sub");
    sh(&there, &["config", "user.name", "t"]);
    sh(&there, &["config", "user.email", "t@t"]);
    sh(&there, &["config", "commit.gpgsign", "false"]);
    sh(&there, &["config", "core.autocrlf", "false"]);
    sh(&there, &["checkout", "-q", "--detach"]);
    commit(&there, "t.txt", "only on the drive");
    let sha = sh(&there, &["rev-parse", "HEAD"]);
    assert!(f.state().drive_ahead >= 1);
    assert!(f.push_err().contains("changes this PC does not have"));

    let r = f.pull();
    assert_eq!(r.status.drive_ahead, 0, "{r:?}");
    assert_eq!(sh(&sub, &["cat-file", "-t", &sha]), "commit");
}

fn after_walk(change: impl FnOnce() + 'static) {
    AFTER_WALK.set(Some(Box::new(change)));
}

fn all_in_progress(f: &Fx) -> bool {
    let m = load_manifest(f.drive.path()).unwrap();
    m.projects.values().all(|e| e.in_progress)
}

/// Commits landing mid-copy, one of them from a linked worktree.
#[test]
fn commits_made_during_the_copy_arrive_whole() {
    let f = Fx::git();
    let wt = f.local.parent().unwrap().join("wt");
    sh(
        &f.local,
        &["worktree", "add", "-q", "-b", "side", &slashes(&wt)],
    );
    let (local, other) = (f.local.clone(), wt.clone());
    after_walk(move || {
        commit(&local, "b.txt", "during");
        commit(&other, "w.txt", "from the worktree");
    });
    let r = f.push();
    assert_eq!(r.status.state, S::DriveBehind, "{r:?}");
    let rep = f.replica();
    for branch in ["main", "side"] {
        assert_eq!(
            sh(&rep, &["rev-parse", branch]),
            sh(&f.local, &["rev-parse", branch])
        );
    }
    fsck(&rep, &f.local).unwrap();
    assert_eq!(f.push().status.state, S::InSync);
}

/// `git gc` mid-copy: loose refs and objects move into packs.
#[test]
fn gc_during_the_copy_is_tolerated() {
    let f = Fx::git();
    f.push();
    commit(&f.local, "b.txt", "two");
    let local = f.local.clone();
    after_walk(move || {
        sh(&local, &["gc", "-q", "--prune=now"]);
    });
    let r = f.push();
    assert_eq!(r.status.state, S::InSync, "{r:?}");
    assert_eq!(
        sh(&f.replica(), &["rev-parse", "main"]),
        sh(&f.local, &["rev-parse", "main"])
    );
    fsck(&f.replica(), &f.local).unwrap();
}

/// A commit on the empty tree, which git never has to store: fsck calls it
/// missing in the source too, so the copy lost nothing.
#[test]
fn empty_tree_missing_in_the_source_too_is_not_a_failure() {
    let f = Fx::git();
    let c = sh(
        &f.local,
        &[
            "commit-tree",
            "4b825dc642cb6eb9a060e54bf8d69288fbee4904",
            "-m",
            "empty",
        ],
    );
    sh(&f.local, &["update-ref", "refs/heads/empty", &c]);
    let r = to_drive(&f.local, f.drive.path(), "proj", false, u64::MAX).unwrap();
    assert!(r.failures.is_empty(), "{r:?}");
    assert_eq!(r.status.state, S::InSync, "{r:?}");
}

#[test]
fn replica_with_a_missing_object_is_not_synced() {
    let f = Fx::git();
    let blob = sh(&f.local, &["rev-parse", "HEAD:a.txt"]);
    let object = f
        .local
        .join(".git/objects")
        .join(&blob[..2])
        .join(&blob[2..]);
    remove_file(&object).unwrap();
    let r = to_drive(&f.local, f.drive.path(), "proj", false, u64::MAX).unwrap();
    assert!(
        r.failures.iter().any(|e| e.contains("git fsck failed")),
        "{r:?}"
    );
    assert_ne!(r.status.state, S::InSync);
    assert!(all_in_progress(&f));
    assert!(to_local(&f.local, f.drive.path()).is_err());
}

#[test]
fn unfinished_copy_sees_drive_edits_and_refuses_what_it_cannot_compare() {
    let f = Fx::git();
    f.push();
    let mut m = load_manifest(f.drive.path()).unwrap();
    m.projects.values_mut().for_each(|e| e.in_progress = true);
    write_meta(f.drive.path(), MANIFEST, &m).unwrap();
    assert_eq!(f.state().state, S::DriveBehind);

    write(&f.replica(), "a.txt", "edited on the drive");
    assert_eq!(f.state().state, S::Diverged);
    assert!(f.push_err().contains("changes this PC does not have"));

    write(&f.replica(), ".git/config", "[broken");
    let s = f.state();
    assert_eq!(s.state, S::Error, "{s:?}");
    assert!(f.push_err().contains("cannot be checked"));
    // Overwriting is the way out.
    let r = to_drive(&f.local, f.drive.path(), "proj", true, u64::MAX).unwrap();
    assert_eq!(r.status.state, S::InSync, "{r:?}");
}

#[test]
fn a_status_error_is_not_reported_as_drive_behind() {
    let f = Fx::git();
    let s = final_status(&f.local, &f.drive.path().join("unplugged"));
    assert_eq!(s.state, S::Error);
    assert!(s.note.is_some());
}

/// A project under its parent's ignored `build/`, copied in either order.
#[test]
fn project_the_parent_skips_keeps_its_own_copy() {
    for child_first in [true, false] {
        let f = Fx::git();
        write(&f.local, ".gitignore", "build/\n");
        sh(&f.local, &["commit", "-q", "-am", "ignore build"]);
        let child = f.local.join("build/child");
        write(&child, "c.txt", "child");
        let drive = f.drive.path();
        let push_child = || to_drive(&child, drive, "child", false, u64::MAX).unwrap();
        let there = f.replica().join("build/child/c.txt");

        if child_first {
            assert!(push_child().failures.is_empty());
            let s = f.state();
            assert_eq!((s.state, s.note), (S::NotOnDrive, None));
            f.push();
        } else {
            f.push();
            assert!(push_child().failures.is_empty());
        }
        commit(&f.local, "b.txt", "two");
        assert_eq!(f.push().status.state, S::InSync);
        assert_eq!(fs::read_to_string(&there).unwrap(), "child");
        assert_eq!(status(&child, drive).unwrap().state, S::InSync);
        // The parent never takes the child's files for its own drive changes.
        f.pull();
        assert_eq!(f.state().state, S::InSync);
    }
}

/// An archived parent no longer covers, though its copy still holds the folder.
#[test]
fn project_inside_another_copy_is_taken_over_only_without_loss() {
    let f = Fx::git();
    write(&f.local, "apps/web/x", "x");
    f.push();
    let child = f.local.join("apps/web");
    let push_child = || to_drive(&child, f.drive.path(), "web", false, u64::MAX);
    write(&f.replica(), "apps/web/x", "edited on the drive");
    let e = push_child().unwrap_err();
    assert!(e.contains("changes this PC does not have"), "{e}");
    assert_eq!(read(&f.replica(), "apps/web/x"), "edited on the drive");

    write(&child, "x", "edited on the drive");
    assert_eq!(push_child().unwrap().status.state, S::InSync);
}

#[test]
fn only_tracked_files_leave_an_ignored_folder() {
    let f = Fx::git();
    write(&f.local, ".gitignore", "build/\n");
    write(&f.local, "build/icons/kept.txt", "tracked");
    sh(
        &f.local,
        &["add", "-f", ".gitignore", "build/icons/kept.txt"],
    );
    sh(&f.local, &["commit", "-q", "-m", "tracked in build"]);
    write(&f.local, "build/junk.o", "o");
    write(&f.local, "build/icons/junk.o", "o");
    assert_eq!(f.push().status.state, S::InSync);
    let rep = f.replica();
    assert_eq!(read(&rep, "build/icons/kept.txt"), "tracked");
    assert!(!rep.join("build/junk.o").exists());
    assert!(!rep.join("build/icons/junk.o").exists());
}

#[test]
fn a_failing_check_ignore_never_copies_the_folder() {
    let f = Fx::git();
    write(&f.local, "build/b.o", "o");
    write(&f.local, ".git/config", "[broken");
    let w = walk(&f.local, false).unwrap();
    assert!(w.errors.contains_key("build"), "{:?}", w.errors);
    assert!(!w.files.contains_key("build/b.o"));
}

#[test]
fn busy_repo_is_refused_and_git_failure_is_not_free() {
    let f = Fx::git();
    f.push();
    assert_eq!(busy(&f.local), Ok(None));
    fs::create_dir(f.local.join(".git/rebase-merge")).unwrap();
    assert_eq!(busy(&f.local), Ok(Some("rebase in progress")));
    let e = to_local(&f.local, f.drive.path()).unwrap_err();
    assert!(e.contains("rebase in progress"), "{e}");
    assert!(busy(Fx::plain().local.as_path()).is_err());
}

#[test]
fn a_dangling_origin_head_does_not_block_the_sync() {
    let f = Fx::git();
    let refs = ["symbolic-ref", "refs/remotes/origin/HEAD"];
    sh(&f.local, &[refs[0], refs[1], "refs/remotes/origin/renamed"]);
    assert_eq!(f.push().status.state, S::InSync);
    assert!(!all_in_progress(&f));
}

fn mtime(p: &Path) -> std::time::SystemTime {
    fs::metadata(p).unwrap().modified().unwrap()
}

fn set_mtime(p: &Path, to: std::time::SystemTime) {
    let f = fs::File::options().write(true).open(p).unwrap();
    f.set_modified(to).unwrap();
}

#[test]
fn same_size_edit_moments_later_is_copied() {
    let f = Fx::plain();
    write(&f.local, "a.txt", "one");
    f.push();
    let was = mtime(&f.local.join("a.txt"));
    write(&f.local, "a.txt", "two");
    let soon = was + std::time::Duration::from_millis(100);
    set_mtime(&f.local.join("a.txt"), soon);
    assert_eq!(f.state().state, S::DriveBehind);
    f.push();
    assert_eq!(read(&f.replica(), "a.txt"), "two");
}

#[test]
fn pulled_file_gets_a_current_mtime_and_reads_in_sync() {
    let f = Fx::plain();
    write(&f.local, "a.txt", "one");
    f.push();
    write(&f.replica(), "a.txt", "from the drive");
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    set_mtime(&f.replica().join("a.txt"), old);
    assert_eq!(f.pull().status.state, S::InSync);
    assert!(mtime(&f.local.join("a.txt")) > old + std::time::Duration::from_secs(1800));
    assert_eq!(f.state().state, S::InSync);
}

#[test]
fn leftover_temp_copies_are_swept() {
    let f = Fx::git();
    f.push();
    let left = format!("a.txt{TMP}");
    let in_git = format!(".git/HEAD{TMP}");
    write(&f.local, &left, "half");
    write(&f.replica(), &left, "half");
    write(&f.replica(), &in_git, "half");
    f.push();
    assert!(!f.local.join(&left).exists());
    assert!(!f.replica().join(&left).exists());
    assert!(!f.replica().join(&in_git).exists());
    write(&f.local, &left, "half");
    f.pull();
    assert!(!f.local.join(&left).exists());
}

#[test]
fn a_second_apply_on_the_drive_is_refused() {
    let f = Fx::plain();
    write(&f.local, "a.txt", "one");
    let held = lock_drive(f.drive.path()).unwrap();
    assert!(f.push_err().contains("Another Portal"));
    drop(held);
    f.push();
}

#[test]
fn failed_fast_forward_leaves_the_index_as_it_was() {
    let f = Fx::git();
    commit(&f.local, "b.txt", "one");
    sh(&f.local, &["checkout", "-q", "-b", "next"]);
    commit(&f.local, "a.txt", "incoming a");
    commit(&f.local, "b.txt", "incoming b");
    sh(&f.local, &["checkout", "-q", "main"]);
    // `a.txt` already matches what is coming; `b.txt` blocks the merge.
    write(&f.local, "a.txt", "incoming a");
    write(&f.local, "b.txt", "mine");
    let index = || sh(&f.local, &["ls-files", "-s"]);
    let before = index();
    assert!(fast_forward(&f.local, "next").is_err());
    assert_eq!(index(), before);

    // A different version the user staged is not replaced.
    write(&f.local, "a.txt", "staged by hand");
    sh(&f.local, &["add", "a.txt"]);
    write(&f.local, "a.txt", "incoming a");
    let before = index();
    assert!(fast_forward(&f.local, "next").is_err());
    assert_eq!(index(), before);
}

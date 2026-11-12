//! The move engine: copy a project's directory to a new location, verify it
//! byte-for-byte, promote it into place, repoint the database, and only ever
//! remove the original on a separate, explicitly confirmed step.
//!
//! Step 4 of the "safe project relocation" plan. It never does an in-place
//! `rename` of the project tree — rename can't skip regenerable dirs and can't
//! keep the original until the copy is proven good — so it always does a
//! selective copy + hash-verify.
//!
//! ## Crash safety
//!
//! A move is journaled in `project_moves` and leaves two sidecar files under
//! `moves/<id>.*`: a **manifest** (the file set captured at the start) and an
//! append-only **progress log** (one `{rel, sha256}` line per file, fsynced).
//! A `.portal-move` marker inside the staging dir ties the staging/destination
//! tree back to its journal row. After a crash the move is still in
//! `copying`/`verifying`; [`Mover::resume_move`] re-checks the source against
//! the manifest and every logged hash and either resumes or refuses (leaving
//! rollback the only option). The progress log saves re-copying work — it is
//! never treated as proof. The final verify re-hashes *every* destination file
//! against the live source regardless of what the log says.
//!
//! ## Seams for later steps
//!
//! - `#217` source hold: before the manifest is built we would take deny-write
//!   handles on the source tree and drop them before the commit/trash step.
//!   Marked `TODO(#217)` at the call sites; until then the post-copy re-walk in
//!   [`final_verify`] is the backstop against the source changing mid-move.
//! - `#221` stored-path rewrite: the commit repoints every project row under
//!   the source (the moved project and any nested registered ones). Rewriting
//!   absolute paths buried in deployment/pipeline config is `TODO(#221)`; those
//!   rows get listed in `project_moves.unrewritten` for the UI.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};
use std::process::Command;
use std::time::UNIX_EPOCH;

use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::database::DatabaseManager;
use crate::domains::coder::CoderService;
use crate::domains::deployments::repositories::deployment_repository::DeploymentRepository;
use crate::domains::projects::discovery::scan::REGENERABLE;
use crate::domains::projects::move_lock::{self, is_under};
use crate::domains::projects::pipelines::services::ExecutionService;
use crate::domains::projects::relocation::usage::{self, UsageReport};
use crate::domains::scripts::ScriptExecutionService;
use crate::domains::terminal::manager::TerminalManager;
use crate::entities::{project as project_entity, project_move};
use crate::process_ext::NoWindowExt;

/// Marker file written into the staging dir (and carried into the destination
/// by the promote rename) holding the move id. Lets recovery tell "this tree
/// was produced by move X" from a stale leftover.
const SIDECAR: &str = ".portal-move";

/// Bumped only on a breaking change to the manifest format. A manifest written
/// by another version is not trusted for resume.
const MANIFEST_VERSION: i32 = 1;

/// Read/hash buffer. 64 KiB is the usual sweet spot for sequential file I/O.
const BUF: usize = 64 * 1024;

/// Journal the row-counter update every this-many files rather than per file —
/// a project can hold 100k files and the row is only informational (resume
/// reads the progress log, not these counters).
const COUNTER_EVERY: u64 = 256;

// ── Types ────────────────────────────────────────────────────────────────────

/// One file in the move set. `mtime` is recorded for diagnostics and future
/// change-detection; the gates compare `len` and re-hash content, so a bare
/// `touch` that leaves content intact does not fail a move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ManifestEntry {
    rel: String,
    len: u64,
    mtime: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
    version: i32,
    /// Regenerable dirs skipped at capture (rel, forward slashes). Carried in
    /// the manifest so the verify/resume re-walk prunes exactly the same dirs
    /// deterministically, instead of re-asking `git check-ignore` — whose
    /// answer can flip mid-move and fail a genuinely-good copy.
    skipped_dirs: Vec<String>,
    entries: Vec<ManifestEntry>,
}

#[derive(Serialize, Deserialize)]
struct ProgressLine {
    rel: String,
    sha256: String,
}

/// Emitted as the copy/verify runs so the command layer can throttle it onto a
/// Tauri event. Serialize-able so that layer can forward it as-is.
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub id: String,
    /// `copying` | `verifying`
    pub phase: String,
    pub files_done: u64,
    pub file_count: u64,
    pub bytes_done: u64,
    pub total_bytes: u64,
}

/// Why [`Mover::start_move`]/[`Mover::resume_move`] didn't complete.
#[derive(Debug)]
pub enum MoveError {
    /// Refused: the usage audit found blockers, or warnings the caller didn't
    /// acknowledge. The report carries the detail for the UI.
    Blocked(UsageReport),
    /// Everything else — an I/O error, a hash mismatch, a refused resume.
    Failed(String),
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::Blocked(r) => write!(f, "blocked by {} signal(s)", r.blockers.len()),
            MoveError::Failed(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for MoveError {}

type ProgressFn<'a> = &'a mut dyn FnMut(Progress);

// ── Engine ───────────────────────────────────────────────────────────────────

/// Services the engine needs for the in-use audit and the journal. Bundled so
/// the public methods don't each take the ten-argument [`usage::check`] list.
pub struct Mover<'a> {
    pub db: &'a DatabaseManager,
    pub coder: &'a CoderService,
    pub terminal: &'a TerminalManager,
    pub pipelines: &'a ExecutionService,
    pub scripts: &'a ScriptExecutionService,
    pub deployments: &'a DeploymentRepository,
    /// Directory the manifest and progress files live under (app-data `moves/`).
    pub moves_dir: PathBuf,
}

impl Mover<'_> {
    async fn usage(&self, source: &Path, dest: &Path, dev: Option<u16>, prod: Option<u16>) -> UsageReport {
        usage::check(
            source,
            dest,
            self.db,
            self.coder,
            self.terminal,
            self.pipelines,
            self.scripts,
            self.deployments,
            dev,
            prod,
        )
        .await
    }

    /// In-use blockers for an already-chosen path (resume, source removal).
    async fn in_use(&self, path: &Path) -> Vec<usage::Signal> {
        usage::check_in_use(
            path,
            self.coder,
            self.terminal,
            self.pipelines,
            self.scripts,
            self.deployments,
        )
        .await
    }

    /// Start a fresh move of `project` to `dest`. Takes the source + dest lock
    /// for the whole run (released on return, success or failure), runs the
    /// usage audit, captures the manifest, journals the row, then copies and
    /// verifies. The original is left untouched; [`Mover::remove_moved_source`]
    /// deletes it later.
    pub async fn start_move(
        &self,
        project: &project_entity::Model,
        dest: &Path,
        include_regenerable: bool,
        acknowledge_warnings: bool,
        on_progress: ProgressFn<'_>,
    ) -> Result<project_move::Model, MoveError> {
        let source = PathBuf::from(&project.path);
        let dest = dest.to_path_buf();
        let _guard = LockGuard::new(vec![source.clone(), dest.clone()]);

        let report = self
            .usage(&source, &dest, port(project.dev_port), port(project.prod_port))
            .await;
        if !report.blockers.is_empty() || (!acknowledge_warnings && !report.warnings.is_empty()) {
            return Err(MoveError::Blocked(report));
        }

        // TODO(#217): take the deny-write source hold here, drop it before the
        // commit. Until then final_verify's re-walk is the backstop.
        let (entries, skipped) = build_manifest(&source, include_regenerable).map_err(MoveError::Failed)?;

        let id = Uuid::new_v4().to_string();
        let staging = staging_path(&dest, &id).map_err(MoveError::Failed)?;
        let manifest_path = self.moves_dir.join(format!("{id}.manifest.json"));
        let progress_path = self.moves_dir.join(format!("{id}.progress.jsonl"));

        fs::create_dir_all(&self.moves_dir).map_err(|e| MoveError::Failed(format!("create moves dir: {e}")))?;
        let manifest = Manifest {
            version: MANIFEST_VERSION,
            skipped_dirs: skipped,
            entries,
        };
        let manifest_sha = write_manifest(&manifest_path, &manifest).map_err(MoveError::Failed)?;

        fs::create_dir_all(&staging).map_err(|e| MoveError::Failed(format!("create staging: {e}")))?;
        write_sidecar(&staging, &id).map_err(MoveError::Failed)?;
        fs::File::create(&progress_path).map_err(|e| MoveError::Failed(format!("create progress log: {e}")))?;

        let total_bytes: i64 = manifest.entries.iter().map(|e| e.len as i64).sum();
        let now = now();
        let row = project_move::ActiveModel {
            id: Set(id.clone()),
            project_id: Set(project.id),
            source_path: Set(source.to_string_lossy().into_owned()),
            dest_path: Set(dest.to_string_lossy().into_owned()),
            staging_path: Set(staging.to_string_lossy().into_owned()),
            state: Set("copying".into()),
            include_regenerable: Set(include_regenerable),
            skipped_dirs: Set(serde_json::to_string(&manifest.skipped_dirs).unwrap_or_else(|_| "[]".into())),
            manifest_path: Set(Some(manifest_path.to_string_lossy().into_owned())),
            manifest_sha256: Set(Some(manifest_sha)),
            manifest_version: Set(Some(MANIFEST_VERSION)),
            progress_path: Set(Some(progress_path.to_string_lossy().into_owned())),
            file_count: Set(manifest.entries.len() as i32),
            total_bytes: Set(total_bytes),
            files_done: Set(0),
            bytes_done: Set(0),
            resume_count: Set(0),
            unrewritten: Set("[]".into()),
            error: Set(None),
            started_at: Set(now.clone()),
            updated_at: Set(now),
            finished_at: Set(None),
        }
        .insert(self.db.get_connection())
        .await
        .map_err(|e| MoveError::Failed(format!("journal move: {e}")))?;

        match self.run_move(&row, &manifest, on_progress).await {
            Ok(done) => Ok(done),
            Err(e) => {
                // Leave the state in copying/verifying so a later resume can
                // pick it up; just record what went wrong.
                let _ = mark_error(self.db, &id, &e).await;
                Err(MoveError::Failed(e))
            }
        }
    }

    /// Resume an interrupted move. Refuses — leaving rollback the only option —
    /// unless the staging sidecar, the manifest, the live source, and every
    /// logged file's source hash all still match what was journaled.
    pub async fn resume_move(
        &self,
        move_id: &str,
        on_progress: ProgressFn<'_>,
    ) -> Result<project_move::Model, MoveError> {
        let row = load_row(self.db, move_id).await.map_err(MoveError::Failed)?;
        if !matches!(row.state.as_str(), "copying" | "verifying") {
            return Err(MoveError::Failed(format!("move is {}, not resumable", row.state)));
        }
        let source = PathBuf::from(&row.source_path);
        let staging = PathBuf::from(&row.staging_path);
        let _guard = LockGuard::new(vec![source.clone(), PathBuf::from(&row.dest_path)]);

        // In-use check only — the move's own in-flight row and its chosen dest
        // would trip the start-only preconditions.
        let blockers = self.in_use(&source).await;
        if !blockers.is_empty() {
            return Err(MoveError::Blocked(UsageReport { blockers, warnings: vec![] }));
        }

        let manifest = read_validated_manifest(&row).map_err(MoveError::Failed)?;
        let logged = read_progress(&progress_path_of(&row).map_err(MoveError::Failed)?)
            .map_err(MoveError::Failed)?;
        can_resume(&row.id, &source, &staging, &manifest, &logged).map_err(MoveError::Failed)?;

        let _ = bump_resume_count(self.db, &row.id).await;
        match self.run_move(&row, &manifest, on_progress).await {
            Ok(done) => Ok(done),
            Err(e) => {
                let _ = mark_error(self.db, &row.id, &e).await;
                Err(MoveError::Failed(e))
            }
        }
    }

    /// The copy → verify → promote → commit core, shared by fresh and resumed
    /// moves. On entry the row is in `copying`, the manifest/progress files and
    /// staging dir exist.
    async fn run_move(
        &self,
        row: &project_move::Model,
        manifest: &Manifest,
        on_progress: ProgressFn<'_>,
    ) -> Result<project_move::Model, String> {
        let source = PathBuf::from(&row.source_path);
        let dest = PathBuf::from(&row.dest_path);
        let staging = PathBuf::from(&row.staging_path);
        let progress_path = progress_path_of(row)?;
        let file_count = manifest.entries.len() as u64;
        let total_bytes: u64 = manifest.entries.iter().map(|e| e.len).sum();

        // The sidecar should already be there; rewrite it so a resume that lost
        // it (e.g. a half-made staging) is anchored before we copy into it.
        write_sidecar(&staging, &row.id)?;

        // ── Copy ──
        let logged = read_progress(&progress_path)?;
        let (to_copy, mut files_done, mut bytes_done) =
            plan_copies(&manifest.entries, &staging, &logged)?;

        for &i in &to_copy {
            let e = &manifest.entries[i];
            let sha = copy_entry(&source, &staging, e)?;
            if !logged.contains_key(&e.rel) {
                append_progress(&progress_path, &e.rel, &sha)?;
            }
            files_done += 1;
            bytes_done += e.len;
            on_progress(Progress {
                id: row.id.clone(),
                phase: "copying".into(),
                files_done,
                file_count,
                bytes_done,
                total_bytes,
            });
            if files_done % COUNTER_EVERY == 0 {
                let _ = update_counts(self.db, &row.id, files_done, bytes_done).await;
            }
        }
        let _ = update_counts(self.db, &row.id, files_done, bytes_done).await;

        // ── Verify ──
        update_state(self.db, &row.id, "verifying").await?;
        on_progress(Progress {
            id: row.id.clone(),
            phase: "verifying".into(),
            files_done,
            file_count,
            bytes_done,
            total_bytes,
        });
        final_verify(&source, &staging, manifest)?;
        update_state(self.db, &row.id, "verified").await?;

        // ── Promote ── (atomic: staging is a sibling of dest, same volume)
        fs::rename(&staging, &dest)
            .map_err(|e| format!("promote {} -> {}: {e}", staging.display(), dest.display()))?;

        // ── Commit ── repoint the moved project (and any nested ones) to dest.
        self.commit(&row.id, &row.source_path, &row.dest_path).await?;

        // The marker has done its job; a committed/done move needs no recovery.
        let _ = fs::remove_file(dest.join(SIDECAR));

        // TODO(#221): rebase absolute paths inside deployment/pipeline config;
        // append the rows it can't fix to project_moves.unrewritten here.

        update_done(self.db, &row.id).await?;
        load_row(self.db, &row.id).await
    }

    /// One transaction: flip the move to `committed` and rebase every project
    /// row whose path sits under the old source onto the new destination.
    async fn commit(&self, move_id: &str, source: &str, dest: &str) -> Result<(), String> {
        let txn = self
            .db
            .get_connection()
            .begin()
            .await
            .map_err(|e| format!("begin commit txn: {e}"))?;

        project_move::ActiveModel {
            id: Set(move_id.to_string()),
            state: Set("committed".into()),
            updated_at: Set(now()),
            ..Default::default()
        }
        .update(&txn)
        .await
        .map_err(|e| format!("mark committed: {e}"))?;

        let projects = project_entity::Entity::find()
            .all(&txn)
            .await
            .map_err(|e| format!("load projects for rebase: {e}"))?;
        for p in projects {
            if let Some(new_path) = rebased_path(&p.path, source, dest) {
                let mut am: project_entity::ActiveModel = p.into();
                am.path = Set(new_path);
                am.update(&txn).await.map_err(|e| format!("rebase project path: {e}"))?;
            }
        }

        txn.commit().await.map_err(|e| format!("commit txn: {e}"))
    }

    /// Delete the original, after the move is committed. Re-runs the in-use
    /// audit, confirms the source still matches the manifest and the
    /// destination is a byte-for-byte copy, then sends the source to the
    /// Recycle Bin. Refuses on any mismatch.
    pub async fn remove_moved_source(&self, move_id: &str) -> Result<(), MoveError> {
        let row = load_row(self.db, move_id).await.map_err(MoveError::Failed)?;
        if !matches!(row.state.as_str(), "committed" | "done") {
            return Err(MoveError::Failed(format!(
                "move is {}, not committed — refusing to delete the source",
                row.state
            )));
        }
        let source = PathBuf::from(&row.source_path);
        let dest = PathBuf::from(&row.dest_path);
        let _guard = LockGuard::new(vec![source.clone()]);

        let blockers = self.in_use(&source).await;
        if !blockers.is_empty() {
            return Err(MoveError::Blocked(UsageReport { blockers, warnings: vec![] }));
        }

        let manifest = read_validated_manifest(&row).map_err(MoveError::Failed)?;
        let current = rewalk(&source, &manifest.skipped_dirs).map_err(MoveError::Failed)?;
        if !same_set(&current, &manifest.entries) {
            return Err(MoveError::Failed(
                "source changed since the move — refusing to delete it".into(),
            ));
        }
        for e in &manifest.entries {
            let src_h = hash_file(&os_join(&source, &e.rel)).map_err(MoveError::Failed)?;
            let dst_h = hash_file(&os_join(&dest, &e.rel)).map_err(MoveError::Failed)?;
            if src_h != dst_h {
                return Err(MoveError::Failed(format!(
                    "destination differs from source at {} — refusing to delete the source",
                    e.rel
                )));
            }
        }

        trash::delete(&source).map_err(|e| MoveError::Failed(format!("recycle source: {e}")))?;
        Ok(())
    }

    /// Finish a move whose copy crashed between the promote (rename) and the
    /// commit, or between commit and the `done` bookkeeping. The destination is
    /// already in place, so there is nothing to copy — just (re)apply the
    /// commit (idempotent) and mark it done. `#206` routes `verified`/
    /// `committed` rows here. Refuses a `verified` row whose destination isn't
    /// the one this move produced.
    pub async fn finish_move(&self, move_id: &str) -> Result<project_move::Model, MoveError> {
        let row = load_row(self.db, move_id).await.map_err(MoveError::Failed)?;
        let dest = PathBuf::from(&row.dest_path);
        let _guard = LockGuard::new(vec![PathBuf::from(&row.source_path), dest.clone()]);

        match row.state.as_str() {
            // Promote done, commit not: the dest must carry this move's marker.
            "verified" => {
                if read_sidecar(&dest).as_deref() != Some(row.id.as_str()) {
                    return Err(MoveError::Failed(
                        "destination was not produced by this move — roll back instead".into(),
                    ));
                }
            }
            // Commit is already durable; nothing to prove, just finish.
            "committed" => {}
            s => return Err(MoveError::Failed(format!("move is {s}, cannot finish"))),
        }

        self.commit(&row.id, &row.source_path, &row.dest_path)
            .await
            .map_err(MoveError::Failed)?;
        let _ = fs::remove_file(dest.join(SIDECAR));
        // TODO(#221): stored-path rewrite seam, as in run_move.
        update_done(self.db, &row.id).await.map_err(MoveError::Failed)?;
        load_row(self.db, &row.id).await.map_err(MoveError::Failed)
    }

    /// Roll an interrupted move back: delete the staging tree (only if its
    /// sidecar id matches) and mark the move aborted. The source was never
    /// touched, so there is nothing to restore.
    ///
    /// Refuses a `verified` move whose destination already carries this move's
    /// marker — the promote happened, so rolling back would orphan a fully
    /// copied tree. That case must go through [`finish_move`].
    pub async fn rollback(&self, move_id: &str) -> Result<(), String> {
        let row = load_row(self.db, move_id).await?;
        let dest = PathBuf::from(&row.dest_path);
        match row.state.as_str() {
            "copying" | "verifying" => {}
            "verified" => {
                if read_sidecar(&dest).as_deref() == Some(row.id.as_str()) {
                    return Err(
                        "destination is already in place — finish the move instead of rolling back"
                            .into(),
                    );
                }
            }
            s => return Err(format!("move is {s}, nothing to roll back")),
        }
        let staging = PathBuf::from(&row.staging_path);
        if staging.exists() {
            match read_sidecar(&staging) {
                // Our staging dir, or one we created but hadn't marked yet.
                Some(id) if id == row.id => remove_staging(&staging)?,
                None => remove_staging(&staging)?,
                Some(_) => {
                    return Err("staging sidecar id does not match — refusing to delete".into())
                }
            }
        }
        update_aborted(self.db, &row.id).await?;
        move_lock::unlock(&[PathBuf::from(&row.source_path), dest]);
        Ok(())
    }
}

// ── Lock guard ───────────────────────────────────────────────────────────────

/// Holds the move lock for a scope and releases it on drop, so every early
/// return and `?` unlocks the source/dest exactly once.
struct LockGuard {
    paths: Vec<PathBuf>,
}

impl LockGuard {
    fn new(paths: Vec<PathBuf>) -> Self {
        move_lock::lock(&paths);
        Self { paths }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        move_lock::unlock(&self.paths);
    }
}

// ── Manifest / walk ──────────────────────────────────────────────────────────

/// Walk `source` and record every file (path relative to the root, forward
/// slashes), skipping regenerable dirs (`node_modules`, `target`, …) that git
/// confirms are ignored — unless `include_regenerable`. Entries are sorted by
/// `rel` so the manifest and every re-walk are directly comparable.
///
/// ponytail: empty directories aren't recorded, so a move won't recreate one
/// that held no files. Add dir entries to the manifest if a project depends on
/// an empty dir existing; files are the norm and preserving them is the point.
fn build_manifest(source: &Path, include_regenerable: bool) -> Result<(Vec<ManifestEntry>, Vec<String>), String> {
    let mut entries = Vec::new();
    let mut skipped = Vec::new();
    walk(source, source, include_regenerable, &mut entries, &mut skipped)?;
    entries.sort_by(|a, b| a.rel.cmp(&b.rel));
    skipped.sort();
    Ok((entries, skipped))
}

fn walk(
    root: &Path,
    dir: &Path,
    include_regenerable: bool,
    entries: &mut Vec<ManifestEntry>,
    skipped: &mut Vec<String>,
) -> Result<(), String> {
    let read = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    for entry in read {
        let entry = entry.map_err(|e| format!("read entry in {}: {e}", dir.display()))?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .map_err(|e| format!("file type of {}: {e}", path.display()))?;

        // Symlinks/junctions are refused up front by usage::check; never follow
        // or copy a reparse point here.
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let rel = rel_path(root, &path);
            if !include_regenerable && REGENERABLE.contains(&name.as_str()) && is_git_ignored(root, &rel) {
                skipped.push(rel);
                continue;
            }
            walk(root, &path, include_regenerable, entries, skipped)?;
        } else if ft.is_file() {
            let md = entry
                .metadata()
                .map_err(|e| format!("metadata of {}: {e}", path.display()))?;
            entries.push(ManifestEntry {
                rel: rel_path(root, &path),
                len: md.len(),
                mtime: mtime_millis(&md),
            });
        }
    }
    Ok(())
}

/// True only when `git check-ignore` positively confirms `rel` is ignored
/// inside `repo`. Any other outcome — not a git repo, git missing, path not
/// ignored — is false, so an uncertain dir is copied rather than dropped.
fn is_git_ignored(repo: &Path, rel: &str) -> bool {
    Command::new("git")
        .current_dir(repo)
        .no_window()
        .args(["check-ignore", "-q", rel])
        .status()
        .map(|s| s.code() == Some(0))
        .unwrap_or(false)
}

/// Re-walk `source` for verify/resume, pruning exactly the dirs captured in the
/// manifest's `skipped_dirs` — no `git check-ignore`, so the result is
/// independent of git state drifting mid-move. A regenerable dir that appears
/// *after* capture is not in the prune set, so it is walked and makes the set
/// differ from the manifest — which is the point: the source changed.
fn rewalk(source: &Path, skipped_dirs: &[String]) -> Result<Vec<ManifestEntry>, String> {
    let prune: HashSet<&str> = skipped_dirs.iter().map(String::as_str).collect();
    let mut entries = Vec::new();
    rewalk_dir(source, source, &prune, &mut entries)?;
    entries.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(entries)
}

fn rewalk_dir(
    root: &Path,
    dir: &Path,
    prune: &HashSet<&str>,
    entries: &mut Vec<ManifestEntry>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| format!("read entry in {}: {e}", dir.display()))?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .map_err(|e| format!("file type of {}: {e}", path.display()))?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            if prune.contains(rel_path(root, &path).as_str()) {
                continue;
            }
            rewalk_dir(root, &path, prune, entries)?;
        } else if ft.is_file() {
            let md = entry
                .metadata()
                .map_err(|e| format!("metadata of {}: {e}", path.display()))?;
            entries.push(ManifestEntry {
                rel: rel_path(root, &path),
                len: md.len(),
                mtime: mtime_millis(&md),
            });
        }
    }
    Ok(())
}

// ── Copy / verify ────────────────────────────────────────────────────────────

/// Decide which manifest entries still need copying. A file counts as done
/// only if it is in the progress log *and* its staging copy is present with the
/// right length; anything else — unlogged (possibly mid-write at the crash),
/// missing, or wrong-length — is deleted from staging and queued for re-copy.
/// Returns the indices to copy plus the done file/byte counts.
fn plan_copies(
    entries: &[ManifestEntry],
    staging: &Path,
    logged: &HashMap<String, String>,
) -> Result<(Vec<usize>, u64, u64), String> {
    let mut to_copy = Vec::new();
    let mut files_done = 0u64;
    let mut bytes_done = 0u64;
    for (i, e) in entries.iter().enumerate() {
        let st = os_join(staging, &e.rel);
        let done = logged.contains_key(&e.rel) && file_len(&st) == Some(e.len);
        if done {
            files_done += 1;
            bytes_done += e.len;
        } else {
            if st.exists() {
                fs::remove_file(&st).map_err(|err| format!("drop stale staging {}: {err}", st.display()))?;
            }
            to_copy.push(i);
        }
    }
    Ok((to_copy, files_done, bytes_done))
}

/// Copy one file into staging, fsync it, then prove it by hashing the source
/// and the destination through separate reads. Returns the shared hash.
fn copy_entry(source: &Path, staging: &Path, e: &ManifestEntry) -> Result<String, String> {
    let src = os_join(source, &e.rel);
    let dst = os_join(staging, &e.rel);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("create {}: {err}", parent.display()))?;
    }
    fs::copy(&src, &dst).map_err(|err| format!("copy {} -> {}: {err}", src.display(), dst.display()))?;

    // Reopen for write and fsync: flushing OS buffers needs write access on
    // Windows (FlushFileBuffers).
    fs::OpenOptions::new()
        .write(true)
        .open(&dst)
        .and_then(|f| f.sync_all())
        .map_err(|err| format!("fsync {}: {err}", dst.display()))?;

    let src_h = hash_file(&src)?;
    let dst_h = hash_file(&dst)?;
    if src_h != dst_h {
        return Err(format!("copy of {} does not match its source", e.rel));
    }
    Ok(src_h)
}

/// The real proof the copy is good: re-walk the source and confirm its file set
/// still matches the manifest (nothing added/removed/resized mid-move), then
/// re-hash every destination file against the live source. The progress log is
/// never consulted here.
fn final_verify(source: &Path, staging: &Path, manifest: &Manifest) -> Result<(), String> {
    let current = rewalk(source, &manifest.skipped_dirs)?;
    if !same_set(&current, &manifest.entries) {
        return Err("source changed during the move (files added, removed, or resized)".into());
    }
    for e in &manifest.entries {
        let src_h = hash_file(&os_join(source, &e.rel))?;
        let dst_h = hash_file(&os_join(staging, &e.rel))?;
        if src_h != dst_h {
            return Err(format!("verify failed: {} differs from its source", e.rel));
        }
    }
    Ok(())
}

/// Resume gate. `Ok` only when the staging sidecar matches, the live source
/// still matches the manifest, every logged file's source still hashes to its
/// logged value, and staging holds nothing the manifest doesn't know. Any `Err`
/// means resume is unsafe and the caller should offer rollback only.
fn can_resume(
    move_id: &str,
    source: &Path,
    staging: &Path,
    manifest: &Manifest,
    logged: &HashMap<String, String>,
) -> Result<(), String> {
    match read_sidecar(staging) {
        Some(id) if id == move_id => {}
        _ => return Err("staging marker missing or from another move — roll back".into()),
    }
    let current = rewalk(source, &manifest.skipped_dirs)?;
    if !same_set(&current, &manifest.entries) {
        return Err("source changed since the copy began — roll back".into());
    }
    for (rel, sha) in logged {
        if &hash_file(&os_join(source, rel))? != sha {
            return Err(format!("source file {rel} changed since it was copied — roll back"));
        }
    }
    let known: HashSet<&str> = manifest.entries.iter().map(|e| e.rel.as_str()).collect();
    for rel in list_rel_files(staging)? {
        if rel != SIDECAR && !known.contains(rel.as_str()) {
            return Err(format!("staging holds an unexpected file {rel} — roll back"));
        }
    }
    Ok(())
}

// ── Hashing / small fs helpers ───────────────────────────────────────────────

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; BUF];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("read {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_manifest(path: &Path, manifest: &Manifest) -> Result<String, String> {
    let bytes = serde_json::to_vec(manifest).map_err(|e| format!("serialize manifest: {e}"))?;
    let mut f = fs::File::create(path).map_err(|e| format!("create manifest: {e}"))?;
    f.write_all(&bytes).map_err(|e| format!("write manifest: {e}"))?;
    f.sync_all().map_err(|e| format!("fsync manifest: {e}"))?;
    Ok(sha256_bytes(&bytes))
}

fn read_validated_manifest(row: &project_move::Model) -> Result<Manifest, String> {
    let path = row.manifest_path.as_deref().ok_or("move has no manifest path")?;
    let expected = row.manifest_sha256.as_deref().ok_or("move has no manifest hash")?;
    let bytes = fs::read(path).map_err(|e| format!("read manifest {path}: {e}"))?;
    if sha256_bytes(&bytes) != expected {
        return Err("manifest hash mismatch — roll back".into());
    }
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|e| format!("parse manifest: {e}"))?;
    if manifest.version != MANIFEST_VERSION {
        return Err(format!(
            "manifest version {} is not {MANIFEST_VERSION} — roll back",
            manifest.version
        ));
    }
    Ok(manifest)
}

fn append_progress(path: &Path, rel: &str, sha: &str) -> Result<(), String> {
    let line = serde_json::to_string(&ProgressLine {
        rel: rel.to_string(),
        sha256: sha.to_string(),
    })
    .map_err(|e| format!("serialize progress line: {e}"))?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open progress log: {e}"))?;
    f.write_all(line.as_bytes()).and_then(|_| f.write_all(b"\n"))
        .map_err(|e| format!("write progress log: {e}"))?;
    f.sync_all().map_err(|e| format!("fsync progress log: {e}"))?;
    Ok(())
}

/// Parse the append-only log into rel → sha. A truncated last line (a crash
/// mid-append) is skipped rather than fatal — that file simply isn't "done".
fn read_progress(path: &Path) -> Result<HashMap<String, String>, String> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(e) => return Err(format!("read progress log: {e}")),
    };
    let mut out = HashMap::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(p) = serde_json::from_str::<ProgressLine>(line) {
            out.insert(p.rel, p.sha256);
        }
    }
    Ok(out)
}

fn write_sidecar(staging: &Path, id: &str) -> Result<(), String> {
    fs::write(staging.join(SIDECAR), id).map_err(|e| format!("write staging marker: {e}"))
}

fn read_sidecar(staging: &Path) -> Option<String> {
    fs::read_to_string(staging.join(SIDECAR)).ok().map(|s| s.trim().to_string())
}

fn remove_staging(staging: &Path) -> Result<(), String> {
    fs::remove_dir_all(staging).map_err(|e| format!("delete staging {}: {e}", staging.display()))
}

/// List every file under `root`, path relative with forward slashes. Unlike
/// [`build_manifest`] it skips nothing — used to find stray files in staging.
fn list_rel_files(root: &Path) -> Result<Vec<String>, String> {
    fn rec(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| format!("read entry: {e}"))?;
            let path = entry.path();
            let ft = entry.file_type().map_err(|e| format!("file type: {e}"))?;
            if ft.is_dir() {
                rec(root, &path, out)?;
            } else if ft.is_file() {
                out.push(rel_path(root, &path));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    rec(root, root, &mut out)?;
    Ok(out)
}

// ── Journal helpers ──────────────────────────────────────────────────────────

async fn load_row(db: &DatabaseManager, id: &str) -> Result<project_move::Model, String> {
    project_move::Entity::find_by_id(id.to_string())
        .one(db.get_connection())
        .await
        .map_err(|e| format!("load move {id}: {e}"))?
        .ok_or_else(|| format!("no move with id {id}"))
}

async fn update_state(db: &DatabaseManager, id: &str, state: &str) -> Result<(), String> {
    project_move::ActiveModel {
        id: Set(id.to_string()),
        state: Set(state.to_string()),
        updated_at: Set(now()),
        ..Default::default()
    }
    .update(db.get_connection())
    .await
    .map(|_| ())
    .map_err(|e| format!("set move state {state}: {e}"))
}

async fn update_counts(db: &DatabaseManager, id: &str, files_done: u64, bytes_done: u64) -> Result<(), String> {
    project_move::ActiveModel {
        id: Set(id.to_string()),
        files_done: Set(files_done as i32),
        bytes_done: Set(bytes_done as i64),
        updated_at: Set(now()),
        ..Default::default()
    }
    .update(db.get_connection())
    .await
    .map(|_| ())
    .map_err(|e| format!("update move counts: {e}"))
}

async fn update_done(db: &DatabaseManager, id: &str) -> Result<(), String> {
    let now = now();
    project_move::ActiveModel {
        id: Set(id.to_string()),
        state: Set("done".into()),
        updated_at: Set(now.clone()),
        finished_at: Set(Some(now)),
        ..Default::default()
    }
    .update(db.get_connection())
    .await
    .map(|_| ())
    .map_err(|e| format!("mark move done: {e}"))
}

async fn update_aborted(db: &DatabaseManager, id: &str) -> Result<(), String> {
    let now = now();
    project_move::ActiveModel {
        id: Set(id.to_string()),
        state: Set("aborted".into()),
        updated_at: Set(now.clone()),
        finished_at: Set(Some(now)),
        ..Default::default()
    }
    .update(db.get_connection())
    .await
    .map(|_| ())
    .map_err(|e| format!("mark move aborted: {e}"))
}

async fn mark_error(db: &DatabaseManager, id: &str, err: &str) -> Result<(), String> {
    project_move::ActiveModel {
        id: Set(id.to_string()),
        error: Set(Some(err.to_string())),
        updated_at: Set(now()),
        ..Default::default()
    }
    .update(db.get_connection())
    .await
    .map(|_| ())
    .map_err(|e| format!("record move error: {e}"))
}

async fn bump_resume_count(db: &DatabaseManager, id: &str) -> Result<(), String> {
    let row = load_row(db, id).await?;
    project_move::ActiveModel {
        id: Set(id.to_string()),
        resume_count: Set(row.resume_count + 1),
        updated_at: Set(now()),
        ..Default::default()
    }
    .update(db.get_connection())
    .await
    .map(|_| ())
    .map_err(|e| format!("bump resume count: {e}"))
}

// ── Path helpers ─────────────────────────────────────────────────────────────

/// Staging sits next to the destination so the promote is an atomic
/// same-volume `rename`.
fn staging_path(dest: &Path, id: &str) -> Result<PathBuf, String> {
    let parent = dest
        .parent()
        .ok_or_else(|| format!("destination {} has no parent", dest.display()))?;
    Ok(parent.join(format!(".portal-staging-{id}")))
}

fn progress_path_of(row: &project_move::Model) -> Result<PathBuf, String> {
    row.progress_path
        .as_deref()
        .map(PathBuf::from)
        .ok_or_else(|| "move has no progress path".into())
}

/// Join a forward-slash manifest rel onto an OS path.
fn os_join(base: &Path, rel: &str) -> PathBuf {
    base.join(rel.replace('/', MAIN_SEPARATOR_STR))
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// New path for a project row sitting under `source`, rebased onto `dest`. The
/// relative tail keeps its original case and separators; only the prefix moves.
/// `None` when `old` isn't under `source`.
fn rebased_path(old: &str, source: &str, dest: &str) -> Option<String> {
    if !is_under(old, source) {
        return None;
    }
    let src_n = split_components(source).len();
    let parts = split_components(old);
    if parts.len() < src_n {
        return None;
    }
    let mut p = PathBuf::from(dest);
    for c in &parts[src_n..] {
        p.push(c);
    }
    Some(p.to_string_lossy().into_owned())
}

fn split_components(s: &str) -> Vec<&str> {
    s.split(['/', '\\']).filter(|c| !c.is_empty()).collect()
}

fn same_set(a: &[ManifestEntry], b: &[ManifestEntry]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.rel == y.rel && x.len == y.len)
}

fn file_len(p: &Path) -> Option<u64> {
    fs::metadata(p).ok().map(|m| m.len())
}

fn mtime_millis(md: &fs::Metadata) -> i64 {
    md.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn port(p: Option<i32>) -> Option<u16> {
    p.and_then(|v| u16::try_from(v).ok())
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ── Tests (temp dirs only; never a real project) ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &Path, body: &str) {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn hash_file_matches_known_vector() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("a");
        fs::write(&f, b"abc").unwrap();
        // SHA-256("abc")
        assert_eq!(
            hash_file(&f).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn manifest_lists_files_relative_and_sorted() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("b.txt"), "two");
        write(&dir.path().join("sub/a.txt"), "one");
        let (entries, skipped) = build_manifest(dir.path(), false).unwrap();
        let rels: Vec<_> = entries.iter().map(|e| e.rel.clone()).collect();
        assert_eq!(rels, vec!["b.txt", "sub/a.txt"]); // sorted, forward slashes
        assert!(skipped.is_empty());
        assert_eq!(entries[1].len, 3);
    }

    #[test]
    fn include_regenerable_keeps_node_modules() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("node_modules/dep/index.js"), "x");
        write(&dir.path().join("src/main.rs"), "y");
        // include_regenerable=true never consults git; node_modules is copied.
        let (entries, _) = build_manifest(dir.path(), true).unwrap();
        assert!(entries.iter().any(|e| e.rel == "node_modules/dep/index.js"));
    }

    #[test]
    fn rewalk_prunes_exactly_the_captured_skip_set_without_git() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("node_modules/dep/index.js"), "x");
        write(&dir.path().join("src/main.rs"), "y");
        // Pruned deterministically from the captured set — no git involved.
        let pruned = rewalk(dir.path(), &["node_modules".to_string()]).unwrap();
        assert_eq!(pruned.iter().map(|e| e.rel.as_str()).collect::<Vec<_>>(), vec!["src/main.rs"]);
        // Empty skip set walks everything.
        let full = rewalk(dir.path(), &[]).unwrap();
        assert_eq!(full.len(), 2);
    }

    #[test]
    fn copy_then_verify_roundtrips_and_catches_tamper() {
        let src = tempdir().unwrap();
        let stg = tempdir().unwrap();
        write(&src.path().join("a.txt"), "alpha");
        write(&src.path().join("sub/b.txt"), "beta");
        let (entries, skipped) = build_manifest(src.path(), false).unwrap();

        for e in &entries {
            copy_entry(src.path(), stg.path(), e).unwrap();
        }
        let manifest = Manifest { version: MANIFEST_VERSION, skipped_dirs: skipped, entries };
        final_verify(src.path(), stg.path(), &manifest).unwrap();

        // Tampered destination: verify must catch it (it re-hashes dest vs src).
        fs::write(stg.path().join("a.txt"), "ALPHA!").unwrap();
        assert!(final_verify(src.path(), stg.path(), &manifest).is_err());
    }

    #[test]
    fn verify_detects_a_new_source_file() {
        let src = tempdir().unwrap();
        let stg = tempdir().unwrap();
        write(&src.path().join("a.txt"), "alpha");
        let (entries, skipped) = build_manifest(src.path(), false).unwrap();
        copy_entry(src.path(), stg.path(), &entries[0]).unwrap();
        let manifest = Manifest { version: MANIFEST_VERSION, skipped_dirs: skipped, entries };
        // A file appears in the source mid-move.
        write(&src.path().join("new.txt"), "surprise");
        assert!(final_verify(src.path(), stg.path(), &manifest).is_err());
    }

    #[test]
    fn plan_copies_recopies_unlogged_and_wrong_length_keeps_logged() {
        let src = tempdir().unwrap();
        let stg = tempdir().unwrap();
        write(&src.path().join("done.txt"), "aaaa");
        write(&src.path().join("partial.txt"), "bbbbbb");
        write(&src.path().join("missing.txt"), "cc");
        let (entries, _) = build_manifest(src.path(), false).unwrap();

        // Stage all three, but: only "done" is logged with a matching length;
        // "partial" is logged but wrong length; "missing" is not staged at all.
        for e in &entries {
            copy_entry(src.path(), stg.path(), e).unwrap();
        }
        fs::remove_file(stg.path().join("missing.txt")).unwrap();
        fs::write(stg.path().join("partial.txt"), "short").unwrap(); // wrong length

        let mut logged = HashMap::new();
        logged.insert("done.txt".to_string(), "h".to_string());
        logged.insert("partial.txt".to_string(), "h".to_string());

        let (to_copy, files_done, _) = plan_copies(&entries, stg.path(), &logged).unwrap();
        let rels: Vec<_> = to_copy.iter().map(|&i| entries[i].rel.clone()).collect();
        assert_eq!(files_done, 1); // only done.txt counts
        assert!(rels.contains(&"missing.txt".to_string()));
        assert!(rels.contains(&"partial.txt".to_string()));
        assert!(!rels.contains(&"done.txt".to_string()));
        // The wrong-length staging file was dropped so it will be re-copied clean.
        assert!(!stg.path().join("partial.txt").exists());
    }

    #[test]
    fn can_resume_refuses_when_a_logged_source_changed() {
        let src = tempdir().unwrap();
        let stg = tempdir().unwrap();
        write(&src.path().join("a.txt"), "original");
        let (entries, _) = build_manifest(src.path(), false).unwrap();
        let manifest = Manifest { version: MANIFEST_VERSION, skipped_dirs: vec![], entries };
        write_sidecar(stg.path(), "move-1").unwrap();

        let sha = hash_file(&src.path().join("a.txt")).unwrap();
        let mut logged = HashMap::new();
        logged.insert("a.txt".to_string(), sha);

        // Matching source + logged hash + sidecar → resume allowed. But the
        // source set must still match: nothing else staged, manifest intact.
        can_resume("move-1", src.path(), stg.path(), &manifest, &logged).unwrap();

        // Change the source content: the logged hash no longer matches.
        fs::write(src.path().join("a.txt"), "changed").unwrap();
        assert!(can_resume("move-1", src.path(), stg.path(), &manifest, &logged).is_err());
    }

    #[test]
    fn can_resume_refuses_on_sidecar_mismatch() {
        let src = tempdir().unwrap();
        let stg = tempdir().unwrap();
        write(&src.path().join("a.txt"), "x");
        let (entries, _) = build_manifest(src.path(), false).unwrap();
        let manifest = Manifest { version: MANIFEST_VERSION, skipped_dirs: vec![], entries };
        write_sidecar(stg.path(), "some-other-move").unwrap();
        let err = can_resume("move-1", src.path(), stg.path(), &manifest, &HashMap::new())
            .unwrap_err();
        assert!(err.contains("roll back"));
    }

    #[test]
    fn rebased_path_moves_prefix_keeps_tail_and_ignores_siblings() {
        // Moved project itself.
        assert_eq!(
            rebased_path(r"D:\old\proj", r"D:\old\proj", r"E:\new\proj"),
            Some(r"E:\new\proj".to_string())
        );
        // Nested registered project keeps its tail (and tail case).
        assert_eq!(
            rebased_path(r"D:\old\proj\Pkg\api", r"D:\old\proj", r"E:\new\proj"),
            Some(PathBuf::from(r"E:\new\proj").join("Pkg").join("api").to_string_lossy().into_owned())
        );
        // A sibling sharing a prefix is not rebased.
        assert_eq!(rebased_path(r"D:\old\project2", r"D:\old\proj", r"E:\new\proj"), None);
    }

    #[test]
    fn list_rel_files_finds_everything_including_the_sidecar() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("x.txt"), "1");
        write(&dir.path().join("d/y.txt"), "2");
        write_sidecar(dir.path(), "id").unwrap();
        let mut files = list_rel_files(dir.path()).unwrap();
        files.sort();
        assert_eq!(files, vec![".portal-move", "d/y.txt", "x.txt"]);
    }
}

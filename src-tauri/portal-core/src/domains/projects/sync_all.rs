//! "Sync all projects": env files to the vault, every git repo fetched /
//! fast-forwarded / (allowlisted) pushed, then one cloud snapshot.
//!
//! Phase order is env, git, snapshot. Every failure is a step status, never an
//! `Err`; `attention` means "needs your hand", `failed` means "a command errored".

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, Unchanged};
use serde::{Deserialize, Serialize};

use crate::database::DatabaseManager;
use crate::domains::credentials::services::VaultSession;
use crate::domains::projects::discovery::git::{self, GitProbe};
use crate::domains::projects::env::backup::{
    EnvBackupService, EnvFileRef, EnvFileState, EnvOutcome,
};
use crate::domains::projects::move_lock;
use crate::domains::projects::repositories::discovery_repository::apply_git_fields;
use crate::domains::sync::service::{PushOutcome, SyncService};
use crate::entities::project;

const WORKERS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum SyncAllPhase {
    Env,
    Git,
    Snapshot,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum SyncStep {
    Env,
    Fetch,
    Pull,
    Push,
    Mark,
    /// The cloud snapshot; only ever on `SyncAllReport::snapshot`.
    Snapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum SyncStepStatus {
    Running,
    Ok,
    Skipped,
    /// Needs your hand.
    Attention,
    /// A command errored.
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncProjectEvent {
    pub id: i32,
    pub name: String,
    pub step: SyncStep,
    pub status: SyncStepStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncAllProgress {
    pub phase: SyncAllPhase,
    #[ts(type = "number")]
    pub total: usize,
    #[ts(type = "number")]
    pub done: usize,
    pub project: Option<SyncProjectEvent>,
}

// No `Debug`: this holds the snapshot password; keep it un-formattable so it can't leak via `{:?}`.
#[derive(Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncAllOptions {
    /// Snapshot password; `None` skips the cloud snapshot. Never logged or emitted.
    pub snapshot_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncStepResult {
    pub step: SyncStep,
    pub status: SyncStepStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncProjectResult {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub steps: Vec<SyncStepResult>,
    /// Git counts after the sync; `-1` = unknown (no upstream / not measured).
    #[ts(type = "number")]
    pub ahead: i64,
    #[ts(type = "number")]
    pub behind: i64,
    #[ts(type = "number")]
    pub dirty: i64,
    #[ts(type = "number")]
    pub untracked: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncAllReport {
    pub projects: Vec<SyncProjectResult>,
    pub env_note: Option<String>,
    pub snapshot: SyncStepResult,
    pub cancelled: bool,
    pub started_at: String,
    pub finished_at: String,
}

type OnStep<'a> = &'a mut dyn FnMut(SyncStep, SyncStepStatus, Option<String>);

fn skip_all(on: OnStep, steps: &[SyncStep], why: &str) {
    for s in steps {
        on(*s, SyncStepStatus::Skipped, Some(why.into()));
    }
}

/// Fetch, then pull / push as the repo's state allows. Split out of
/// `sync_repo` so every early exit is a plain `return`.
fn net_steps(dir: &Path, push_ok: bool, cancel: &AtomicBool, on: OnStep) {
    use SyncStep::*;
    use SyncStepStatus as S;
    if cancel.load(Ordering::Relaxed) {
        return skip_all(on, &[Fetch, Pull, Push], "cancelled");
    }
    on(Fetch, S::Running, None);
    match git::git_fetch(dir, cancel) {
        Ok(()) => on(Fetch, S::Ok, None),
        // Killed by Cancel, not a failure.
        Err(_) if cancel.load(Ordering::Relaxed) => {
            return skip_all(on, &[Fetch, Pull, Push], "cancelled");
        }
        Err(e) => {
            on(Fetch, S::Failed, Some(e));
            return skip_all(on, &[Pull, Push], "fetch failed");
        }
    }
    if let Some(why) = git::busy(dir) {
        return skip_all(on, &[Pull, Push], why);
    }
    if cancel.load(Ordering::Relaxed) {
        return skip_all(on, &[Pull, Push], "cancelled");
    }

    let ab = git::ahead_behind(dir);
    let mut blocked = false;
    match ab {
        None => on(Pull, S::Skipped, Some("no upstream".into())),
        Some((_, 0)) => on(Pull, S::Ok, Some("up to date".into())),
        Some((0, behind)) => {
            on(Pull, S::Running, None);
            match git::git_pull_ff_only(dir) {
                Ok(()) => on(Pull, S::Ok, Some(format!("fast-forwarded {behind}"))),
                Err(e) => {
                    on(Pull, S::Failed, Some(e));
                    blocked = true;
                }
            }
        }
        Some((ahead, behind)) => {
            let d = format!("diverged ({ahead} ahead, {behind} behind) — needs manual merge");
            on(Pull, S::Attention, Some(d));
            blocked = true;
        }
    }

    let ahead = ab.map(|(a, _)| a);
    if blocked {
        on(Push, S::Skipped, Some("pull did not complete".into()));
    } else if ahead == Some(0) {
        on(Push, S::Skipped, Some("nothing to push".into()));
    } else if !push_ok {
        match ahead {
            Some(n) => on(
                Push,
                S::Attention,
                Some(format!("{n} commits not pushed — not on push allowlist")),
            ),
            None => on(Push, S::Skipped, Some("no upstream".into())),
        }
    } else if cancel.load(Ordering::Relaxed) {
        on(Push, S::Skipped, Some("cancelled".into()));
    } else {
        on(Push, S::Running, None);
        match git::git_push_net(dir, cancel) {
            Ok(b) => on(Push, S::Ok, Some(format!("pushed {b}"))),
            Err(_) if cancel.load(Ordering::Relaxed) => {
                on(Push, S::Skipped, Some("cancelled".into()))
            }
            Err(e) => on(Push, S::Failed, Some(e)),
        }
    }
}

/// One repository, start to finish. Never errors: every outcome is reported
/// through `on_step`. The returned probe is the state after the sync
/// (`GitProbe::absent()` when nothing was inspected).
pub fn sync_repo(dir: &Path, push_ok: bool, cancel: &AtomicBool, on_step: OnStep) -> GitProbe {
    use SyncStep::*;
    let all = [Fetch, Pull, Push, Mark];
    if cancel.load(Ordering::Relaxed) {
        skip_all(on_step, &all, "cancelled");
        return GitProbe::absent();
    }
    if !dir.is_dir() {
        skip_all(on_step, &all, "folder not on this machine");
        return GitProbe::absent();
    }
    if let Err(e) = move_lock::ensure_unlocked(dir) {
        skip_all(on_step, &all, &e);
        return GitProbe::absent();
    }
    if git::toplevel(dir).is_none() {
        skip_all(on_step, &all, "not a git repository");
        return GitProbe::absent();
    }
    if git::has_remote(dir) {
        net_steps(dir, push_ok, cancel, on_step);
    } else {
        skip_all(on_step, &[Fetch, Pull, Push], "no remote");
    }

    let p = git::probe(dir);
    if p.dirty > 0 || p.untracked > 0 {
        let d = format!(
            "{} modified, {} untracked — commit manually",
            p.dirty.max(0),
            p.untracked.max(0)
        );
        on_step(Mark, SyncStepStatus::Attention, Some(d));
    } else {
        on_step(Mark, SyncStepStatus::Ok, None);
    }
    p
}

/// One git toplevel and the projects living in it.
struct Group {
    /// Index of the project whose path the repo is synced from.
    leader: usize,
    members: Vec<usize>,
    push_ok: bool,
}

struct RepoOutcome {
    steps: Vec<SyncStepResult>,
    probe: GitProbe,
}

pub struct SyncAllService {
    db: Arc<DatabaseManager>,
    session: Arc<VaultSession>,
}

type Emit = Arc<dyn Fn(SyncAllProgress) + Send + Sync>;

/// Worst status wins: failed > attention > ok.
fn worst(a: SyncStepStatus, b: SyncStepStatus) -> SyncStepStatus {
    use SyncStepStatus::*;
    match (a, b) {
        (Failed, _) | (_, Failed) => Failed,
        (Attention, _) | (_, Attention) => Attention,
        _ => Ok,
    }
}

impl SyncAllService {
    pub fn new(db: Arc<DatabaseManager>, session: Arc<VaultSession>) -> Self {
        Self { db, session }
    }

    pub async fn run(
        &self,
        options: SyncAllOptions,
        cancel: Arc<AtomicBool>,
        emit: Emit,
    ) -> Result<SyncAllReport, String> {
        let started_at = Utc::now().to_rfc3339();
        let conn = self.db.get_connection();
        // Parked projects are left alone.
        let rows = project::Entity::find()
            .filter(project::Column::Archived.eq(false))
            .all(conn)
            .await
            .map_err(|e| format!("Failed to load projects: {e}"))?;
        let total = rows.len();
        let mut results: Vec<SyncProjectResult> = rows
            .iter()
            .map(|r| SyncProjectResult {
                id: r.id,
                name: r.name.clone(),
                path: r.path.clone(),
                steps: Vec::new(),
                ahead: -1,
                behind: -1,
                dirty: -1,
                untracked: -1,
            })
            .collect();
        let tick = |phase, done, project| {
            emit(SyncAllProgress {
                phase,
                total,
                done,
                project,
            })
        };

        // --- env ---
        tick(SyncAllPhase::Env, 0, None);
        let mut env_note = None;
        if !cancel.load(Ordering::Relaxed) {
            if !self.session.is_unlocked() {
                env_note = Some("Vault locked — env files not backed up".to_string());
            } else {
                match self.env_phase().await {
                    Ok(per_project) => {
                        for (id, status, detail) in per_project {
                            let Some(r) = results.iter_mut().find(|r| r.id == id) else {
                                continue;
                            };
                            tick(
                                SyncAllPhase::Env,
                                0,
                                Some(SyncProjectEvent {
                                    id,
                                    name: r.name.clone(),
                                    step: SyncStep::Env,
                                    status,
                                    detail: detail.clone(),
                                }),
                            );
                            r.steps.push(SyncStepResult {
                                step: SyncStep::Env,
                                status,
                                detail,
                            });
                        }
                    }
                    Err(e) => env_note = Some(e),
                }
            }
        }

        // --- git ---
        tick(SyncAllPhase::Git, 0, None);
        let (groups, outcomes) = {
            let paths: Vec<String> = rows.iter().map(|r| r.path.clone()).collect();
            let push_ok: Vec<bool> = rows.iter().map(|r| r.push_ok).collect();
            let names: Vec<(i32, String)> = rows.iter().map(|r| (r.id, r.name.clone())).collect();
            let (cancel, emit) = (cancel.clone(), emit.clone());
            tokio::task::spawn_blocking(move || git_phase(&paths, &push_ok, &names, &cancel, &emit))
                .await
                .map_err(|e| e.to_string())?
        };

        // One writer, after the workers have joined.
        for (g, o) in groups.iter().zip(&outcomes) {
            for &m in &g.members {
                let r = &mut results[m];
                if m == g.leader {
                    r.steps.extend(o.steps.iter().cloned());
                } else {
                    r.steps.push(SyncStepResult {
                        step: SyncStep::Mark,
                        status: SyncStepStatus::Skipped,
                        detail: Some(format!("same repository as {}", rows[g.leader].name)),
                    });
                }
                if o.probe.present {
                    (r.ahead, r.behind, r.dirty, r.untracked) = (
                        o.probe.ahead,
                        o.probe.behind,
                        o.probe.dirty,
                        o.probe.untracked,
                    );
                    let mut am = project::ActiveModel {
                        id: Unchanged(r.id),
                        ..Default::default()
                    };
                    apply_git_fields(&mut am, &o.probe);
                    am.updated_at = Set(Some(Utc::now().into()));
                    // A row deleted mid-run is not worth failing the report over.
                    let _ = am.update(conn).await;
                }
            }
        }

        // --- snapshot ---
        tick(SyncAllPhase::Snapshot, total, None);
        let snap = |status, detail: Option<String>| SyncStepResult {
            step: SyncStep::Snapshot,
            status,
            detail,
        };
        let snapshot = if cancel.load(Ordering::Relaxed) {
            snap(SyncStepStatus::Skipped, Some("cancelled".into()))
        } else if let Some(password) = options.snapshot_password {
            // Never force: a newer cloud snapshot is the user's call.
            match SyncService::new(self.db.data_dir())
                .push(password, false)
                .await
            {
                Ok(PushOutcome::Ok { record }) => snap(
                    SyncStepStatus::Ok,
                    Some(format!("version {}", record.version)),
                ),
                Ok(PushOutcome::Conflict { .. }) => snap(
                    SyncStepStatus::Attention,
                    Some("cloud has newer snapshot — resolve in Settings > Sync".into()),
                ),
                Err(e) => snap(SyncStepStatus::Failed, Some(e)),
            }
        } else {
            snap(SyncStepStatus::Skipped, Some("no snapshot password".into()))
        };

        tick(SyncAllPhase::Done, total, None);
        Ok(SyncAllReport {
            projects: results,
            env_note,
            snapshot,
            cancelled: cancel.load(Ordering::Relaxed),
            started_at,
            finished_at: Utc::now().to_rfc3339(),
        })
    }

    /// Backs up env files; one (project id, status, detail) per project that has any.
    async fn env_phase(&self) -> Result<Vec<(i32, SyncStepStatus, Option<String>)>, String> {
        use SyncStepStatus as S;
        let svc = EnvBackupService::new(self.db.get_connection().clone(), self.session.clone());
        let scan = svc.scan().await?;
        // project -> (worst status, ok count, non-ok notes), in first-seen order.
        let mut acc: HashMap<i32, (S, usize, Vec<String>)> = HashMap::new();
        let mut order = Vec::new();
        let mut note = |id: i32, st: S, msg: Option<String>| {
            let e = acc.entry(id).or_insert_with(|| {
                order.push(id);
                (S::Ok, 0, Vec::new())
            });
            e.0 = worst(e.0, st);
            match msg {
                Some(m) => e.2.push(m),
                None => e.1 += 1,
            }
        };

        let newer = |a: &Option<String>, b: &Option<String>| {
            let p = |s: &Option<String>| {
                s.as_deref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            };
            matches!((p(a), p(b)), (Some(a), Some(b)) if a > b)
        };
        let mut refs = Vec::new();
        for f in &scan.files {
            // A differing vault copy is only replaced when the disk file is newer.
            let overwrite = match f.state {
                EnvFileState::New => Some(false),
                EnvFileState::Differs if newer(&f.disk_modified, &f.vault_updated_at) => Some(true),
                EnvFileState::Differs | EnvFileState::Unknown => {
                    let m = format!("{}: vault copy newer — review manually", f.rel_path);
                    note(f.project_id, S::Attention, Some(m));
                    None
                }
                _ => None,
            };
            if let Some(overwrite) = overwrite {
                refs.push(EnvFileRef {
                    project_id: f.project_id,
                    rel_path: f.rel_path.clone(),
                    overwrite,
                });
            }
        }
        if !refs.is_empty() {
            for it in svc.import(Some(refs)).await?.items {
                let Some(id) = it.project_id else { continue };
                let detail = it.detail.unwrap_or_default();
                let msg = Some(format!("{}: {detail}", it.rel_path));
                match it.outcome {
                    EnvOutcome::Failed if detail.contains("Vault is locked") => {
                        note(id, S::Attention, msg)
                    }
                    EnvOutcome::Failed => note(id, S::Failed, msg),
                    EnvOutcome::Conflict => note(id, S::Attention, msg),
                    _ => note(id, S::Ok, None),
                }
            }
        }
        Ok(order
            .into_iter()
            .map(|id| {
                let (st, n, notes) = acc.remove(&id).expect("seen");
                let mut parts = Vec::new();
                if n > 0 {
                    parts.push(format!("{n} backed up"));
                }
                parts.extend(notes);
                (id, st, Some(parts.join("; ")))
            })
            .collect())
    }
}

/// Runs `f(0..n)` on up to `WORKERS` scoped threads pulling from a shared index.
fn pool(n: usize, f: &(dyn Fn(usize) + Sync)) {
    let next = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..WORKERS.min(n) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= n {
                    break;
                }
                f(i);
            });
        }
    });
}

/// Runs every repo through `sync_repo` on a small worker pool. Unit of work is
/// the git toplevel, so projects sharing a repo are synced once.
fn git_phase(
    paths: &[String],
    push_ok: &[bool],
    names: &[(i32, String)],
    cancel: &AtomicBool,
    emit: &Emit,
) -> (Vec<Group>, Vec<RepoOutcome>) {
    let total = paths.len();
    let done = AtomicUsize::new(0);

    let tops: Vec<Mutex<Option<String>>> = paths.iter().map(|_| Mutex::new(None)).collect();
    pool(total, &|i| {
        let t = git::toplevel(Path::new(&paths[i])).map(|t| t.to_string_lossy().into_owned());
        *tops[i].lock().unwrap() = t;
    });

    // Group by toplevel; a project with no repo is its own group. The member
    // sitting at the toplevel leads (and its push_ok decides); without one the
    // first member leads and every member must allow the push.
    let mut by_key: HashMap<String, usize> = HashMap::new();
    let mut groups: Vec<Group> = Vec::new();
    let mut at_top: Vec<bool> = Vec::new();
    for i in 0..total {
        let top = tops[i].lock().unwrap().clone();
        let key = top
            .as_deref()
            .map(move_lock::norm)
            .unwrap_or_else(|| format!("#{i}"));
        let gi = *by_key.entry(key).or_insert_with(|| {
            groups.push(Group {
                leader: i,
                members: Vec::new(),
                push_ok: false,
            });
            at_top.push(false);
            groups.len() - 1
        });
        groups[gi].members.push(i);
        if top.as_deref().map(move_lock::norm) == Some(move_lock::norm(&paths[i])) {
            groups[gi].leader = i;
            at_top[gi] = true;
        }
    }
    for (g, top) in groups.iter_mut().zip(&at_top) {
        g.push_ok = if *top {
            push_ok[g.leader]
        } else {
            g.members.iter().all(|&m| push_ok[m])
        };
    }

    let ev = |step, status, detail: Option<String>, i: usize| {
        emit(SyncAllProgress {
            phase: SyncAllPhase::Git,
            total,
            done: done.load(Ordering::SeqCst),
            project: Some(SyncProjectEvent {
                id: names[i].0,
                name: names[i].1.clone(),
                step,
                status,
                detail,
            }),
        });
    };
    let out: Vec<Mutex<Option<RepoOutcome>>> = groups.iter().map(|_| Mutex::new(None)).collect();
    pool(groups.len(), &|gi| {
        let g = &groups[gi];
        let mut steps = Vec::new();
        let probe = if cancel.load(Ordering::Relaxed) {
            let d = Some("cancelled".to_string());
            ev(SyncStep::Mark, SyncStepStatus::Skipped, d.clone(), g.leader);
            steps.push(SyncStepResult {
                step: SyncStep::Mark,
                status: SyncStepStatus::Skipped,
                detail: d,
            });
            GitProbe::absent()
        } else {
            let r = catch_unwind(AssertUnwindSafe(|| {
                let dir = Path::new(&paths[g.leader]);
                sync_repo(dir, g.push_ok, cancel, &mut |step, status, detail| {
                    ev(step, status, detail.clone(), g.leader);
                    if status != SyncStepStatus::Running {
                        steps.push(SyncStepResult {
                            step,
                            status,
                            detail,
                        });
                    }
                })
            }));
            r.unwrap_or_else(|_| {
                let d = Some("internal error while syncing this repository".to_string());
                ev(SyncStep::Mark, SyncStepStatus::Failed, d.clone(), g.leader);
                steps.push(SyncStepResult {
                    step: SyncStep::Mark,
                    status: SyncStepStatus::Failed,
                    detail: d,
                });
                GitProbe::absent()
            })
        };
        for &m in g.members.iter().filter(|&&m| m != g.leader) {
            let d = format!("same repository as {}", names[g.leader].1);
            ev(SyncStep::Mark, SyncStepStatus::Skipped, Some(d), m);
        }
        done.fetch_add(g.members.len(), Ordering::SeqCst);
        *out[gi].lock().unwrap() = Some(RepoOutcome { steps, probe });
    });

    let outcomes = out
        .into_iter()
        .map(|m| m.into_inner().unwrap().expect("every group ran"))
        .collect();
    (groups, outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn sh(dir: &Path, args: &[&str]) -> String {
        let o = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
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

    fn commit(dir: &Path, file: &str) {
        std::fs::write(dir.join(file), file).unwrap();
        sh(dir, &["add", "."]);
        sh(dir, &["commit", "-m", file]);
    }

    /// Bare remote with one commit, plus two clones of it. No network.
    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let t = tempfile::tempdir().unwrap();
        let (remote, a, b) = (
            t.path().join("remote.git"),
            t.path().join("a"),
            t.path().join("b"),
        );
        std::fs::create_dir(&remote).unwrap();
        sh(&remote, &["init", "--bare", "-b", "main"]);
        std::fs::create_dir(&a).unwrap();
        sh(&a, &["init", "-b", "main"]);
        sh(&a, &["remote", "add", "origin", remote.to_str().unwrap()]);
        commit(&a, "one");
        sh(&a, &["push", "-u", "origin", "main"]);
        sh(t.path(), &["clone", remote.to_str().unwrap(), "b"]);
        (t, remote, a, b)
    }

    fn run(dir: &Path, push_ok: bool, cancel: bool) -> Vec<SyncStepResult> {
        let mut steps = Vec::new();
        sync_repo(
            dir,
            push_ok,
            &AtomicBool::new(cancel),
            &mut |step, status, detail| {
                if status != SyncStepStatus::Running {
                    steps.push(SyncStepResult {
                        step,
                        status,
                        detail,
                    });
                }
            },
        );
        steps
    }

    fn status(steps: &[SyncStepResult], step: SyncStep) -> SyncStepStatus {
        steps
            .iter()
            .find(|s| s.step == step)
            .expect("step reported")
            .status
    }

    #[test]
    fn sync_all_behind_only_fast_forwards() {
        let (_t, remote, a, b) = fixture();
        commit(&a, "two");
        sh(&a, &["push"]);
        let steps = run(&b, false, false);
        assert_eq!(status(&steps, SyncStep::Pull), SyncStepStatus::Ok);
        assert_eq!(
            sh(&b, &["rev-parse", "HEAD"]),
            sh(&remote, &["rev-parse", "main"])
        );
    }

    #[test]
    fn sync_all_diverged_is_left_alone() {
        let (_t, _remote, a, b) = fixture();
        commit(&a, "two");
        sh(&a, &["push"]);
        commit(&b, "local");
        let head = sh(&b, &["rev-parse", "HEAD"]);
        let steps = run(&b, true, false);
        assert_eq!(status(&steps, SyncStep::Pull), SyncStepStatus::Attention);
        assert_eq!(status(&steps, SyncStep::Push), SyncStepStatus::Skipped);
        assert_eq!(sh(&b, &["rev-parse", "HEAD"]), head);
    }

    #[test]
    fn sync_all_ahead_push_follows_allowlist() {
        let (_t, remote, _a, b) = fixture();
        commit(&b, "local");
        let before = sh(&remote, &["rev-parse", "main"]);
        let steps = run(&b, false, false);
        assert_eq!(status(&steps, SyncStep::Push), SyncStepStatus::Attention);
        assert_eq!(sh(&remote, &["rev-parse", "main"]), before);

        let steps = run(&b, true, false);
        assert_eq!(status(&steps, SyncStep::Push), SyncStepStatus::Ok);
        assert_ne!(sh(&remote, &["rev-parse", "main"]), before);
        assert_eq!(
            sh(&remote, &["rev-parse", "main"]),
            sh(&b, &["rev-parse", "HEAD"])
        );
    }

    #[test]
    fn sync_all_dirty_tree_is_untouched() {
        let (_t, _remote, a, b) = fixture();
        commit(&a, "two");
        sh(&a, &["push"]);
        std::fs::write(b.join("one"), "edited").unwrap();
        std::fs::write(b.join("new.txt"), "x").unwrap();
        let before = sh(&b, &["status", "--porcelain"]);
        let steps = run(&b, false, false);
        assert_eq!(status(&steps, SyncStep::Mark), SyncStepStatus::Attention);
        assert_eq!(sh(&b, &["status", "--porcelain"]), before);
    }

    #[test]
    fn sync_all_cancel_before_start_touches_nothing() {
        let (_t, _remote, a, b) = fixture();
        commit(&a, "two");
        sh(&a, &["push"]);
        let head = sh(&b, &["rev-parse", "HEAD"]);
        let tracking = sh(&b, &["rev-parse", "origin/main"]);
        let steps = run(&b, true, true);
        assert!(steps.iter().all(|s| s.status == SyncStepStatus::Skipped));
        assert_eq!(sh(&b, &["rev-parse", "HEAD"]), head);
        assert_eq!(sh(&b, &["rev-parse", "origin/main"]), tracking);
    }
}

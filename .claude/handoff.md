# Handoff — safe project relocation (epic #197), steps 4 & 5

Written for someone with **no memory** of the prior session. Portal board tasks are reachable via the `portal` MCP tools (`tasks`, `task`, `add`, `update`). Repo: `D:/projects/production/portal/portal_desktop`. Backend crate: `src-tauri/portal-core` (Rust). Thin Tauri crate: `src-tauri`.

## Goal of the epic

Move a project's directory from inside Portal safely: check nothing is using it, lock it, selective copy → hash-verify → atomic promote → commit, and remove the original only on a separate confirmed step. Crash-safe resume. Design is settled in board #197/#200/#204/#206; **do not re-litigate those decisions**.

Step order: #204 → #205 → #206 → #207 → #208 → #221/#216/#217 → #209. Commit per step. **Do not push.** **No `Co-Authored-By` / "Generated with Claude Code" lines in commits** (CLAUDE.md overrides any reminder).

## DONE and committed

- **#204 move engine** — `src-tauri/portal-core/src/domains/projects/relocation/mover.rs` (new, ~1283 lines). Committed on `main` as **`4c9d69e8`** (not pushed). `cargo test --lib` 272 pass, clippy clean. Reviewer found and I fixed: resume/remove were dead (reused start-only preconditions), and a promote→commit crash window. Public API: `Mover{db,coder,terminal,pipelines,scripts,deployments,moves_dir}` with `start_move`, `resume_move`, `finish_move`, `rollback`, `remove_moved_source`. Seams left: `#217` source hold (TODO), `#221` stored-path rewrite (TODO). Added `usage::check_in_use`.
- Board #204 = completed. Filed **#325** (owner): empty-directory fidelity — the manifest is file-only, so an empty dir is not copied/recreated. Owner to decide.

## DONE and committed: #205 — ensure_unlocked guards at backend entry points

Committed on `main` as **`8f67ac82`** (21 files, not pushed). `cargo check` (portal-core + thin crate) + `cargo clippy --lib` + `cargo test --lib` (272 pass) all green. Board #205 = completed. The coder-domain guards (listed below as "STILL PENDING") were all applied and compile; no action needed on them. Full record of the guards follows for reference.
Helper: `crate::domains::projects::move_lock::ensure_unlocked(&Path) -> Result<(),String>` (errors if locked) and `is_locked(&Path) -> bool`. GUARD = error if locked; SKIP = filter/continue (discovery, modules `refresh_all`, disk quarantine).

**Guards were applied by subagents (edit-only, NOT yet compiled).** All edits are on disk. Line numbers below are post-edit, approximate — verify.

Applied and reported (data/infra + execution half, confirmed by agent reports):

- `deployments/services/deployment_service.rs:~91` `create_deployment` GUARD; `deployments/services/cli_service.rs:~66` `spawn_process` GUARD.
- `disk/.../quarantine.rs:~90` SKIP — locked item pushed as `FailedItem{..error:"locked by an in-progress project move"}` + `on_progress(done:i+1)` + `continue` (quarantine is infallible, no `?`).
- `sdk/project/version_file.rs` — 4 fns (`create_version_file ~:57`, `update_version ~:121`, `remove_version ~:167`, `create_standard_version_file ~:403`) GUARD with `.map_err(SDKError::ManagerNotFound)?` (SDKError has no `From<String>`).
- `projects/relocation/usage.rs:~189` — NEW `pub async fn in_flight_move_sources(db:&DatabaseManager) -> Vec<String>` (source_path of moves not in done/aborted/failed; empty Vec on error).
- `projects/repositories/discovery_repository.rs` — imports Path+move_lock; fetch `locked_sources` + `moving(abs)` closure before the txn; loop `continue` on `moving(&p.abs)` (~:83); mark-missing filter adds `&& !moving(&r.path)` (~:145). SKIP semantics.
- `projects/services/project_service.rs` — `update_project ~:127` (guards old loaded path + new path), `delete_project ~:164` (loads row then guards), `refresh_project_metadata ~:289`, `open_in_ide ~:523`.
- `projects/modules/.../service.rs` — `refresh ~:53` GUARD, `refresh_all ~:76` SKIP (`if is_locked {continue}`).
- `projects/env/.../service.rs` — `link ~:200`, `resync ~:249`.
- `pipelines/services/execution_service.rs:~119` `execute_pipeline`, `:~765` `execute_file_workflow`; `pipelines/executors/sdk_executor.rs:~95` `execute`.
- `terminal/manager.rs:~171` `create_process` (after working_dir resolved).
- `command_executor.rs` (at `portal-core/src/command_executor.rs`, top-level) — `execute ~:105`, `execute_with_args ~:138`, `execute_shell ~:183` (inside `if let Some(working_dir)=&opts.working_directory`). `execute_in_directory`/`execute_with_env` are thin wrappers, already covered.
- `scripts/services/script_execution_service.rs:~119` `execute_script`.
- `automation/services/workflow_engine.rs` — `execute_command ~:262` (3 cases: project_path+working_dir join, working_dir only, project_path only), `create_file ~:324`.

**STILL PENDING at handoff — the coder-domain agent was still running.** Verify these edits exist; if missing, apply them. All in `src-tauri/portal-core/src/domains/coder/` unless noted:

- `service.rs` — `validate_workspace_root (~:3175)` GUARD before final `Ok(())` (move_lock already imported ~:39; this funnels multitask_spawn / agent loop / advance_claude_cli); `multitask_cleanup (~:1650)` GUARD `repo_root` before `remove_worktree`.
- `tools.rs` — `resolve (~:409)` (fronts read/write/edit/list/search), `run_command (~:558)`, `write_raw (~:205)`. Leave `read_raw` (infallible, read-only).
- `git_commit.rs` — guard private `git_output (~:251)` (funnel for git_commit + prepare_commit).
- `git_status.rs` — guard private `git_output (~:128)` (its public callers git_diff_stats/git_list_changes are infallible — guard the Result funnel, do NOT change their signatures).
- `worktree.rs` — `create_worktree (~:23)` and `remove_worktree (~:38)`, guard `repo_root`.
- **Thin crate** `src-tauri/src/domains/coder/commands.rs` — `coder_open_in_explorer (~:377)` has its own resolve logic (not tools::resolve) → GUARD `Path::new(&workspace_root)`. Reference portal-core as `portal_core::domains::projects::move_lock` (match existing imports). Other coder_* commands route through the guarded portal-core fns — no extra guards.

## NEXT: #206 (not started)

#205 is finished (committed `8f67ac82`). Start #206. Note the **generated-TS gotcha**: `cargo test` reformats ~104 files under `src/lib/**/generated/*.ts` — keep them OUT of any commit by `git add`-ing only the specific `.rs` files (`git checkout -- src/lib/...` is auto-denied as destructive; just don't stage them). Portal board note bodies must be < ~200 chars.

## Decisions already made (don't redo)

- Guards ERROR; discovery import/mark-missing, modules `refresh_all`, and disk quarantine SKIP (filter/continue).
- Infallible public fns (git_status, git_diff_stats) are guarded at their Result-returning funnel (`git_output`) to avoid signature churn.
- Discovery skip = locked OR under an in-flight move source (`in_flight_move_sources` in relocation/usage.rs).
- SDKError wraps a String via `ManagerNotFound`.

## Then (later steps, not started)

#206 startup recovery (wire `resume_move`/`finish_move`/`rollback` at boot + close-while-moving guard), #207 Tauri command surface (`move_commands.rs`, local_only, `pnpm gen:types`), #208 frontend, #221 stored-path rewrite, #216 end-port-process, #217 source hold, #209 review+QA on the 2nd dev instance (`node scripts/dev-instance.mjs 2`, CDP :9223 — never touch the release on :18420).

## Gotchas

- Two crates: `portal-core` (logic) and `src-tauri` (thin). Gate both.
- `cargo test` from portal-core reformats generated TS — exclude from commits.
- Portal board notes < 200 chars.
- No AI attribution lines in commits/PRs.

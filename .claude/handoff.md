# Handoff — agent-platform → portal_desktop migration

Written for a fresh session with no memory of the prior conversation.

## Goal
Migrate the operator UI out of the separate **agent-platform** repo into
**portal_desktop**. `agent-platformd` (agent-platform's headless Rust AI server:
OpenAI-compatible proxy + multi-agent orchestration, loopback `:18410`) stays the
**AI brain** — other apps consume it; nobody re-implements its logic. Portal becomes
the operator **UI** and **owns the daemon as a managed sidecar**. The old native
**iced** operator app retires. Authority: agent-platform **ADR 0020**
(`D:/projects/production/ai/agentic-ai/agent-platform/docs/adr/0020-operator-ui-in-portal-desktop.md`).
Full plan: `docs/development/AGENT_PLATFORM_UI_MIGRATION.md` (this repo).

## Status
- **Plan doc**: written — `docs/development/AGENT_PLATFORM_UI_MIGRATION.md`.
- **Phase 1 (sidecar ownership): DONE, committed, NOT pushed.** Branch
  `feat/agent-platform-sidecar`, commit `6e3b8a5b`, 15 files, **no AI attribution**.
  Implemented, reviewed, and live-verified against the real `agent-platformd`
  (health 200, Attach on our key, Foreign (401) on a stranger's, DB created at our
  path). Release CI wired + verified against real GitHub.
- **Phase 2 (Settings parity): IN PROGRESS** in a separate session
  `phase2-settings` (id `local_d0e6511a-ab33-435f-8c43-fa5c1894ea59`), its own
  worktree on branch `phase2-settings-parity`, stacked on the Phase 1 branch. It is
  building the operator Settings screens in Svelte (Status/Logs first).

## What Phase 1 changed (all in commit `6e3b8a5b`)
- `src-tauri/portal-core/src/domains/ai/platform_daemon.rs` — **new, pure/testable**:
  `data_dir()`, `load_or_create_key()`, `daemon_env()`, `decide(PortProbe)->Action`,
  binary resolution. 4 unit tests.
- `src-tauri/src/domains/ai/platform_sidecar.rs` — **new, host side**: spawn the
  child, drain stdout/stderr to the log, `stop()` kills it. Handle is a static
  `Mutex<Option<Child>>`.
- `src-tauri/portal-core/src/domains/ai/platform_admin.rs` — **rewritten**:
  `start_platform(settings, spawn_fn)` probes `:18410`, then Attach / Foreign (report
  only) / Spawn. Dropped `exe_path`/`is_installed`/`RELEASES_URL`/`install_url` (the
  old "launch the installed app" model).
- `src-tauri/src/lib.rs` — setup registers the sidecar dir + spawns attach-or-spawn
  (non-blocking); `.build(context).run(|_,ev| if RunEvent::Exit { stop() })`.
- `src-tauri/src/domains/ai/commands.rs`, both `ai/mod.rs` — wire-up.
- `src/lib/domains/ai/types/index.ts` — removed `install_url`.
- `scripts/fetch-agent-platformd.mjs` — **new**: fetches pinned `agent-platformd`
  release per target triple into `src-tauri/binaries/`. Tag `v0.7.6` (verified
  latest). Asset names `agent-platform-server-<triple>.{zip|tar.xz}` + `.sha256`
  (verified against the real release). arm64-mac→x86_64 (Rosetta) map.
- `src-tauri/tauri.sidecar.conf.json` — **new**: `externalBin:["binaries/agent-platformd"]`,
  kept OUT of `tauri.conf.json` on purpose (a missing externalBin fails
  `cargo check`/`tauri:dev`); merged only at build via `--config`.
- `.github/actions/build-release/action.yml` — fetch step before tauri-action +
  `--config src-tauri/tauri.sidecar.conf.json` on the build.
- `package.json` (`tauri:build` fetches then builds), `.gitignore` (ignore
  `src-tauri/binaries/`), `README.md` (dropped "run in a second terminal").

## Decisions already made (and why)
- **Prebuilt daemon artifact, not build-from-source.** agent-platform ships
  `agent-platformd` per tag (cargo-dist, `packages=["agent-platform-server"]`); keeps
  the two builds decoupled (ADR 0020 "sidecar, not crate-link").
- **One canonical AI database = the existing agent-platform DB.** User: "one single
  database for AI stuff; only app-related stuff in the app's own db." Release portal's
  `data_dir()` = `dirs::config_dir()/com.tanvoid0.agentplatform` which on Windows is
  `%APPDATA%\Roaming\com.tanvoid0.agentplatform` — the **exact dir** holding the real
  `agent_platform.db` + `master.key`, so Phase 1 adopts them byte-for-byte. Portal's
  own SQLite keeps only app-local data (terminal, projects, disk, task board, pairing).
  Consolidating any AI data portal still keeps in its own DB into the AI DB is a LATER
  phase, not done.
- **Dev build uses a `.dev` data_dir sibling** (`com.tanvoid0.agentplatform.dev`) so a
  debug portal never corrupts the release AI DB (matches portal's own dev/release split).
- **Master key**: reuse `<data_dir>/master.key` if present (keeps existing `agp_` tokens
  valid), else generate 32 random bytes hex via `OsRng`. Key is env-only for the daemon
  (`AGENT_PLATFORM_MASTER_KEY`), never stored in the DB.
- **iced app retires** — user confirmed they won't run it once it ships inside portal.
- **No AI attribution** in commits/PRs — CLAUDE.md rule, overrides the harness's
  `Co-Authored-By` reminder.

## What's left
- **Phase 1 is unpushed.** Decide: push `feat/agent-platform-sidecar` + open PR (CI is
  wired so the daemon bundles), or keep local. User was asked; chose to start Phase 2
  instead of PR — so **not yet decided**.
- **Phase 2** — the `phase2-settings` session is building it. Settings screens to build:
  Status/Logs (biggest gap — portal only has banners today), Model-ops console,
  Performance/resource-mode knob (agent-platform ADR 0010), API/OpenAPI + key panel,
  Providers to full parity. Against `/api/v1` + live `/openapi.json`.
- **Not runtime-verified** (deferred, each needs setup):
  - Full release-CI run — needs tagging a release; the fetch/bundle path is wired but
    never executed end to end in Actions.
  - Full-app launch — portal's own process spawning the daemon + reaping on close.
    Needs freeing `:18410` (the installed app's daemon currently holds it) + a built or
    fetched `agent-platformd`. All its dependencies are independently proven.
- **Later phases** (plan doc §5): Phase 3 Runs/Teams/Workflows, Phase 4 retire iced in
  agent-platform, Phase 5 Studio/Plans/Agenda/Search/Memory/voice.

## Gotchas that bit us
- **Fixed port 18410 for both dev and release daemon.** A dev portal (`.dev` key) sees a
  running release daemon as **Foreign** and won't share the port. Known wrinkle, not yet
  solved — a dev daemon on a different port would need the AI provider base_url to follow.
- **`cargo test` in portal-core regenerates ~120 ts-rs `.ts` bindings** (unformatted).
  Prettier them back; never commit the churn. (Phase 1's run did NOT dirty them — verify
  before any commit.)
- **Pre-existing uncommitted frontend edits** live in the main checkout's working tree
  (sidebar/coder `.svelte`, `host-stats.svelte` rename, `routes/ai/coder/*`). They are
  NOT part of this migration — keep them OUT of migration commits. The `host-stats.svelte`
  rename was already **staged** pre-session; had to `git restore --staged` it before the
  Phase 1 commit.
- `platform_admin.rs`'s OLD doc comment said "Portal must never spawn the daemon, like
  Docker Desktop not dockerd" — that stance is **reversed** now; don't be misled by any
  stale copy.
- No `aarch64-apple-darwin` daemon asset exists upstream → the fetch maps it to the
  x86_64 build (Rosetta).

## Reference (agent-platform repo, read-only)
`D:/projects/production/ai/agentic-ai/agent-platform` — ADR 0020 (`docs/adr/`), the env
contract and sidecar manager we ported (`desktop/crates/app/src/shell.rs`), the iced
screens to re-implement in Svelte (`desktop/crates/app/src/{providers_view,modelops_view,
apidocs_view}.rs`, and `status_view`/`performance_view` in `screen.rs`). Daemon's
`/openapi.json` (live on `:18410`) is the API contract source of truth.

## To resume
Start a fresh session with **"read .claude/handoff.md"**. Decide Phase 1 push/PR; check
on the `phase2-settings` session; pick up Phase 2 or later phases from the plan doc.

# Handoff: board loop pass, devices redesign landed, vault review next (2026-11-22)

Repo `D:\projects\production\portal\portal_desktop`, branch `feat/agent-platform-sidecar` at `d9cdf955`, six commits above `origin/main` (`c91c5767`). Local `main` was fast-forwarded to the same commit after a rebase on a freshly fetched `origin/main` (no-op) and green gates. Nothing pushed, working tree clean. The older agent-platform migration handoff that lived in this file is kept below the divider, unchanged; ignore it unless you are on that workstream.

## Goal

Work the Portal board for this project in a loop, the `/crew:work` way: take the first open task not assigned to `owner`, claim it, route it through the crew agents (`crew:scout`, `crew:planner`, `crew:coder`, `crew:reviewer`), run the gates, close it with a note. The board is the `portal` MCP tools (`tasks`, `task`, `add`, `update`); pass the repo path as `project`. Tasks assigned to `owner` are skipped.

## Done this session

- `#1278` Extend button plus the devices-UI redesign it lives in: reviewed by `crew:reviewer` (8 findings: 1 high, 2 medium, 5 low), all fixed by `crew:coder`, re-checked by the reviewer against the real files: ship. Gates green on the rebased tree (`cargo check`, 17 pairing tests, `pnpm check` 0 errors, `pnpm lint` exit 0, vitest 282 tests); `#1278` is closed on the board.
- Landed as `adccbe20`, one 35-file commit made at 19:16 by someone else (the owner or the app's commit action) that swept in the whole dirty tree: devices redesign, review fixes, the vault prompter, and the three small edits below. The branch was rebased onto `origin/main` right after.
- `#1062` PORTAL_MOBILE.md Phase 5 status, `#1119` `scripts/dev-instance.mjs` (instance N>1 now defaults cargo dirs to `I:/build/cargo/portal-desktop-instance-N`, caller env wins), `#1101` CLAUDE.md icon line (FontAwesome `~icons/fa6-solid`, not lucide): completed.
- `#1064` trackpad page and `#1063` remote-input decision: cancelled, superseded by `9510386c` (input injection removed, no remote control by any client).
- Parked on `owner` with BLOCKED notes naming the exact ask: `#542` (needs `#643` real-phone alert check and `#644` Phase 3b decision), `#186` (`#194`: retire the agent-monitor import? and neither `codex` nor `cursor-agent` is installed, so those adapters never ran live), `#170` (needs a `content:read+write` token, the call on 4 private repos, the demo flow).
- Added `#1381`: review the vault-gate work.

## Left

1. Nothing to redo for `#1278` or the merge: gates are green on the rebased tree and `main` equals the branch. To re-verify after any new commit: `cd src-tauri && cargo check`; `cd src-tauri/portal-core && cargo test pairing_service` (17 pass); `pnpm check` (0 errors, about 120 warnings); `pnpm lint` (exit 0, 0 errors, about 3194 warnings, eslint takes about 7 minutes); `pnpm exec vitest run` (56 files, 282 tests).
2. `#1381` (claude): run `crew:reviewer` on the vault-gate code that went into `adccbe20` unreviewed. Check: can any command hold a lock or DB transaction across `VaultSession::require_key`; the 120 s timeout path; every caller of the removed `RETRY_AFTER_UNLOCK_COMMANDS` list in `invokeClient.ts` still handles `Vault is locked`.
3. Next board picks, all low bugs and unclaimed: `#1090` (`mobile/src-tauri` has no `.cargo/config.toml`, so cargo builds land on D:; add `target-dir` under `I:/build/cargo/portal-desktop-mobile`, CI does not build mobile today), `#1095` (CoderPanel squeezes chat at 1280), `#1094` (`/control` `X-Portal-Device` renames any paired device on every connect).
4. Wart: `adccbe20` includes cargo-test churn in `src-tauri/portal-core/bindings/serde_json/JsonValue.ts` (pure formatting) and `src/lib/domains/projects/generated/EnvFileView.ts` (adds a doc comment; may be legitimate). Optional cleanup commit.
5. Known low leftovers from the devices review, deliberately not fixed: in `src/lib/domains/devices/components/PendingRequestRow.svelte` (about line 125) Allow stays disabled with no hint if the capabilities query never loads; a rustfmt nit on the `matches!` line in `set_lifetime`; svelte-check `state_referenced_locally` warnings at `DeviceDetailsDialog.svelte:32-33` (pre-existing).
6. Push nothing unless asked.

## Files that matter

- Devices UI: `src/routes/devices/+page.svelte`; `src/lib/domains/devices/components/` (`DeviceRow`, `DeviceDetailsDialog`, `PairRequestGate`, `PendingRequestRow`, `ReconnectQrDialog`, `IconAction`, `CapabilityChecklist`); `queries/deviceQueries.ts` (5 s poll while a request is pending); `utils/capabilityChecklist.ts`; `state/devicesUi.svelte.ts`; `api/deviceApi.ts`; mounts in `src/lib/components/layout/app-shell.svelte` and `appNotifications.ts`.
- Devices backend: `src-tauri/portal-core/src/domains/network/services/pairing_service.rs` (`extend_expiry`, `set_lifetime`, tests); `src-tauri/src/domains/network/commands.rs` (`extend_device_expiry`, `set_device_lifetime`); handler list in `src-tauri/src/lib.rs`.
- Vault gate (`#1381`): `src-tauri/src/domains/credentials/prompter.rs`; `src-tauri/portal-core/src/domains/credentials/services/vault_session.rs` and `credential_service.rs`; `src-tauri/src/domains/credentials/commands.rs` (`vault_unlock_settled`); `src/lib/domains/credentials/components/VaultGate.svelte` and `VaultUnlockDialog.svelte`; `src/lib/utils/invokeClient.ts`; the "Vault-gated commands" paragraph in `CLAUDE.md`.
- Docs: `docs/development/PORTAL_MOBILE.md`.

## Decisions already made

- The Extend button ships inside the redesign commit, not split out (owner call logged on `#1278`).
- No remote input from any client (owner commit `9510386c`), hence `#1063` and `#1064` cancelled.
- `extend_device_expiry` and `set_device_lifetime` carry no capability, so remote tokens are refused (`src-tauri/rpc-macro/src/lib.rs:69-71`, `src-tauri/src/daemon/mod.rs:583-595`). The reviewer confirmed.
- No Claude attribution line in any commit or PR (user's global CLAUDE.md beats the session reminder). Check commit bodies for `Co-Authored-By` before anything is pushed.
- A fixer never grades its own work: coder fixes, then the reviewer re-checks.

## What bit me

- The checkout is shared. Mid-session the dirty tree was committed and the branch rebased by someone else. I then applied a stale zero-context hunk (`git apply --cached --unidiff-zero`) onto the moved `HEAD`: it inserted a duplicate `set_device_lifetime` handler line and I committed it as `cf750b80`. Undone with `git reset HEAD~1`, working tree untouched. Before staging anything, run `git log -n 3` and `git status --short`. Saved as memory `concurrent-commits-in-shared-tree`.
- `set -e` does not stop on a failing `diff` inside an `&&` list; use `|| exit 1`.
- A PreToolUse hook blocks unbounded `git diff` and `git log`: always `-n N`, `| head`, or `--stat`.
- A trailing `cd ..` in a Bash call moves the session cwd; use a subshell.
- `pnpm lint` is `prettier --check . && eslint src && node scripts/check-shadcn-ui.mjs`; eslint (the slow part) only runs if prettier passes, and this file is prettier-checked.
- cargo: run from inside `src-tauri/portal-core/` (it is not in `src-tauri`'s workspace) so the `I:` target-dir applies. `cargo test <filter>` does not churn the ts-rs bindings; an unfiltered run does (prettier `--write` them back, never commit them).
- Do not read subagent output files (full transcripts, they flood context).

## Phone and daemon facts (carried over from the 2026-11-20 handoff, which is no longer in this file)

Needed for `#643`, the real-phone alert check.

- The shared release daemon listens on `:18420`. Its LAN listener (`0.0.0.0`) is bound only at startup, and only when `remote_lan_enabled=true` in `%APPDATA%\portal_desktop\settings.json` (`app.remote_lan_enabled`). Toggling it without a full process kill does nothing, and closing the window only trays the app.
- Loopback calls with no `Origin` header get the full Local pass, which is how you list and approve devices with curl. `approve_device` args wrap as `{"request":{...}}`. `/pair/request` takes `{"device":"<name>"}` and returns `{token,port,name}` on approve, 403 on deny, 408 after 170 s.
- Android 16 needs the `ACCESS_LOCAL_NETWORK` runtime permission to reach a LAN daemon (fixed in the "enable LAN pairing on Android 16" commit: manifest, `MainActivity.kt`, and `mobile/android.mjs` re-patches both on every build; `gen/android` is gitignored).
- Debug APK build: `node mobile/android.mjs build --apk --debug --target aarch64` with `JAVA_HOME=C:\Users\tanvo\.sdkman\candidates\java\21-tem` (not the default Java 25, it breaks AGP) and `NDK_HOME=D:\sdk\android\ndk\28.2.13676358`.
- Git Bash mangles `/sdcard/...` adb paths; use PowerShell for `adb pull` and `screencap`, or `adb exec-out screencap -p`. adb to the S26 (SM-S948B) is wireless TLS.

---

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

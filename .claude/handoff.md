# Handoff — `agents` domain shipped; coder's claude-cli engine now shares it; one infra gap left

Written 2026-11-11, for someone with **no memory** of this session. Supersedes the previous
version of this file (about the `agents` domain's P1 review-fix + live-QA pass). That work is
now **committed** — see git history (`cd674366`) if you need the detail; this handoff doesn't
repeat it.

## Goal

Same overall goal as before: bring the new `agents` domain to production quality, then extend
its value by having coder's own claude-cli engine reuse it instead of maintaining a second,
weaker CLI-spawn path. This session did the second part end to end: scoped it, implemented it,
found and fixed two live bugs it surfaced, verified, committed.

## State: **committed**, two commits this session

- `cd674366` — the `agents` domain P1 review-fix pass + QA-110/111/112 fixes (previous
  handoff's work, finally landed).
- `268121af` — coder's claude-cli engine (`advance_claude_cli`) reworked to call the `agents`
  domain's `ClaudeCodeBackend` directly instead of its own one-shot CLI spawn. Includes two live
  bugs this surfaced (QA-119, QA-120), fixed and verified. **Nothing pushed.**

### What changed in `268121af`

Before: coder's claude-cli engine spawned a fresh `claude -p` process per turn, mapped Portal's
`PermissionMode` straight onto the CLI's `--permission-mode` flag, and had **no live approve/deny
at all** — headless mode can't prompt, so Review mode silently auto-denied anything needing one.

After: `advance_claude_cli` calls `ClaudeCodeBackend::start` directly (one level under
`AgentRunService`, which has no subscriber API coder needs). Review mode now gets a real live
approval banner via the same permission-hook relay the `agents` Run launcher uses. Concretely:

- `agents/backend.rs`'s `StartParams` gained `permission_mode: String` and `isolate_settings:
bool`. Coder passes `isolate_settings: false` — **deliberate, tested**: `--setting-sources ""`
  was confirmed via a canary-file test to strip project `CLAUDE.md` governance (with it, Claude
  blindly followed a planted instruction; isolated, it read the file as data and refused,
  flagging it as a test). Coder's users rely on their real `CLAUDE.md`/hooks loading; the agents
  Run launcher's own isolation (`true`) is unchanged.
- `coder_approve` on a claude-cli thread now routes around `spawn_run`'s cancel-and-restart path
  (which would otherwise kill the in-flight process mid-decision) via a new
  `claude_cli_controls: Mutex<HashMap<thread_id, Box<dyn RunControl>>>` +
  `decide_claude_cli_approval`.
- Old raw-JSON parser (`parse_cli_event`/`cli_edit_targets`/`cli_tool_result_ids`) ported onto
  the typed `AgentEvent` path. File-change/diff tracking and session continuity (`--resume`)
  preserved.
- Frontend: `showRemember={!isCli}` (no remember-rule concept for claude-cli, same call as
  QA-112); `coderSession.svelte.ts`'s `onRunning` now does `rt.running = running && !rt.pending`
  so Accept/Reject stay clickable while a banner shows (QA-110-class fix, one layer deeper); new
  `canStop` prop on `AIComposerShell`/`CoderComposer` keeps Stop reachable while pending,
  independent of `running`.
- `ollama` and every other coder provider are untouched — they go through `advance_platform`,
  a completely separate branch (`service.rs`'s only provider check is `== CLAUDE_CLI_PROVIDER`).

### QA-119, QA-120 — found live, fixed, verified (qa-tester closed both)

- **QA-119**: reloading mid-approval silently dropped the banner, permanently stuck on "Running…
  Thinking…", with Stop unreachable either (composer swapped to Send because `running` was
  forced false while pending). Fixed: `canStop` prop (independent of `running`); new
  `claude_cli_pending: Mutex<HashMap<thread_id, PendingApproval>>` +
  `coder_get_claude_cli_pending` command so a reload re-fetches live pending state from Rust
  instead of only replaying events. Verified live, including that the restored banner's
  Accept/Reject genuinely resolves the original tool call (file gets written on accept), not
  just that buttons are clickable.
- **QA-120**: cancelling leaked the real `claude.exe` grandchild on Windows forever — `process.rs`'s
  `start_kill` only killed the `cmd /C claude` wrapper. A first tree-kill attempt
  (fire-and-forget `taskkill /T /F`) raced `Child::start_kill()`'s own immediate kill and usually
  lost. Fixed by running `taskkill /T /F` via `.output()` (blocking on the syscall, not the
  tree's exit) **before** `child.start_kill()`, so the wrapper is still alive for taskkill to
  enumerate. Shared code — benefits the `agents` domain too. Verified: 3x repro runs + UI-driven
  variants, zero leaked PIDs each time.

## What's left (in order)

1. **`agents.exec` capability gating — infra gap, filed as board task #7 (owner).** Decided
   coder's claude-cli live-execution path (now better: live-ask, not silent-deny) should require
   `agents.exec`, same as the `agents` domain's own run-start, not just the default `coder` cap.
   **No per-call capability hook exists to do this today**: `daemon/mod.rs`'s `rpc()` checks a
   remote caller's `Trust::Remote{caps}` against the command's one static cap via `rpc.rs`'s
   `capability(name)` _before_ calling `dispatch()` — `dispatch()` itself never receives caps.
   Worse: a **local** Tauri IPC call (the app's own frontend) never goes through `daemon/mod.rs`
   or `dispatch()` at all — `#[rpc]` expands to a plain `#[tauri::command]`, no trust/caps
   parameter in either path. Doing this right means plumbing `Trust`/caps through the
   `rpc-macro` proc-macro crate (`src-tauri/rpc-macro/src/lib.rs`), `rpc.rs`'s `dispatch()`, and
   `daemon/mod.rs` — and deciding what a local call (no remote-Trust concept) should be treated
   as. Real infra work, not a `commands.rs` edit. Until it lands: a device holding just the
   default `coder` cap can reach claude-cli's live execution path — **same access as before this
   session's change**, not worsened by it, just not improved either.
2. **`cargo test -p portal-core` compile failure — board task #6 (owner), unrelated to this
   session.** `tempfile` dev-dependency unresolved in several `#[cfg(test)]` modules. Confirmed
   pre-existing on `main` via `git stash` before this session's changes — don't let it get
   attributed to the claude-cli rework.
3. **Push, or don't — your call.** Both commits are local-only on `main`. Nobody's asked for a
   PR; this was worked directly on `main` per the existing pattern in this repo's recent history
   (see `git log`).
4. Continue the original phased roadmap (P2–P6: persistence/Sessions list, Board/Kanban reuse,
   Analytics/Workflows, Codex/Cursor adapters) — still not started, nothing new to report here.

## Decisions made, don't re-litigate

- **`--setting-sources ""` for coder: no.** Empirically tested (canary file), confirmed it drops
  project `CLAUDE.md` governance. Coder keeps real settings loaded; the `agents` Run launcher
  keeps isolating (unchanged, different tradeoff for a different feature).
- **`agents.exec` gating: wanted, not done — see "What's left" #1.** Don't silently drop this or
  reattempt it as a quick fix without reading that section; it needs cross-cutting infra work.
- **coder calls `ClaudeCodeBackend` directly, not through `AgentRunService`.** The service has no
  subscriber API coder needs (it fans events to a global ring buffer + `agents://event`, keyed
  by its own `run_id`) — one layer down is the right seam.
- Previous handoff's decisions (icon-import non-bug, QA-112's hide-not-build-remember call,
  paired devices can start agent runs, ambient-session watching not yet implemented) all still
  stand, unaffected by this session.

## Files that matter (this session's part)

- `src-tauri/portal-core/src/domains/agents/{backend.rs,claude_code.rs,process.rs}` —
  `StartParams` fields, stderr-on-empty-events, the QA-120 tree-kill fix.
- `src-tauri/portal-core/src/domains/coder/service.rs` — `advance_claude_cli` rewrite,
  `claude_cli_controls`, `claude_cli_pending`, `decide_claude_cli_approval`.
- `src-tauri/src/domains/coder/commands.rs` — `coder_approve`'s claude-cli branch,
  `coder_get_claude_cli_pending`.
- `src/lib/domains/coder/state/coderSession.svelte.ts`, `components/{CoderPanel,CoderComposer}.svelte`,
  `src/lib/domains/ai/components/chat/AIComposerShell.svelte` — `canStop`, `showRemember`,
  the running/pending invariant.
- `.claude/qa/issues/QA-119-*.md`, `QA-120-*.md` — both `verified`, closed.
- **Not this session's concern, don't touch without checking**: `src/app.css`,
  `src/lib/domains/devices/components/PairDeviceCard.svelte`, `src/routes/devices/+page.svelte`,
  `src/lib/utils/invokeClient.ts`, `.claude/handoff-devices-phone-qa.md`, and
  QA-107/108/109/113-118 — a **different, possibly still-active concurrent session** worked the
  devices/pairing domain in this same checkout, still uncommitted as of this session's end. Don't
  attribute those diffs to this work, don't revert them, don't "clean them up" as unrelated noise
  — check with whoever owns that session first.

## Things that bit this session

- `cargo test`/`cargo check` leaves generated TS bindings **unprettified** (ts-rs's raw
  single-line output, not the repo's multiline style) — this showed up as a huge, scary-looking
  diff across _every_ domain's `generated/*.ts` files (disk, github, projects, devices — nothing
  to do with this session's actual change). Fix: `pnpm gen:types` (regenerates + formats
  correctly). Confirmed via `git status` before/after that this reverted every unrelated file
  back to matching `HEAD` exactly — none of it needed committing. Already noted in the previous
  handoff; noting again because it bit this session too, for a bigger blast radius than last
  time.
- Leaked `claude.exe` processes from QA-120 testing/repro runs needed a manual `taskkill` from
  the owner — the sandboxed agent environment's auto-mode classifier blocks `Stop-Process`/
  `taskkill` calls as "Interfere With Workloads," even for cleanup of your own test artifacts.
  If you hit this, ask the human rather than trying to work around the block.
- Portal (the board MCP) was down for most of this session — board items got filed late, once it
  came back up. If `mcp__plugin_crew_portal__add` fails with a connection error, note it once and
  carry on; retry later rather than blocking on it.

---
id: QA-107
title: Agents → Run — every run hangs on "Starting…" forever; backend panics building the permission-hook router (axum `:token` route syntax)
status: verified
severity: blocker
kind: bug
area: agents/run
mode: tauri
found: 2026-11-11
---

## Use case

Start any Claude Code run from Agents → Run (the page's entire purpose).

## Steps

1. Open Agents → Run (`/agents/run`).
2. Pick "Claude Code", fill in a working directory and a first message.
3. Click "Start run".

## Expected

The run starts: the page switches to the conversation view, the agent begins
responding, and eventually either an approval banner appears (for a
tool call) or assistant text streams in.

## Actual

The button switches to "Starting…" and never recovers — no error, no toast,
no navigation to the conversation view, forever. `agents_start_run`'s
`invoke` promise on the frontend never resolves or rejects.

The backend log shows why: `spawn_permission_hook` in `claude_code.rs`
registers an axum route with the old `:param` syntax, which the axum version
in use rejects at router-construction time:

```
thread 'tokio-rt-worker' (37340) panicked at portal-core\src\domains\agents\claude_code.rs:297:10:
Path segments must not start with `:`. For capture groups, use `{capture}`. If you meant to literally match a segment starting with a colon, call `without_v07_checks` on the router.
```

`src-tauri/portal-core/src/domains/agents/claude_code.rs:297`:
```rust
let app = axum::Router::new()
    .route("/hook/:token", axum::routing::post(permission_hook_handler))
    .with_state(state);
```

This panics every single time `spawn_permission_hook` runs (i.e. every
`agents_start_run` call), which appears to take down the tokio task the
Tauri command is running on before it ever replies over IPC — hence the
frontend hangs on "Starting…" with no error surfaced, rather than seeing
`run.start()`'s existing `catch`/`toast.error` path.

This is a total blocker for the Agents domain: no run can ever start, so
the approval banner (Accept once / Accept & remember / Reject), interrupt,
cancel, and session delete requested for this sweep are all unreachable —
this is the one bug blocking all of it.

## Evidence

`.claude/qa/shots/qa-sweep-agents-run1-filled.png` — form filled, about to
submit.
`.claude/qa/shots/qa-sweep-agents-run1-approval-or-timeout.png` — stuck on
"Starting…" after 30s.
Backend log excerpt above (`pnpm tauri:dev` stdout), panic at
`claude_code.rs:297:10`, recurring on every attempt.

## Repro

`node .claude/qa/scripts/qa-sweep-agents-run1.mjs` — fills the launcher for a
throwaway cwd and prompt, submits, and waits up to 30s for either an
approval banner or the run ending; prints `saw approval banner: false` and
the page stays on "Starting…" forever. Confirm the panic by tailing the
`pnpm tauri:dev` stdout/log while the script runs.

## Fix

Root cause: `spawn_permission_hook` registered its axum route as
`"/hook/:token"` — the pre-0.7 axum path-param syntax. The axum version in
this workspace rejects `:param` at router-construction time (panics with
"Path segments must not start with `:`... use `{capture}`"), and this ran on
every `agents_start_run` call, so the panic happened before the Tauri
command could reply over IPC and the frontend hung on "Starting…" forever.

Changed `src-tauri/portal-core/src/domains/agents/claude_code.rs:297` from
`.route("/hook/:token", ...)` to `.route("/hook/{token}", ...)`. The handler
already extracts the token via `Path<String>`, which is unaffected by the
registration-syntax change. Grepped the whole `src-tauri` tree for other
`:param`-style axum route registrations (`\.route\(\s*"[^"]*:[a-zA-Z]`) —
this was the only occurrence, no other routes had the bug.

Checks: `cargo check` from `src-tauri/` — clean build, only pre-existing
unrelated warnings. `cargo clippy -p portal-core` — no warnings on the
touched file.

## Verification

2026-11-11, mode: tauri (real backend, instance-2, CDP :9223, webview :1421 —
instance-1 on :1420/:9222 was out of scope this pass and wasn't touched). Ran
`node .claude/qa/scripts/QA-107-reverify.mjs` against the "claude-code"
backend:

- Run 1 (shell tool call): "Start run" left "Starting…" immediately, an
  "Approval required / Bash" banner appeared
  (`.claude/qa/shots/QA-107-after-run1-approval-or-timeout.png`), clicked
  "Accept once" — tool ran, "Stop" (interrupt) appeared, clicked it and the
  composer returned to "Send a follow-up…" with the tool call collapsed
  (`.claude/qa/shots/QA-107-after-run1-interrupt.png`, run stayed alive, not
  killed), then "End run" cleanly ended it
  (`.claude/qa/shots/QA-107-after-run1-ended.png`, "Run ended" + "New run").
- Run 2 (file-write tool call, new run via "New run"): same — past
  "Starting…", "Approval required / Write" banner, this time clicked
  "Reject" — the agent's own next message correctly reported the write was
  blocked by a permission hook
  (`.claude/qa/shots/QA-107-after-run2-reject.png`), then "End run" ended it.
- No backend panic, no stuck "Starting…", no console errors either run
  (`app.mjs`'s console-error capture was empty both runs).

Confirms the `:token`→`{token}` fix holds: router construction no longer
panics, so `agents_start_run` replies over IPC and the full lifecycle
(approval accept, approval reject, interrupt, end run, start a second run)
all work. "Accept & remember" is correctly hidden on this page
(`showRemember={false}` in `+page.svelte`) — consistent with the other
concurrent session's QA-112 decision to not implement real persistence for
it here; not re-investigated per this task's scope.

Repro script added: `.claude/qa/scripts/QA-107-reverify.mjs` (uses this
session's own scratchpad dir as a throwaway cwd — the original
`qa-sweep-agents-run1.mjs`'s hardcoded cwd was a different, no-longer-present
session's scratchpad path, so this is a fresh script rather than an edit to
that one). Verified.

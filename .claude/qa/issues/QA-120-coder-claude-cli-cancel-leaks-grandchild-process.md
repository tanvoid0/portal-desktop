---
id: QA-120
title: Cancelling a claude-cli run only kills the `cmd /C claude` wrapper — the real claude.exe process (and its open workspace cwd) is left running forever
status: verified
severity: major
kind: bug
area: ai/coder (claude-cli engine) / agents/backend (shared RunControl::kill)
mode: tauri
found: 2026-11-11
---

## Use case

Stopping a claude-cli coder run (button, or indirectly via QA-119's
reload-recovery path) should actually end it — no lingering process, no
workspace directory held open.

## Steps

1. Isolated dev instance (`node scripts/dev-instance.mjs 2`, CDP `:9223`).
2. `ai/coder` → claude-cli engine, Review mode, any writable workspace.
3. Ask for a Bash tool call, wait for the live approval banner (confirms the
   process is alive and blocked on the hook relay).
4. Cancel the run (`coder_stop`, or "Stop agent" once reachable — see
   QA-119).
5. Check the OS process list for `claude.exe` processes whose command line
   references Portal's `...\Temp\portal-agents\hook-*.json` (the settings
   file this engine always passes).

## Expected

No `claude.exe` process remains after the cancel. The workspace directory is
free to delete/rename immediately afterward.

## Actual

`coder_stop` reports success and `coder_list_running` goes back to empty
immediately — but the real `claude.exe` process **keeps running**,
confirmed via `Get-CimInstance Win32_Process`. Root cause:
[`process.rs:17`](../../../src-tauri/portal-core/src/domains/agents/process.rs#L17)

```rust
pub fn start_kill(child: &mut Child) {
    let _ = child.start_kill();
}
```

...own doc comment already says why:

> On Windows both this domain and coder's `claude-cli` path spawn through
> `cmd /C claude` (the npm shim is a `.cmd`, which `CreateProcess` can't run
> directly) — same pre-existing `start_kill`-only behavior as coder here,
> not a tree-kill.

`child` here is the `cmd.exe` wrapper. `start_kill` terminates `cmd.exe`;
`claude.exe`, started *by* `cmd.exe`, is never in Portal's direct process
tree so Windows does not tear it down too. The real CLI process — and
whatever file handles/cwd it holds in the workspace — is simply orphaned.

Confirmed live: three `claude.exe ... --settings ...\portal-agents\hook-*.json`
processes from earlier cancels in this same session were still running
*minutes* later, one of them holding this session's QA fixture workspace
open tightly enough that `rmdir` on it failed with "Device or resource busy"
even after Portal reported zero active runs.

## Why it matters

This exact `cmd`-wrapper behavior is **pre-existing** (confirmed: the old,
pre-rework `coder` claude-cli code — `git show HEAD:...service.rs` before
this session's diff — spawned the identical `cmd /C claude` way). It was
low-impact there because old Review mode auto-denied anything that would
prompt, so a `claude -p` call never blocked for long and cancelling one
mid-flight was a rare, short-lived race. This session's rework changes that:
Review mode threads can now sit genuinely blocked for minutes (a human has
to notice and decide), which makes "the user gives up and clicks Stop (or
reloads, see QA-119) while a tool call is still pending" a routine path
rather than an edge case — and every time it happens, it leaks one real
`claude.exe` process for the rest of the app's lifetime. Over a working
session this accumulates: this sweep alone left 4 confirmed orphans behind
(PIDs 46192, 36004, 33608, 44644) that I could not clean up — my shell tools
are blocked from `taskkill`/`Stop-Process`; the owner will need to kill them
by hand (Task Manager, or `taskkill /PID <pid> /F`) and remove
`.claude/qa/fixtures/QA claude-cli engine test/` once they're gone (the
directory itself is empty but still locked by one of them).

## Evidence

Console output from the repro below — process list before/after a cancel,
showing one new PID left running with no corresponding Portal-side run.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/QA-120.mjs` (Windows + PowerShell
required; point `WORKSPACE` env var at any writable folder) — snapshots
Portal-spawned `claude.exe` PIDs, starts a claude-cli/Review thread, sends a
Bash request, waits for it to become active, cancels it, and diffs the
process list. Prints `LEAK CONFIRMED: <pid>` when one is left behind, which
it currently always is.

## Fix

Root cause: `process::start_kill` only called `Child::start_kill()`, which on
Windows terminates the `cmd /C claude` wrapper but not its `claude.exe`
descendant (Windows doesn't tear down a dead process's children) — plus a
real timing trap once a tree-kill was added: a fire-and-forget `taskkill`
`spawn()` raced `start_kill()`'s immediate `TerminateProcess` on the wrapper,
and usually lost, so `taskkill /T` found its target pid already gone and
killed nothing. Fixed by running `taskkill /T /F /PID <wrapper pid>` via
`.output()` (blocking on taskkill's own quick syscall, not on the tree
actually exiting) *before* `child.start_kill()`, so the wrapper is still in
the live process list for taskkill to walk from.

Files changed: `src-tauri/portal-core/src/domains/agents/process.rs` only —
`start_kill()` is the single function both `agents/claude_code.rs`'s
`ClaudeCodeRun::kill()` and coder's claude-cli engine (which calls into the
same `ClaudeCodeBackend`/`ClaudeCodeRun` via `service.rs`'s
`advance_claude_cli`, confirmed: both route through this one function) go
through, so this is a one-fix-covers-both-callers change. Non-Windows path
untouched (no `cmd` wrapper there, so no grandchild to leak).

Checks run: `cargo check` (portal-core and full workspace) and
`cargo clippy -p portal-core` both clean, same pre-existing warning counts,
no new ones. `cargo test -p portal-core` could not run — pre-existing
compile failure in the test target (`tempfile` dev-dependency unresolved in
several `#[cfg(test)]` modules), confirmed present on `main` before this
change too (reproduced via `git stash`), unrelated to this fix; out of scope
here.

Live verification: `CDP_PORT=9223 node .claude/qa/scripts/QA-120.mjs` against
`node scripts/dev-instance.mjs 2` (instance 2 only, :9223/:1421 — instance 1
was never touched/running this session), run 3 times after the final fix:
all 3 reported `NEW leaked PID(s) from this run (expect none): []`, and a
live tree dump during debugging showed `taskkill /T /F /PID <wrapper>`
terminating the full 5-process chain (wrapper → … → `claude.exe`) in one
shot. A post-run scan for any `claude.exe` with `*portal-agents*` on its
command line turned up none. Does not fix QA-119 (separate issue, reload
recovery) — this is QA-120 only.

## Verification

2026-11-11, tauri mode, isolated instance 2 only (`:1421`/`:9223`;
instance-1 `:1420`/`:9222` confirmed not running throughout, untouched).

Baseline `Get-CimInstance Win32_Process -Filter "Name='claude.exe'"` (filtered
to `*portal-agents*` on the command line) was empty before this sweep.

Reran `CDP_PORT=9223 node .claude/qa/scripts/QA-120.mjs` **3 times**
(direct `coder_stop` invoke after a pending approval is reached): all 3
reported `NEW leaked PID(s) from this run (expect none): []`.

Wrote two independent checks the repro script doesn't cover, per the issue
asking to vary timing/entry point:
- Cancel via the actual **UI "Stop agent" button** while the approval
  banner is showing (not a direct invoke) — banner up, clicked Stop,
  `coder_list_running` empty within ~1.5s, no new `claude.exe` PID. PASS.
- Cancel via the **post-reload recovery path** (QA-119): banner up, reload,
  banner restored, clicked the restored "Stop agent" button — same result,
  no leaked PID. PASS.

Final process scan after all runs (QA-119 verification included, run in the
same sweep): zero `claude.exe` processes with `*portal-agents*` on their
command line. No orphans left for the owner to clean up this time (contrast
with the 4 PIDs this issue originally reported stuck).

Fix holds for both the plain cancel path and the two pending-approval
variants (UI click, post-reload). `status: verified`.

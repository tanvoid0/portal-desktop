---
id: QA-119
title: Reloading while a claude-cli tool call is pending approval loses the decision forever; thread shows a misleading permanent "Running… Thinking…", and there is no way to Stop a turn while the banner itself is showing
status: verified
severity: major
kind: bug
area: ai/coder (claude-cli engine, Review permission mode)
mode: tauri
found: 2026-11-11
---

## Use case

Coder (`ai/coder`) on the Claude Code (CLI) engine, Review permission mode: the
agent wants to run a tool (e.g. Bash), Portal shows a live "Approval required"
banner, the user decides Accept/Reject. This is the exact feature this
session's `advance_claude_cli` rework shipped (reusing the `agents` domain's
`ClaudeCodeBackend` so Review mode gets a real live banner instead of a
silent auto-deny).

## Steps

1. Isolated dev instance (`node scripts/dev-instance.mjs 2`, CDP `:9223`).
2. `ai/coder` → create a session on engine **Claude Code (CLI)**, permission
   mode **Review**, any writable workspace.
3. Ask it to run a Bash command (e.g. "Use your Bash tool to run `echo hi >
   marker.txt`"). Wait for the "Approval required" banner.
4. While the banner is showing, look at the composer: there is no "Stop" icon
   anywhere, only the (disabled) Send arrow.
5. Reload the page (`F5` / `page.reload()` — simulates the app restarting, or
   the webview reloading).
6. Observe the thread view and the sidebar.

## Expected

Either the banner survives the reload (reconstructed from backend state) so
the user can still Accept/Reject it, or the UI clearly shows the turn ended
(cancelled/needs retry) rather than pretending it is still in progress.
Either way, a reload should not make an in-flight decision undecidable, and a
working "Stop" affordance should exist whenever a turn — including one
paused on an approval — is actually running.

## Actual

- **No Stop control while the banner is showing** (step 4): `CoderComposer`
  only renders the "Stop agent" button when `running` is true, and this
  session's own QA-110-equivalent fix (`coderSession.svelte.ts`'s `onRunning`
  handler) intentionally forces `rt.running = running && !rt.pending` so the
  Accept/Reject buttons stay enabled. Side effect: there is no way to abort
  the whole turn while deciding — only Accept (run it) or Reject (skip just
  that one call; the turn then continues on its own).
- **Reload loses the pending decision silently** (step 5-6): the
  "Approval required" banner does not come back. The thread instead renders
  a permanent **"Running" / "Thinking…"** state — which is not true: the
  backend run is not making progress, it is still blocked inside
  `advance_claude_cli`'s event loop waiting for a decision on the exact tool
  call the now-vanished banner was showing. `coder_list_running` confirms the
  thread is still "active" indefinitely (checked for several minutes across
  this session; it does not time out on its own). The original tool call is
  never answered: not approved, not rejected, and Claude never finds out —
  it is simply stuck.
- The only way out is clicking **"Stop agent"**, which *is* visible again
  after reload (because the client no longer thinks a decision is pending,
  so `running` reverts to true) and *does* correctly cancel the backend run
  (`coder_list_running` goes back to empty within ~1s of clicking it,
  confirmed via `coder-check-stop-after-reload.mjs`). But nothing in the UI
  tells the user this is what's wrong or that Stop is what they need — the
  screen just says "Thinking…" forever.
- If the user never revisits that specific thread, the backend run — and the
  `claude` child process it owns — leaks for the rest of the app session.
  Sending a *new* message on the same thread does self-heal it (`spawn_run`
  cancels whatever old handle was registered for that thread id before
  starting the new one), so this is not a permanent leak if the thread gets
  reused, but it is an invisible one with no cleanup path otherwise.

## Why it matters

This is new surface area from this session's rework: the previous one-shot
`claude -p` spawn per turn never had a live, in-between-events "paused
awaiting a human" state that depends on transient, never-persisted frontend
state (`rt.pending`) to ever get resolved. Losing that state on reload (or if
the webview silently reloads, which WebView2 can do) means the exact
situation Review mode exists to handle — a destructive tool call awaiting a
decision — has no durable record of ever having been asked, and the turn
hangs with a misleading "still working" indicator instead of a clear error
or a recovered banner.

## Evidence

`.claude/qa/shots/QA-119-before-banner-up-no-stop-control.png` — banner up,
only a Send arrow in the composer, no Stop control.
`.claude/qa/shots/QA-119-before-after-reload-banner-gone.png` — same thread
immediately after reload: banner gone, header says "Running", body says
"Thinking…", sidebar shows the thread "Active" — none of which is true.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/QA-119.mjs` (point `WORKSPACE` env var
at any writable folder) — creates a claude-cli/Review thread, triggers a Bash
approval, confirms no Stop control while it's pending, reloads, confirms the
banner is gone while the backend run is still "active" per
`coder_list_running`, then cleans up via a direct `coder_stop` invoke (the
only way to end it once the page lost the pending state).

## Fix

**Bug A (no Stop while banner showing) — root cause:** `AIComposerShell.svelte`
only showed the Stop control when `running` was true, and claude-cli's own
QA-110-style fix forces `rt.running = running && !rt.pending`, so the two
conditions were mutually exclusive by construction — Stop could never show
while Accept/Reject could. Added a `canStop` prop (defaults to `running`,
back-compatible for `ChatInput.svelte`'s unrelated caller) so Stop visibility
can be keyed on "is there a live run" independent of "is a decision pending".
`CoderPanel.svelte` now passes `canStop={running || !!pending}`.

**Bug B (reload drops the banner) — root cause:** the platform engine's turn
actually *returns* when a tool call needs approval (so whatever transcript it
built lands in the DB, and `inferPending` reconstructs the banner on reload).
claude-cli's turn does not return — it stays open, blocked in
`advance_claude_cli`'s event loop, waiting on `decide_claude_cli_approval`
against the still-running process — so nothing about the wait was ever
persisted anywhere a reload could see. Added `CoderService::claude_cli_pending`
(`Mutex<HashMap<thread_id, PendingApproval>>`, alongside the existing
`claude_cli_controls` map): populated on `PermissionRequest`, cleared on
`PermissionResolved` or turn end. New command `coder_get_claude_cli_pending`
exposes it; `coderSession.svelte.ts`'s `selectThread()` calls it as a fallback
when `inferPending` finds nothing and the thread's engine is claude-cli, so a
reload re-fetches the live pending state instead of only replaying events.
Also fixed a latent corollary: `selectThread()` was setting `rt.running` from
`coder_list_running` unconditionally, but claude-cli's run stays "active"
for the whole wait — without `&& !rt.pending`, restoring the banner this way
would have re-disabled Accept/Reject (the exact regression QA-110 fixed).

**Files changed:**
- `src-tauri/portal-core/src/domains/coder/service.rs` — `claude_cli_pending`
  field + lifecycle, `claude_cli_pending_for()`.
- `src-tauri/src/domains/coder/commands.rs`, `src-tauri/src/lib.rs` —
  `coder_get_claude_cli_pending` command.
- `src/lib/domains/coder/services/coderService.ts` — `getClaudeCliPending()`.
- `src/lib/domains/coder/state/coderSession.svelte.ts` — `selectThread()`
  fallback fetch + the `running && !pending` invariant.
- `src/lib/domains/ai/components/chat/AIComposerShell.svelte`,
  `src/lib/domains/coder/components/CoderComposer.svelte`,
  `src/lib/domains/coder/components/CoderPanel.svelte` — `canStop` prop.

**Checks run:** `cargo check` (portal-core + src-tauri), `cargo test -p
portal-core` (63 coder tests, incl. the `claude_cli_tests` module) — all
clean. `cargo clippy --no-deps` on both crates — no new warnings.
`pnpm check` — 0 errors (pre-existing warnings only, none in touched files).
`pnpm lint` — eslint 0 errors and shadcn-ui check pass; `prettier --check`
fails only on ~100 pre-existing unformatted files already dirty before this
session (generated bindings etc., none of them files this fix touched —
verified `prettier --check` and `eslint` clean on every file this fix edited).
Live: reran `QA-119.mjs` against `node scripts/dev-instance.mjs 2` (CDP
:9223) — banner up, Stop button now present (previously absent), banner
survives reload, backend run still correctly tracked. Additionally wrote and
ran a one-off script (not committed) that accepts the approval *after* the
reload: banner clears, the Bash tool actually runs (marker file appears on
disk), and the thread leaves `coder_list_running` — confirms the restored
`call_id` still resolves against the live hook, not just that Stop works.

No other open QA ids touched by this fix.

## Verification

2026-11-11, tauri mode, isolated instance (`node scripts/dev-instance.mjs 2`,
vite `:1421`, CDP `:9223`; instance-1 on `:1420`/`:9222` was not running at
any point this sweep — confirmed via `netstat` before and after).

Reran `CDP_PORT=9223 node .claude/qa/scripts/QA-119.mjs`: Stop agent button
now present while the banner is up (was 0, now 1); banner present after
reload (was 0, now 1); backend run correctly still active after reload.

Wrote and ran an independent deeper check (not the fixer's own one-off,
mine) exercising the thing the Fix note claims but the repro script doesn't
itself assert — that the *restored* banner's Accept genuinely continues the
original run, not just that it's clickable:
- banner up pre-reload → reload → banner restored (1) → "Stop agent"
  reachable on the restored banner (1) → clicked **Accept once** →
  `qa-119-verify-marker.txt` written to disk with the exact expected content
  (`qa119verify`) → banner cleared → thread left `coder_list_running` on its
  own once the turn finished. PASS, confirms the restored `call_id` is live,
  not a dead end.

Also checked the QA-110-class invariant this fix depends on (Accept/Reject
clickable while pending, with no reload): the first `QA-119.mjs` run above
already shows a reachable Stop button while the banner is up pre-reload;
Accept/Reject were separately confirmed functional in the deeper check.
Checked `auto-accept-all` mode stays behavior-neutral: no banner appeared,
tool ran and wrote its marker file unattended, thread ended cleanly — no
regression from today's `canStop`/`claude_cli_pending` changes.

Both bugs A and B described in the issue hold fixed. Cleaned up all QA
fixture marker files and stopped every thread created. No `claude.exe`
leaked (see QA-120 verification below — checked jointly). `status: verified`.

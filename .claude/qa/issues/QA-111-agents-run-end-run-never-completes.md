---
id: QA-111
title: Agents → Run — "End run" kills the process but the UI never learns the run ended, so it's stuck on "Run in progress" forever with no way to reach "New run"
status: verified
severity: blocker
kind: bug
area: agents/run
mode: tauri
found: 2026-11-11
---

## Use case

Agents → Run's "End run" button (header bar, shown while a run is active)
is supposed to stop the run and let the user start a fresh one.

## Steps

1. Open Agents → Run (`/agents/run`), isolated dev instance.
2. Start any run (e.g. "Say hello and wait.").
3. Once it's running ("Run in progress" header, "End run" button visible),
   click "End run".

## Expected

`run.cancel()` → `agents_cancel` stops the backend process; the run's
`exited` event eventually arrives, `run.view.ended` becomes `true`, the
header switches to "Run ended" and the button becomes "New run" (per
`+page.svelte`'s `{#if run.view.ended}` branch).

## Actual

The header stays on "Run in progress" / "End run" indefinitely — polled for
60 straight seconds after the click with no change. Checked the backend's
own event stream directly (`agents_get_events`, bypassing the UI): no
`exited` event is ever recorded for that run either, in the same window.
So this isn't just a UI refresh glitch — the run genuinely never reaches the
"ended" state the frontend is waiting on.

At the OS level the process *does* appear to actually die shortly after the
click (its `cmd.exe`/`claude.exe` pair disappears from the process list
within a few seconds), so `cancel()` is doing something right — the problem
looks like it's between "the process is dead" and "an `Exited` event reaches
the ring buffer the frontend polls."

Suspect area: `process::kill_and_wait`
(`src-tauri/portal-core/src/domains/agents/process.rs`) does
`child.start_kill()` then `child.wait().await` while holding the lock on
`Arc<Mutex<Child>>` from inside `ClaudeCodeRun::kill()`
(`claude_code.rs:235-239`). The stdout-pump task that's supposed to detect
EOF and emit `AgentEvent::Exited` (`claude_code.rs:156-183`) needs the same
lock afterwards (`child.lock().await.wait().await` at line 181) to read the
exit code before sending `Exited`. I was not able to get a clean root-cause
confirmation (no panic output), but the shape of this — one task holding the
lock across an await that only resolves once the other task's own progress
depends on the same lock — looks like the kind of thing that could stall
indefinitely, or contend badly under load.

**Separately, and I can't rule out this is the same underlying issue:**
while testing this specific scenario (multiple runs started/ended across
repeated test passes against the isolated dev instance), the whole instance's
Rust backend process exited on its own (`target\instance-2\debug\portal_desktop.exe`
exit code 1), killing the whole dev session, with no panic message captured
in the log — just the bare "process didn't exit successfully" line from
`tauri dev`'s wrapper. I could not get a clean, isolated repro for the crash
by itself (it happened once, amid heavier concurrent testing), so I'm
flagging it here as a strong "investigate together" lead rather than filing
it as its own ticket with a repro I can't reproduce on demand. If the coder
fixes the lock/await ordering above, worth watching whether the crash stops
recurring too.

## Evidence

`.claude/qa/shots/qa-sweep2-endrun-after.png` — after clicking "End run",
header still says "Run in progress", button still says "End run".
`.claude/qa/scripts/qa-sweep2-cancel-raw.mjs` output: "clicked End run" at
`01:06:20`, polled `agents_get_events` every 0.5s for 20s afterward, found no
`exited` event and the UI never flipped either.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/qa-sweep2-cancel-raw.mjs` — starts a
run, clicks "End run", polls raw backend events for 20s for an `exited`
event. Prints `backend emitted exited event: false` and
`UI shows Run ended text: false`.

## Fix

Root cause (confirms the shape of QA's lead, on a different lock than
guessed): `ClaudeCodeRun::kill()` (`claude_code.rs`) held the per-run
`Arc<Mutex<Child>>` across its own blocking `child.wait()` (via
`process::kill_and_wait`) — and `AgentRunService::cancel()`
(`service.rs:189-193`) calls `kill()` while holding the *service-wide*
`runs: Mutex<HashMap<...>>` for the entire awaited call. That lock is the one
`start_run`'s per-run pump task (`service.rs:128-157`) needs to append every
event — including the terminal `Exited` — to the ring buffer `get_events`
reads. The other `RunControl` methods (`send_user`/`decide_permission`/
`interrupt`) are a quick stdin write or oneshot send, so holding `runs`
across them is fine (see the `ponytail:` comment at `service.rs:162-165`) —
`kill()` broke that assumption by being the one call that waits on the OS to
actually tear a process down, which stacked with the EOF-triggered `Exited`
in the stdout-pump task (`claude_code.rs`) never firing reliably right after
an explicit kill. Net effect: the terminal event either never reached the
ring buffer or was delayed indefinitely behind contention, so `run.view.ended`
never went true.

Fix: `ClaudeCodeRun::kill()` now only signals the kill
(`process::start_kill`, sync, lock held only for that instant — renamed from
`kill_and_wait`, which no longer waits) and immediately sends its own
`AgentEvent::Exited { code: None }` on the run's `events_tx` instead of
depending on the stdout-pump task's own EOF-triggered exit event. This makes
`kill()` resolve near-instantly, which also fixes `cancel()`'s long hold of
the service-wide `runs` lock as a side effect (no restructuring needed there).
The stdout-pump task still does its own reap-and-emit once it sees EOF for
the real exit code; the service pump loop breaks on the first `Exited` and
silently drops the (harmless) second one. `kill_on_drop(true)` on the spawned
`Child` is untouched — not a tree-kill either way, pre-existing behavior.

Files changed: `src-tauri/portal-core/src/domains/agents/process.rs`
(`kill_and_wait` → `start_kill`, no longer async/waits),
`src-tauri/portal-core/src/domains/agents/claude_code.rs` (`kill()` emits
`Exited` directly). `service.rs` untouched — its lock-holding pattern is now
safe given `kill()`'s new latency.

Checks: `cargo check` and `cargo clippy` from `src-tauri/` both clean (no
errors, no new warnings in the `agents` domain). Traced the event path by
hand: `AgentEvent::Exited` → `eventsToMessages.ts`'s `exited` case sets
`ended = true` regardless of `code` (already `code?: number | null` in the
generated TS type, no `pnpm gen:types` needed) → `+page.svelte`'s
`run.view.ended` branch flips the header/button. Also reran this ticket's own
repro live: a `tauri:dev` instance was already up on CDP `:9222` (auto-rebuilt
and relaunched off the file-watcher the moment these files were saved —
confirmed via the rebuilt `portal_desktop.exe`'s timestamp), so
`node .claude/qa/scripts/qa-sweep2-cancel-raw.mjs` against it, unmodified. New
result: `backend emitted exited event: true` at `+0.5s` (was `false` after
20s), `UI shows Run ended text: true` (was `false`), and the screenshot shows
the header reading "Run ended" with a "New run" button and the composer
placeholder "Run has ended." — the stuck state is gone.

No other open QA id references this exact path.

## Verification

2026-11-11, mode: tauri, isolated instance 2 (`CDP_PORT=9223`), independent
of the fix agent's own run.

**Single run** (`.claude/qa/scripts/qa-verify-111-single.mjs`): started a
run, clicked "End run" — backend `exited` event appeared at +298ms, UI
showed "Run ended" at +300ms, "New run" button present. Was stuck
indefinitely before the fix; now flips in well under a second.

**Concurrent check** (`.claude/qa/scripts/qa-verify-111-concurrent.mjs`):
Tauri exposes one page per window (confirmed in `app.mjs` — no second page
to drive a second UI from), so run A was driven through the UI (the path a
user actually clicks) and run B was started via direct `invoke` on the same
page — same `AgentRunService`/shared `runs` lock either way, which is what
the bug was about. Clicked "End run" on A; A's `exited` event landed at
+354ms. Polled B's raw event count through the cancel window: count rose
5→8 during/after the cancel, then held at 8 for ~5s before the window
closed. To rule out that 5s flat stretch being the bug rather than normal
model pacing, ran `.claude/qa/scripts/qa-verify-111-baseline.mjs` — same
"count slowly" prompt, single run, **no cancel at all** — and saw the
identical shape (events climb in a burst, then a ~7s flat stretch with no
new events). The flat stretch is the model's own turn-taking pace, not a
lock stall; B kept advancing across the cancel exactly as it would have
without it.

**Crash-lead stress test** (`.claude/qa/scripts/qa-verify-111-stress.mjs`):
8 rounds of starting 3 runs simultaneously and cancelling all 3 in quick,
overlapping succession (24 runs total), checking backend responsiveness
(`agents_list_backends`) after each round. All 8 rounds responsive; `netstat`
confirmed the instance-2 backend process (CDP :9223) was still listening
throughout and after. **The flagged crash did not recur** under this load.

Root cause fix (`kill()` now signal-only + emits `Exited` directly, no
longer waiting on the OS while holding the per-run lock) confirmed present
in `claude_code.rs`/`process.rs`.

Both the primary symptom and the secondary lock-stall effect are gone, and
the unreproduced crash lead did not resurface under deliberate stress.
**Verified.**

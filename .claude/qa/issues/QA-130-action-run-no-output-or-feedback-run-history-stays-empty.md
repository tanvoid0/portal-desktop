---
id: QA-130
title: "Projects > Actions: clicking an action's Run produces no visible output or state — Local run history stays \"No action runs yet\" and success/failure are indistinguishable"
status: fixed
severity: major
kind: bug
area: projects/actions
mode: tauri
found: 2026-11-11
---

## Use case

A developer clicks Run on an action (e.g. Install Dependencies / Test) and expects to see it
running, then its output and exit status.

## Steps

1. `/projects/201/pipelines` (Actions tab). Click an action's **Run** button (confirmed the
   click lands on a real `<button>` whose card contains the command).
2. Watch the "Local run history" panel and the action card for ~60-75s.

## Expected

A running indicator, streamed command output, and a terminal exit state (pass/fail) recorded
in Local run history.

## Actual

- After the click, **"Local run history" stays "No action runs yet"** for the full poll
  window; no inline output, no spinner, no error toast appears on the card. Success and
  failure look identical (nothing).
- (The underlying command likely died at the pnpm preflight gate — see QA-129 — but the UI
  gives zero signal either way, so the user can't tell a run started, failed, or never ran.)

## Why it matters

The Actions surface is the CI/CD "run and see the result" feature; with no feedback it can't
be used to diagnose anything, and a silent failure (QA-129) is indistinguishable from "didn't
click."

## Suggested fix

Show an immediate running state on Run, stream stdout/stderr inline (or into the project
Terminal with a clear link), and always record a run row with its exit code — loudly on
non-zero.

## Evidence

- Driven over CDP `:9223`; Run click registered (card matched), history unchanged after 75s.
- Screenshot of the empty history after Run in the session scratchpad.

## Fix

Root cause: `runTarget()` only called `openExecution()` + `loadHistory()` **after**
`actions.run()` resolved — i.e. after the whole run finished. With stdin inherited, a
pnpm prompt could block the process forever, so `run()` never resolved and the history
stayed empty with no signal. Two changes:

1. **Surface the run on spawn, not on finish.** `ActionRunOptions` gained an
   `onStepStart({ actionId, executionId })` callback; `actionsService.runActionList()`
   fires it the instant `executeScript` returns the id. The panel opens that execution and
   reloads history immediately, so a "running" row appears at once.
   (`src/lib/domains/actions/{types.ts,actionsService.ts}`,
   `src/lib/domains/actions/components/ProjectActionsPanel.svelte`)
2. **Stream live output while running.** A 1s-interval `$effect` polls
   `getExecution` + `getLiveOutput` for the selected running execution and renders the live
   buffer, flipping to the persisted terminal status (pass/fail + exit code) on completion.
3. **No more silent hangs.** The Rust runner now sets `stdin(Stdio::null())` so a process
   can't block on an unanswerable prompt — it fails fast and the row records the failure.
   (`src-tauri/portal-core/src/domains/scripts/services/script_execution_service.rs`)

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
Code + gates: `pnpm check` 0 errors, `pnpm lint` clean, `cargo check` clean. Live run-in-app
(CDP) verification still pending — needs a running dev instance. Expect: clicking Run shows a
running row with streamed output, then a pass/fail terminal state.

---
id: QA-151
title: Diagnose & fix shows only "Diagnosing…" for minutes while the agent waits on shell approvals; Coder link appears only after the run ends
status: verified
severity: major
kind: ux
area: actions/doctor
mode: tauri
found: 2026-11-12
---

## Use case

Owner clicks "Diagnose & fix" (Claude Code CLI backend, Review mode) and expects the agent to work or tell them what it needs.

## Steps

1. Dev build, aipairs (id 14) or Devstrail (id 4) -> Actions -> Run `Test` (default `pnpm test`, script missing) -> failure card.
2. Click "Diagnose & fix". Watch the card for ~40 s.
3. Open /ai/coder by hand and select the thread "A Portal Action command failed…".

## Expected

While the agent runs, the card links to its thread ("Open in Coder") and says when it is waiting for the owner ("Waiting for your approval in Coder"). A thread blocked on an approval should not just spin.

## Actual

- Card shows `Diagnosing…` with no "Review changes in Coder" link for the whole run (aipairs: 6 m 46 s; Devstrail: ~6 min). Cause (read in code, matches the unchanged card text): `diagnose()` in `state/doctor.svelte.ts` does `set({ threadId: r.threadId })` only AFTER `await runCoderTask(...)` resolves; same in `generatePipeline`. `runCoderTask` creates the thread first, so the id exists early.
- In the Coder thread the agent sits on `Approval required: Bash` / `PowerShell` (no command shown, see QA-126) several times in a row (I rejected 3-4 per run; it still finished). Nothing in Actions says so. A user who does not know to open /ai/coder sees a spinner indefinitely; there is no timeout.
- Generate finished in ~20 s on all three projects (it only used Read/Glob/Write); Diagnose explored with shells and blocked.

## Evidence

`.claude/qa/shots/QA-151-before-diagnosing-no-coder-link.png` (card: "Diagnosing…", no link), `sweep-doctor-coder-diag-thread-aipairs-dev.png` (pending Bash approval), `sweep-doctor-devstrail-thread-done.png`.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/qa-diag-full.mjs 4 Test QA-151` (it rejects the shell approvals itself; use a clean repo with no `test` script).

## Fix

Root cause: threadId set only after runCoderTask resolved. runCoderTask now takes onThread/onWaiting hooks; thread linked immediately ("Open in Coder") and "Waiting for your approval in Coder" shown on coder pending events until the next message. Diagnose and Generate. No timeout added. Files: src/lib/domains/actions/{doctor.ts,state/doctor.svelte.ts,components/ProjectActionsPanel.svelte,doctorFlow.test.ts}. Checks: vitest actions (35 pass), eslint 0 errors, prettier. Not run in live app.

Review follow-up: retries reuse the same coder thread (context kept, reviewThreadId no longer overwritten); Generate and Diagnose are disabled while the other runs; attempt counter cleared on success; per-project panel state (lastResult, requirement, selection) reset on project change.

## Verification

2026-11-12, mode tauri (dev build :9223). Diagnose on staged failure: at 5 s the card showed 'Diagnosing… · Waiting for your approval in Coder · Open in Coder' (QA-151-after-diagnosing.png); Generate on four_thousand done in 20 s. Residual (not blocking, see QA-156): after leaving the page for Coder and coming back, the failure card and Diagnosing status are gone; only the 'Agent changes · pending your review' card remains. Retry-reuses-thread not exercised (agent succeeded on first turn).

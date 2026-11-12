---
id: QA-156
title: Diagnosing status and the failure card disappear when you open Coder and come back to the project page mid-run
status: fixed
severity: minor
kind: ux
area: actions/doctor
mode: tauri
found: 2026-11-12
---

## Use case

Diagnose & fix is running and waiting for approval; the owner opens Coder (via "Open in Coder"), handles it, and returns to the project's Actions tab.

## Steps

1. aipairs -> staged failing action -> Run -> Diagnose & fix (card: "Diagnosing… · Waiting for your approval in Coder · Open in Coder").
2. Open Coder, then browser Back to /projects/14?tab=actions while the agent is still running.

## Expected

The same card/status (Diagnosing / waiting, Open in Coder) is still shown until the run ends.

## Actual

The "Run failed" card (component-local `lastResult`) is gone; no "Diagnosing" text. Only "Agent changes · pending your review in Coder" is visible, so it looks finished while the agent is still working (the proposal card appeared later). Doctor state lives at module level, but the card is gated on `lastResult?.failure`.

## Evidence

`.claude/qa/shots/QA-156-before-return-from-coder-card-gone.png` (copy of QA-151-after-return-from-coder-card-gone.png).

## Repro

manual — steps above.

## Fix

Root cause: the "Run failed" / Diagnosing card was gated on the panel's component-local `lastResult`, which is lost when the panel remounts (Coder → Back), while the doctor run lives at module level.

- `src/lib/domains/actions/state/doctor.svelte.ts` — `DoctorRun.failure`: set when `diagnose()` starts, updated to the latest failure on a red verify, cleared when the action passes (`fixed`) and by `resetDoctorRun` (a fresh manual run).
- `src/lib/domains/actions/components/ProjectActionsPanel.svelte` — card renders from `shownFailure = lastResult?.failure ?? doctorRun.failure` (same pattern as the QA-153 proposal card); the "Agent changes · pending your review" pointer hides while that card shows, so a running doctor no longer looks finished.
- Tests: `doctorFlow.test.ts` › "failure card survives leaving the page mid-run (QA-156)" — failure held while the agent works, dropped once fixed; a red verify keeps the new failure.

Checks (2026-11-12): `pnpm exec vitest run src/lib/domains/actions` 48/48, `pnpm check` 0 errors, `pnpm lint` exit 0.

Live (coder smoke test, 2026-11-12, dev build CDP :9223): aipairs with `typecheck` staged as `pnpm run typecheck-all` (file restored afterwards) -> Run -> Diagnose & fix -> "Diagnosing… · Open in Coder" -> Open in Coder -> Back. After Back: `Run failed | … | Diagnosing… | Waiting for your approval in Coder | Open in Coder`; the "pending your review" card was not shown. After Stop agent in Coder -> Back: the card showed "Gave up · Cancelled." Shots: `QA-156-after-diagnosing-before-leaving.png`, `QA-156-after-back-from-coder-card-closeup.png`. The agent never finished (the approval was stuck after Reject — Portal board bug #283), so I could not exercise the Fixed -> card-clears path live; the unit test covers it.

Also in this change (QA-153 follow-up): Verify now runs the exact resolved action it re-checked (`actionsService` `forProject().runResolved`), so a `.portal/pipeline.yml` write after the check cannot change what executes.

## Verification

<!-- qa-tester -->

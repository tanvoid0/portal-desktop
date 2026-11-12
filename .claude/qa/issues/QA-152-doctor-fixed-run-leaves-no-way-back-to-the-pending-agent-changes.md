---
id: QA-152
title: After a successful Diagnose & fix the failure card (status + "Review changes in Coder") vanishes, leaving unreviewed agent changes with no pointer
status: verified
severity: minor
kind: ux
area: actions/doctor
mode: tauri
found: 2026-11-12
---

## Use case

Diagnose & fix succeeded; the owner should now review the agent's file change (pending in the Coder gate).

## Steps

1. Devstrail (id 4) -> Actions -> Run `Test` -> failure card -> Diagnose & fix -> let it finish (verify run green).

## Expected

A persistent "Fixed · review the change in Coder" line with the thread link, whatever the fix kind.

## Actual

- Panel `lastResult` becomes the green verify run, so the whole "Run failed" card (status text and `Review changes in Coder` link) disappears. Only a transient success toast remains (gone when I looked) and, for fixes of a Portal *default* only, the amber "Portal default needs a fix" card, which has no Coder link. A fix of an existing file override or an app-code fix leaves nothing at all.
- The agent's change stays pending in Coder ("Agent changes 1 pending"), unlinked. Doctor state is in-memory, so a restart loses the link for good.

## Evidence

`.claude/qa/shots/QA-152-before-fixed-no-review-pointer.png`. Code: `ProjectActionsPanel.svelte` ~L576-587 (`lastResult = verify`); the thread link is rendered only inside `{#if lastResult?.failure}`.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/qa-diag-full.mjs 4 Test QA-152`.

## Fix

Root cause: link lived only in the failure card. Added doctor reviewThreadId (survives resetDoctorRun) and a persistent card "Fixed/Agent changes · pending your review in Coder" with link + Dismiss when no failure card shows. In-memory only (lost on app restart). Files: src/lib/domains/actions/{doctor.ts,state/doctor.svelte.ts,components/ProjectActionsPanel.svelte,doctorFlow.test.ts}. Checks: vitest actions (35 pass), eslint 0 errors, prettier. Not run in live app.

## Verification

2026-11-12, mode tauri (dev build :9223). After Verify the panel shows 'Fixed · pending your review in Coder' with 'Review changes in Coder' and Dismiss (QA-152-after-fixed-pending-review-card.png). Pointer also present on return from Coder.

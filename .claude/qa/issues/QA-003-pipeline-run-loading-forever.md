---
id: QA-003
title: Pipeline run page stays on "Loading execution..." forever when the execution fails to load
status: verified
severity: major
kind: bug
area: projects/pipelines/run
mode: browser
found: 2026-11-05
---

## Use case
Open a pipeline run (from a link/history) whose execution is missing or whose backend call fails.

## Steps
1. Open `/projects/1/pipelines/run/999`, wait 6s.

## Expected
An error / not-found state with a way back, like `/projects/1` and `/projects/edit/1` ("Failed to load ... Retry").

## Actual
Spinner text "Loading execution..." never goes away. Console shows `[ExecutionService] Failed to get execution` plus an uncaught `pageerror`. The page also does not use the project-not-found state the sibling pages show. In browser mode the failure is a thrown invoke; confirm in the real app whether a missing execution id hits the same path (it should: the monitor never leaves loading on error).

## Evidence
`.claude/qa/shots/QA-003-before-run-loading.png`. Component: `src/lib/domains/projects/pipelines/components/ExecutionMonitor.svelte`; page `src/routes/projects/[id]/pipelines/run/[executionId]/+page.svelte`.

## Repro
`node .claude/qa/scripts/QA-003.mjs`

## Fix
Root cause: `ExecutionMonitor` never handled a rejected/null `getExecution` (unhandled rejection in `onMount`, stuck loading). Added `loadError` state + shell `PageError` with Retry; load errors are caught, event listeners subscribe only after a successful load (retry re-attempts). Header now `PageHeader`. Files: `src/lib/domains/projects/pipelines/components/ExecutionMonitor.svelte`. Checks: `pnpm check` 0 errors; eslint clean on touched files; repro now shows 'Failed to load execution / Retry'. Browser mode only (no backend); real-app missing id (`null`) shows 'Execution not found.'.

## Verification

2026-11-05, browser mode (HMR build), 1280x800 and 1024x600. QA-003.mjs: 'Failed to load execution' + Retry shown, no uncaught pageerror. Message text is the raw 'Cannot read properties...' (browser-mode artefact; check real-app wording for a missing id: 'Execution not found.' per Fix). Verified.

2026-11-05 re-verify (PageLoading + guards), browser mode: error state + Retry present, Back while loading adds no run-page errors (only the destination Project page's own browser-mode invoke error), goBack clean. Still verified.
Screenshot: .claude/qa/shots/QA-003-after-error-retry.png
Earlier verify rounds: QA-003-after-error-retry-round-0/1.png, QA-005-after-run-left-edge-error-state.png

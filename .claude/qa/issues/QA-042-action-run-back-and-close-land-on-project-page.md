---
id: QA-042
title: Run detail opened from Action Runs: "Back to Actions" and "Close" land on the project page
status: verified
severity: minor
kind: ux
area: pipeline-runs
mode: tauri
found: 2026-11-06
---

## Use case

Browse the global Action Runs list (`/pipeline-runs`), open a run to read its log, go back to the list.

## Steps

1. Open `/pipeline-runs`, click a run (e.g. "Finalize test").
2. Click "Back to Actions" (or "Close").

## Expected

Return to the list I came from, with my search/status filter intact.

## Actual

Lands on `/projects/228?tab=actions` (the project's Actions tab). Filter and list position are lost. The detail page also lives under `/projects/<id>/pipelines/run/<uuid>`, so the breadcrumb reads Home > Projects > portal_desktop and the sidebar highlights "Projects" instead of "Action Runs".

## Evidence

`.claude/qa/shots/QA-042-before-back-to-actions-lands-on-project.png`, `sweep-pipeline-runs-row-clicked.png` (sidebar highlight).

## Repro

`node .claude/qa/scripts/QA-042.mjs`

## Fix

Root cause: back/close hardcoded to the project Actions tab; list filter was local state. Run link now carries `?from=/pipeline-runs?q=&status=`, detail page returns there (only `/pipeline-runs*` accepted), list restores filters from URL. Project-origin runs unchanged. Not done: sidebar/breadcrumb on the detail page still say Projects (route lives under /projects/[id]).
Files: routes/pipeline-runs/+page.svelte, routes/projects/[id]/pipelines/run/[executionId]/+page.svelte. Checks: pnpm check 0 errors, eslint 0 errors, prettier clean. Repro not run (CDP off-limits).
Follow-up (review): `from` is parsed with `new URL`, accepted only if same-origin and pathname === '/pipeline-runs'; status param validated against options; breadcrumb ignores from/q/status params.

## Verification

2026-11-06, tauri. Reran QA-042.mjs (button is now named 'Back to Action Runs'; script updated). From /pipeline-runs?q=Finalize, opening a run then Back and Close both return to /pipeline-runs?q=Finalize with the search restored. Sidebar/breadcrumb on detail still say Projects (accepted in Fix). After: shots/QA-042-after-back-to-action-runs.png. Result: gone.

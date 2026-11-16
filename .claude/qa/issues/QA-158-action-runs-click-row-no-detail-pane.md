---
id: QA-158
title: Action Runs page - clicking a run sets ?run= but the detail pane never appears
status: fixed
severity: major
kind: bug
area: pipeline-runs
mode: tauri
found: 2026-11-15
---

## Use case
Open Action Runs, click a run, read its output / Stop / Re-run in the right-hand detail.

## Steps
1. `/pipeline-runs` (dev instance, CDP 9223), a few runs listed.
2. Click any run row.

## Expected
Right-hand `ResponsivePanel` with `ActionRunDetail` ("Run details").

## Actual
URL becomes `?run=<id>` (back/forward also move the URL) but nothing renders: no "Run details", no "Run not found", no panel, at 1024/1280/3440 wide. Loading the same URL directly (`/pipeline-runs?run=<id>`) DOES render the detail. So the page's `selectedRunId` (`page.url.searchParams.get('run')`) does not update after `selectRun()`'s `pushState(url, {})` (`src/routes/pipeline-runs/+page.svelte` ~L96-101). Suspect: shallow-routing `pushState` does not update `page.url` in this SvelteKit version; use `goto(url, { noScroll, keepFocus })` or a local `$state` for the selection synced from the URL.

## Evidence
`.claude/qa/shots/QA-158-before-no-detail-pane.png`, `.claude/qa/shots/QA-158-before-direct-load.png` (direct load shows the pane). Probe prints `Run details in DOM: 0` after click, `1` after direct load.

## Repro
`CDP_PORT=9223 node .claude/qa/scripts/QA-158.mjs`

## Fix

## Verification

## Fix

`selectRun()` used `pushState` (shallow routing), which does not update `page.url`, so the
`?run=` selection never reached `selectedRunId`. It now navigates with `goto(url)` (same
route, component kept). Verified 2026-11-15 on dev instance 2 (CDP :9223, 1280px): clicking a
row sets `?run=<id>` and the Run details panel renders.

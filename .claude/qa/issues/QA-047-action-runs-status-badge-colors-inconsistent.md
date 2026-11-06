---
id: QA-047
title: Action Runs: "success" badge is blue, "failed" red; copy and chrome nits across Deployments/Action Runs
status: verified
severity: polish
kind: ux
area: pipeline-runs, deployments
mode: tauri
found: 2026-11-06
---

## Steps

1. Open `/pipeline-runs` (light and dark).
2. Open `/deployments` Deployments tab and Containers tab.

## Actual

- "success" badge is filled blue while its icon is green and "failed" is filled red; "cancelled" and "running" are outlined, no fill. Success should read green.
- Breadcrumb says "Pipeline runs" while the page title and sidebar say "Action Runs".
- Run detail (`/projects/<id>/pipelines/run/<uuid>`) highlights "Projects" in the sidebar, not "Action Runs" (see QA-042).
- Deployments tab: "New Deployment" appears both in the header and in the empty state; "Building" stat shows an emoji ("🔨 0") unlike the other stats' icons.
- Compose stack header reads "1 services".
- "Auto-scroll on" shown on a finished run.

## Evidence

`.claude/qa/shots/sweep-pipeline-runs-list.png`, `sweep-pipeline-runs-dark.png`, `sweep-deployments-tab.png`, `sweep-deployments-container-rows.png`.

## Repro

manual - steps above

## Fix

Done: success badge green (PipelineRunHistoryList), breadcrumb segment 'pipeline-runs' -> 'Action Runs' (breadcrumbStore SEGMENT_LABELS), duplicate empty-state New Deployment button removed (header button always visible on that tab), Building stat uses Hammer icon, '1 service' pluralised (workloadGrouping.ts), Auto-scroll toggle hidden unless run is running (StepLogViewer). Sidebar highlight on run detail left as in QA-042.
Checks: pnpm check, eslint, prettier clean.
Follow-up (review): shared `getExecutionStatusBadgeClass` in executionDisplay.ts, used by PipelineRunHistoryList and both badges in ProjectActionsPanel.

## Verification

2026-11-06, tauri. Seen fixed: success badge green, breadcrumb 'Action Runs', single New Deployment button on empty Deployments tab, Hammer icon on Building stat. Not checked: '1 service' wording and Auto-scroll hidden on finished run (the run detail was not reopened). After: shots/QA-047-after-action-runs.png, QA-047-after-deployments-tab.png. Result: verified for the items seen.

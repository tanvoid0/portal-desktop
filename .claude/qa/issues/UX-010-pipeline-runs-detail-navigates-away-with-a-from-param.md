---
id: UX-010
title: Opening a pipeline run leaves the list entirely and hand-rolls its own back link
status: fixed
severity: major
kind: ux
area: pipeline-runs
mode: tauri
found: 2026-11-06
---

## Use case

Scan the run history, open a failed run, read its output, go back, open the next one.

## Steps

1. Open `/pipeline-runs`.
2. Click any run.

## Expected

The run's output appears beside the list; the next run is one click away.

## Actual

The app navigates to `/projects/<projectId>/pipelines/run/<executionId>` — a different route
family under a different project — and carries the list state in a query string so it can get
back:

```js
// src/routes/pipeline-runs/+page.svelte:67-75
const back = `/pipeline-runs${params.toString() ? `?${params}` : ''}`;
goto(`/projects/${execution.projectId}/pipelines/run/${execution.id}?from=${encodeURIComponent(back)}`);
```

The `?from=` parameter exists only because the detail view cannot live next to the list. Every
run you inspect is a full page swap plus a manual return, and the search/filter you typed has
to be serialised and replayed.

(Related: QA-078 already filed the breadcrumb showing a raw query string — that is the same
`?from=` hack surfacing in the breadcrumb.)

## Fix — right rail

`/pipeline-runs` is 134 lines and already loads everything it needs
([src/routes/pipeline-runs/+page.svelte](../../../src/routes/pipeline-runs/+page.svelte)).

1. `handleSelect` sets `selectedRunId` instead of calling `goto`. Drop the `back` /
   `?from=` construction entirely.
2. Wrap the page in `ResponsivePanel side="right" desktopClass="w-[28rem]"`
   ([src/lib/components/shell/responsive-panel.svelte:8,30](../../../src/lib/components/shell/responsive-panel.svelte#L8)
   already takes `side: 'left' | 'right'` — working callers are
   `CoderPanel.svelte:829` and `ai/chat/+page.svelte:612`).
3. The rail renders the same component `/projects/[id]/pipelines/run/[executionId]` renders
   today: status, duration, step list, log output. Extract that page's body into
   `PipelineRunDetail.svelte` taking `{ executionId: string }` so both the rail and the route
   use it.
4. Keep the route `/projects/[id]/pipelines/run/[executionId]` working — it is a valid deep
   link and it is where a run opened from inside a project should still go. It just stops
   being the only way to read a run.
5. Mirror `selectedRunId` into the URL as `?run=<id>` so reload and back/forward keep the
   selection, without the `from=` round trip.

## Evidence

`.claude/qa/shots/ux/ux-pipeline-runs.png` (1280x800: list at chrome 84px, 1241px of rows).
Code quoted above.

## Repro

`node .claude/qa/scripts/app.mjs pipeline-runs ux-pipeline-runs`, then click a run and watch
the URL.

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

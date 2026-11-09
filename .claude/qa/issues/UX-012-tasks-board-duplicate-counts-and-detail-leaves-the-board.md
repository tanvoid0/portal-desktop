---
id: UX-012
title: Tasks board repeats its column counts as badges and loses the board when you open a task
status: verified
severity: minor
kind: ux
area: tasks
mode: tauri
found: 2026-11-06
---

## Use case

Work the board: scan columns, open a task, read it, move on.

## Measured (1280x800, real data)

The board already has the best layout in the app — a left project rail and a real column
view. Two things undo it.

**1. The counts are printed twice, 44px apart.**

Row 1 ([TaskManager.svelte:394-396](../../../src/lib/domains/tasks/components/TaskManager.svelte#L394)):

```
[Backlog 8]  [In Progress 3]  [Done this week 8]
```

Row 2, the column headers 44px below, in the same order:

```
Backlog  8        In Progress  3        Done
```

Deleting the badge row costs nothing and buys 44px.

**2. Columns are `min-w-[300px]` and there is no room for three.**

[KanbanBoard.svelte:107-110](../../../src/lib/domains/tasks/components/KanbanBoard.svelte#L107):
`overflow-x-auto` over `min-w-[300px] flex-1` columns. With the 290px project rail plus the
48px nav rail, the content width at 1280 is 870px — the third column is cut mid-card and a
horizontal scrollbar appears at the very bottom of the page (already filed as QA-076). At
1024 it is worse: the measured `clippedControls` list on `/tasks` at 1024x600 returns 8
buttons whose right edge is past the viewport (`Create subtask`, `Toggle task status`, six
`Toggle subtask status`).

**3. Clicking a card leaves the board.**

[TaskManager.svelte:141](../../../src/lib/domains/tasks/components/TaskManager.svelte#L141)
`goto(\`/tasks/${task.id}\`)` — a 965-line full-page route. Scroll position, the active
project filter and the search text are all lost on the way back.

## Fix

1. Delete the three count `Badge`s at `TaskManager.svelte:394-396`. The column headers carry
   them.
2. Narrow the project rail from 290px to `w-56` (224px) and drop the row height from 36px to
   28px — it holds 5 entries and a count. That plus (1) gets three full 300px columns at
   1280.
3. Open a task into a right rail (`ResponsivePanel side="right" desktopClass="w-[26rem]"`)
   instead of navigating: extract the body of `src/routes/tasks/[id]/+page.svelte` into
   `TaskDetail.svelte` taking `{ taskId }`, render it in the rail, and mirror the selection
   as `/tasks?task=<id>` so reload and back/forward keep it. Keep `/tasks/[id]` as a route
   for deep links — it renders the same `TaskDetail`.
   At 1280 that leaves 870 − 416 = 454px for the board, so pair it with (2) and have the
   rail default closed, opening on card click.

## Evidence

`.claude/qa/shots/ux/ux-tasks-2.png` (1280x800: Done column clipped),
`.claude/qa/shots/ux/ux-min1024-tasks.png` (1024x600: 8 controls past the right edge).

## Repro

`node .claude/qa/scripts/app.mjs tasks ux-tasks`

## Fix

<!-- coder -->

## Verification

2026-11-09 — tauri, real data (34 tasks, 5 projects). (1) The duplicate count-badge row above
the columns is gone — each column header (`Backlog 9`, `In Progress 1`, `Done`) now carries the
count exactly once. (2) The project rail narrowed from 290px to **224px** (confirmed via
`getBoundingClientRect`, matches the shared `--sidebar-width-domain: 14rem` token from UX-020,
not a page-local override) — but at 1280x800 the board's `.overflow-x-auto` still measures
`scrollWidth: 932` against `clientWidth: 752`, i.e. three `min-w-[300px]` columns still don't
fit and a horizontal scrollbar remains. That specific sub-problem is explicitly cross-referenced
in this issue's own Steps section to **QA-076**, which already tracks it — not reopening this
issue over it. (3) Clicked a task card: URL became `/tasks?task=117`, stayed on `/tasks`, and a
right rail opened with Task Details / Edit / Delete / Generate Tasks with AI, board still
visible underneath — matches the Fix. Screenshots: `.claude/qa/shots/QA-UX012-after-1280x800.png`,
`.claude/qa/shots/QA-UX012-after-task-click.png`. Noted in passing: the breadcrumb shows the raw
`?task=117` query string, same pattern QA-078 already tracks. **Verified** on items 1 and 3; item
2's residual horizontal scroll is QA-076's scope, not new.

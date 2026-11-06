---
id: QA-032
title: Tasks header: description squeezed into a 4-line column; at 1024px "New Task" is pushed off-screen
status: verified
severity: minor
kind: ux
area: tasks
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens `/tasks`, at the default size and at the minimum window (1024x600).

## Steps

1. Open `/tasks` at 1280x800: look at the title block.
2. Resize to 1024x600.

## Expected

Title and description on their own line(s) (the Documents page does this); toolbar buttons wrap or collapse so every action is visible.

## Actual

- 1280: the toolbar (Kanban/List, Multi-Select, Shortcuts, Generate Tasks with AI, New Task) takes most of the row, so "Organize and track your work efficiently" wraps to 4 lines in an 89 px column.
- 1024: the toolbar overflows; "Generate Tasks with AI" is clipped and "New Task" ends at x=1216 (viewport 1024), reachable only by horizontal scrolling the pane.

## Evidence

`.claude/qa/shots/QA-032-before-tasks-header-min-window.png`, `.claude/qa/shots/sweep-tasks-list-retry.png`.

## Repro

`node .claude/qa/scripts/QA-032.mjs`

## Fix

Root cause: PageHeader keeps actions in a non-wrapping shrink-0 row beside the title block. Fix: PageHeader now has title and description only; the toolbar is a `flex flex-wrap gap-2` row below it, so every action stays visible at 1024px.
Files: src/lib/domains/tasks/components/TaskManager.svelte. Checks as QA-025.

## Verification

2026-11-06, tauri mode (:9222). At 1280 the title and description are on their own lines; at 1024x600 the toolbar wraps and New Task is at x<=649, fully visible. After shots: shots/QA-032-after-tasks-header.png, shots/QA-032-after-tasks-header-min-window.png. Verified.

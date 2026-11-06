---
id: QA-025
title: Tasks search box and Filters modal show a chip but never filter the board or list
status: verified
severity: major
kind: bug
area: tasks
mode: tauri
found: 2026-11-06
---

## Use case

Owner has a growing task board and narrows it by typing in "Search tasks..." or by status/priority/type in Filters.

## Steps

1. Open `/tasks` (List view or Kanban).
2. Type `zzzznomatch` in "Search tasks...".
3. Open Filters, tick Status = Cancelled (no cancelled tasks exist), Apply.

## Expected

Board/list shows only matching tasks; an empty state when nothing matches.

## Actual

The "Active filters" chip and the Filters badge count appear, but every task is still shown (12 rows before and after). Stats, columns and list are unchanged.

## Evidence

`.claude/qa/shots/QA-025-before-filter-ignored.png`, `.claude/qa/shots/sweep-tasks-no-match-2.png`.
Likely cause: `TaskManager.svelte` builds `filteredTasksWithSearch` (and status/priority/type state) but only uses it for select-all; `KanbanBoard` and `TaskList` are rendered without it and read `taskUi` directly, which is never given the filters.

## Repro

`node .claude/qa/scripts/QA-025.mjs` prints row counts (expected 0 and 0, actual 12 and 12).

## Fix

Root cause: TaskManager kept search/status/priority/type in local state, but Kanban/List read `taskUi.filteredTasks`, which never received them. Fix: an effect in TaskManager pushes them into `taskUi.taskFilters` (cleared on unmount); `filteredTasksWithSearch` now is `taskUi.filteredTasks`; added a "No matching tasks" PageEmpty with Clear filters.
Files: src/lib/domains/tasks/components/TaskManager.svelte. Checks: pnpm check 0 errors, eslint 0 errors, prettier clean. Repro script not run (app is in use).

## Verification

2026-11-06, tauri mode (:9222). Reran: search 'zzzznomatch' -> 0 rows and 'No matching tasks' empty state; search 'QA cascade' -> only the fixture; Status=Cancelled filter -> 0 rows; Clear -> all 12 rows back. After shot: shots/QA-025-after-search-no-match.png. Verified.

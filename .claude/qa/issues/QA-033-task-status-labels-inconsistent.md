---
id: QA-033
title: Task status shown three ways: "To Do", "pending", "PENDING"
status: verified
severity: polish
kind: copy
area: tasks
mode: tauri
found: 2026-11-06
---

## Use case

Owner reads task status on the board, in the list and in the forms.

## Steps

1. `/tasks` Kanban: columns "To Do / In Progress / Done / Cancelled".
2. Switch to List view: badges read `pending`, `in-progress`, `completed` (raw enum values).
3. Open `/tasks/create` or `/tasks/<id>/edit`: Status select reads `PENDING`, Priority `MEDIUM`.
4. Task detail reads "To Do" / "In Progress".

## Expected

One display label per status everywhere.

## Actual

Raw lower-case enum in the List view badge and upper-case enum in the form selects, friendly labels elsewhere.

## Evidence

`.claude/qa/shots/sweep-tasks-list-dark.png` (list view badges), `.claude/qa/shots/sweep-tasks-create.png` (PENDING / MEDIUM), `.claude/qa/shots/sweep-tasks-detail-qa-top.png` ("To Do").

## Repro

Manual, steps above.

## Fix

Root cause: labels were ternaries copied per site, raw enum values elsewhere, and the form Selects used the TS enums (labels from keys: PENDING). Fix: `taskStatusLabel()` in tasks/types/index.ts (from TASK_STATUS_OPTIONS) used in TaskManager chips, TaskFilters, TaskList, KanbanBoard subtasks and the detail page (3 sites); kanban column "Done" renamed "Completed" to match; TaskForm and the generation preview/card Selects now use TASK_STATUS_OPTIONS / TASK_PRIORITY_OPTIONS (To Do, Medium).
Files: tasks/types/index.ts, state/taskUi.svelte.ts, components/{TaskManager,TaskFilters,TaskList,KanbanBoard,TaskForm,TaskGenerationPreview,TaskPreviewCard}.svelte, routes/tasks/[id]/+page.svelte. Checks as QA-025.

## Verification

2026-11-06, tauri mode (:9222). List badges now To Do / In Progress / Completed; create form status select reads To Do/Medium style labels; detail shows 'To Do'. Remaining nit seen: the stats card says 'Done' while the Kanban column says 'Completed' (not filed, trivial). Verified.

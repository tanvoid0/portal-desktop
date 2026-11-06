---
id: QA-034
title: Task tags are saved but shown nowhere except the edit form
status: verified
severity: minor
kind: ux
area: tasks/[id]
mode: tauri
found: 2026-11-06
---

## Use case

Owner tags a task and expects to see the tags when reading it.

## Steps

1. `/tasks/create`: title `QA ...`, add tag `qa-tag` (Enter), Create Task.
2. On the detail page look for the tag. Reload. Also check Kanban and List.
3. Open `/tasks/<id>/edit`.

## Expected

Tags visible on the detail page (and ideally on cards).

## Actual

Detail page, Kanban card and List row show no tags (`qa-tag` count 0). The edit form does show both tags after reload, so they are persisted. Same for assignee and estimated/actual time (set `QA person` and 90 min on the fixture: only the due date appears on the detail page).

## Evidence

`.claude/qa/shots/sweep-tasks-detail-qa-top.png` (no tags), `.claude/qa/shots/sweep-tasks-detail-with-extra-fields.png` (due date only), `.claude/qa/shots/sweep-tasks-edit-qa-fixture-loaded.png` (tags `qa-tag`, `qa-tag2` present in the form).

## Repro

Manual, steps above.

## Fix

Root cause: the views never rendered these fields. Added Assignee, Time (estimated / spent) and Tags to the detail page Task Details card; a one-line truncated "#tag #tag" under the title on kanban cards and list rows (time/assignee left off cards to avoid clutter).
Files: routes/tasks/[id]/+page.svelte, components/KanbanBoard.svelte, components/TaskList.svelte. Checks as QA-025.


Review follow-up: tag list on the detail page is keyed by index, so duplicate tags no longer throw.

## Verification

2026-11-06, tauri mode (:9222). Fixture with assignee, 90 min estimate and tag: detail page shows Assignee, 'Time 1h 30m estimated' and tag chips; Kanban card shows '#qa-tag'. After shot: shots/QA-034-after-detail-fields.png. Verified.

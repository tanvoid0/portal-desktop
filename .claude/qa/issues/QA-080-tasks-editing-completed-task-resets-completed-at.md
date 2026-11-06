---
id: QA-080
title: Editing a completed task restamps completed_at; leaving Done never clears it
status: verified
severity: minor
kind: bug
area: tasks backend (task_repository.rs update)
mode: tauri
found: 2026-11-06
---

## Steps
1. Drag fixture `QA Beta` into Done; read `get_task` via `window.__TAURI_INTERNALS__.invoke('get_task',{id})`: `completed_at` = 20:50:31.
2. Open `/tasks/68/edit`, change only the title, Update Task. Read again.
3. Drag the task back to Backlog and read again.

## Expected
Drop into Done stamps completed_at once (confirmed: it does persist, `scripts/QA-tasks-v2drag.mjs`). Editing other fields of an already-completed task keeps the stamp. Leaving Done clears it (or at least it is not treated as completion time).

## Actual
- Title-only edit changed completed_at 20:50:31 -> 20:50:58 (`scripts/QA-tasks-v2edit.mjs`; the edit form sends status=completed every time and `task_repository.rs:182-185` stamps on `is_completed`). Effect: any edit of a Done task pushes its auto-archive (7 days) back.
- Moving Done -> Backlog / In Progress leaves completed_at set on a pending task (`QA Beta` showed status pending with completed_at 20:46:31 at the start of a run). Harmless for the Archive filter (it checks status), but the field lies.
Fix: stamp only when the stored status is not already completed; set None when the new status is not completed.

## Verification
2026-11-06, mode: tauri, process 56316 started 21:03:29, after the 21:03 rebuild of portal_desktop.exe. `scripts/QA-080.mjs`, read via `get_task`: pending/null -> drag to Done: completed, 21:04:57.739 -> title-only edit via /tasks/80/edit: completed_at UNCHANGED -> drag to In Progress: in-progress, null -> drag to Done again: fresh stamp 21:05:06.939 -> Backlog: pending, null. Verified.

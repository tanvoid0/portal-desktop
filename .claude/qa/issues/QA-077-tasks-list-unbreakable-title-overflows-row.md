---
id: QA-077
title: Tasks List view: a title with no spaces runs past the row and card edge
status: verified
severity: minor
kind: bug
area: tasks/list (TaskList)
mode: tauri
found: 2026-11-06
---

## Steps
1. Create a task titled `QA Nospace` + 88 `A` (100 chars max).
2. `/tasks?project=none&view=list`, 1280x800.

## Expected
Title truncates or wraps inside the row like the kanban card (which clamps with an ellipsis).

## Actual
The title text node is 1019px wide with right edge x=1527 in a 1280 window; it is clipped by an overflow-hidden ancestor, pushing priority/status badges out of view. Same at 760px (`shots/sweep-tasks-hscroll-list-760.png`). Needs `min-w-0` + `break-words`/`line-clamp-2` on the title in the list row.

## Evidence
`shots/QA-077-before-list-nospace-title.png`

## Verification (REOPENED)
2026-11-06, mode: tauri, `scripts/QA-tasks-v077.mjs`, `v077b.mjs`. Still broken at 1024x600: the title h4 is `truncate min-w-0 flex-1` but is 1118px wide (left 296, right 1414); the row's scroller has scrollWidth 1348 vs clientWidth 694 and shows a horizontal scrollbar; the badges and status are off-screen (`shots/QA-077-after-nospace-1024x600.png`).
Cause (measured ancestor widths): `TaskList.svelte` ~line 70-110, the row is `flex items-center justify-between` (width 670) holding `<div class="flex items-center gap-3">` (width 1198) which holds `<div class="min-w-0">`. The middle `gap-3` flex child has default `min-width:auto`, so it grows to the text's max-content instead of shrinking; the title's own `min-w-0` never gets a bounded parent. Give that `flex items-center gap-3` wrapper `min-w-0` (and `flex-1`), and keep the badges `shrink-0`.

## Verification (round 2)
2026-11-06, mode: tauri, pid 56316 throughout, `scripts/QA-tasks-v077.mjs`. 1024x600, `QA Nospace`+88 A: h4 now 452px wide (388-840, truncated with ellipsis), scroller scrollWidth 694 = clientWidth 694 (no h-scrollbar), HIGH and To Do badges at x 921-978 inside the 1024 window. `shots/QA-077-after-nospace-1024x600-v2.png`. Verified.

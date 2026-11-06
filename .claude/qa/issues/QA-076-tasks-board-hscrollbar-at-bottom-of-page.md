---
id: QA-076
title: Tasks board horizontal scrollbar sits at the bottom of a 3600px page, so the right columns look cropped
status: verified
severity: major
kind: bug
area: tasks (KanbanBoard / TaskManager / ShellSidebarLayout)
mode: tauri
found: 2026-11-06
---

## Use case
Open /tasks board view in a window too narrow for three 300px columns and reach the Done column.

## Steps
1. `/tasks?project=all&view=board` (12 Backlog, 3 In Progress, 7 Done), viewport 1024x700 (also 1280x800).
2. Done column is cut at the right edge; no scrollbar visible.
3. Vertical mouse wheel over the board does not move it sideways. Only horizontal wheel/trackpad does (scrollLeft 0 -> 286), or scrolling to the very bottom of the page to find the scrollbar.

## Expected
Board scrolls horizontally with a visible scrollbar in the viewport; the page chrome (header, sidebar) stays put and columns scroll vertically inside the board.

## Actual (measured, `.claude/qa/scripts/QA-tasks-hscroll.mjs`, `QA-tasks-hscroll2.mjs`)
`overflow-x-auto` DOES engage and the last column IS reachable (scrollLeft max 410, lastRight == rootRight at 900px; no clipping by padding). The problem is height: nothing in the chain is height-bound, so the board is as tall as the longest column.

| element | scrollWidth | clientWidth | overflow-x | note |
|---|---|---|---|---|
| board root `flex gap-4 overflow-x-auto pb-2` | 932 | 806 (1280) / 646 (1024) / 522 (900) | auto | height 3672px, bottom at y=3968 (vh 700). Scrollbar is down there |
| TaskManager scroll div `min-h-0 min-w-0 flex-1 space-y-4 overflow-y-auto p-4` | 854/694/570 | same | auto | clientHeight == scrollHeight == 3816, so it never scrolls itself |
| ShellSidebarLayout main `flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden` | 854/694/570 | same | hidden | |
| contentClass wrapper `flex h-full min-h-0 w-full flex-1 overflow-hidden` | 1174/918/794 | same | hidden | |
| `div.w-full px-[var(--content-gutter)] py-4` (app page wrapper) | | | visible | natural height; `h-full` of TaskManager resolves to auto here |
| `div.h-full min-h-0 flex-1 overflow-y-auto` | | | auto | THIS is what actually scrolls the whole page (header + sidebar + board together) |

Cropping happens at the board root's own box (right edge 1222 at 1280, 842 at 900). Columns do not shrink (always 300px). Same inner column count at 760px: scrollLeft max 246.

Root cause: TaskManager/ShellSidebarLayout assume a bounded parent height (`h-full min-h-0`), but the page wrapper above is auto-height, so the heights never constrain. Fix: give the TaskManager root a bounded height (e.g. make the page wrapper for this route fill the viewport), then make each column scroll vertically inside the board so the board root is viewport height and its horizontal scrollbar is always visible.

## Evidence
`shots/sweep-tasks-hscroll-1024x700-top.png`, `shots/sweep-tasks-hscroll-1024x700-bottom.png`, `shots/sweep-tasks-hscroll-900.png`

## Verification
2026-11-06, mode: tauri (pid 8116, CDP). Scripts `scripts/QA-tasks-v076.mjs`, `v076b`, `v076c`, `v076d`.
Board root at 1280x800 / 1024x700 / 900x700 / 1024x600: document height == viewport (800/700/700/600, was ~3700), board clientHeight 538/394/370/294, scrollWidth 932 vs clientWidth 864/704/580/704, bottom edge 776/676/676/576 so the scrollbar is inside the viewport. Dragging the scrollbar thumb reaches scrollLeft 228 == max, third column fully visible, page scrollTop stays 0. Third column: header y and footer y unchanged (282/320) while its card area scrollTop 0 -> 600; columns 1 and 2 untouched (independent). List view scrolls to its end (6079/6079) and Archive scrolls; neither is clipped. `/tasks/create`, `/tasks/67`, `/tasks/67/edit`, `/tasks/generate` at 1024x600 all scroll to the bottom, Create Task / Update Task / Generate Tasks buttons reachable. No console errors. Shots `QA-076-after-*`.
Caveat: at 1024x600 the board has only ~230px of card area under the page header + toolbar (`shots/QA-076-after-colscroll-1024x600.png`), usable but tight. Not filed.

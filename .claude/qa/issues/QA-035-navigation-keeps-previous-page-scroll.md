---
id: QA-035
title: After saving a long form, the next page opens scrolled down with its header cut off
status: verified
severity: minor
kind: ux
area: tasks (app shell scroll containers)
mode: tauri
found: 2026-11-06
---

## Use case

Owner finishes creating or editing a task (long form) and lands on the task detail page.

## Steps

1. Open `/tasks/<id>/edit`, scroll to the bottom, press "Update Task" (same with `/tasks/create` then Create Task).

## Expected

New page starts at the top.

## Actual

The detail page opens with its inner scroller at scrollTop 104 (edit page had 313): the "Back to Tasks / Edit / Delete" row and the top of the Task Details card are hidden under the breadcrumb bar, and a success toast overlaps the rest. The outer scroller also keeps its 240.

## Evidence

`.claude/qa/shots/QA-035-before-detail-opens-scrolled.png`, `.claude/qa/shots/sweep-tasks-detail-qa-fixture.png` (same effect after create).

## Repro

`node .claude/qa/scripts/QA-035.mjs`

## Fix

Root cause: the app scrolls inside <main>'s nested containers, so SvelteKit's window scroll reset never applied. Fix once in the shell: `afterNavigate` (skipping popstate, so back/forward keeps position) zeroes scrollTop of every scrolled element inside <main>.
Files: src/lib/components/layout/app-shell.svelte. Checks as QA-025. Repro script not run (app is in use).


Review follow-up: the reset now skips elements inside `[data-sidebar]` (persistent domain sidebars keep their scroll) and returns early when `from` and `to` pathnames match (reset:false tab switches).

Null-guard: the afterNavigate hook now returns early unless both from.url and to.url exist (null on leave/unload and first load).

## Verification

2026-11-06, tauri mode (:9222). Edit/create with the pane scrolled to the bottom, then save: detail page opens with no scrolled element in main. Sidebar: at 1024x500 scrolled the nav sidebar to 540/700 and clicked the lowest item (Settings); scrollTop kept (540, clamped by the new page height, not reset). Back/forward position not tested. After shot: shots/QA-035-after-settings-sidebar-scroll.png. Verified.

---
id: QA-172
title: Escape does not close the open task rail
status: verified
severity: minor
kind: a11y
area: tasks
mode: tauri
found: 2026-11-22
---

## Use case

Open a task, glance at it, dismiss with the keyboard.

## Steps

1. /tasks, click a card (or focus it and press Enter).
2. Press Escape.

## Expected

The rail closes like a dialog.

## Actual

The rail and `?task=` stay; only the X button or Back closes it. Keyboard-only users must Tab to the X.

## Evidence

`.claude/qa/shots/QA-172-before-task-rail.png`.

## Repro

`node .claude/qa/scripts/QA-172.mjs`

## Fix

Root cause: no Escape handling for the task rail. Fix: in TaskManager handleKeydown Escape chain (after dialogs/multi-select, skipping typing targets) calls closeTask(). File: tasks/state/taskManager.svelte.ts. Checks: pnpm check, eslint.

## Verification

2026-11-22, tauri. node QA-172.mjs: Escape closes the rail (ok: closed).

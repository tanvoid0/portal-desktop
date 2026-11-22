---
id: QA-173
title: "Breadcrumb shows the raw query string `?task=638` when a task is open"
status: verified
severity: polish
kind: bug
area: tasks
mode: tauri
found: 2026-11-22
---

## Use case

Know where you are while a task is open.

## Steps

1. /tasks, open any task.
2. Read the breadcrumb.

## Expected

`Home > Tasks` or `Home > Tasks > <task title>`.

## Actual

`Home > Tasks > ?task=638`: the last segment is derived from the URL's search part.

## Evidence

`.claude/qa/shots/QA-173-before-breadcrumb-query-string.png`.

## Repro

manual - steps above

## Fix

Root cause: breadcrumbStore turned every unfiltered query param into a crumb. Fix: `task` added to the ignored params (also covers its sidebar-like page-local state). File: src/lib/domains/shared/stores/breadcrumbStore.ts. Checks: pnpm check.

## Verification

2026-11-22, tauri. Breadcrumb with a task open is 'Home > Tasks' (QA-173-after-breadcrumb.png).

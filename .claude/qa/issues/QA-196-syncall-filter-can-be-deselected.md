---
id: QA-196
title: Clicking the active filter in the Sync all sheet leaves no filter selected
status: fixed
severity: polish
kind: ux
area: projects (Sync all)
mode: tauri
found: 2026-11-26
---

## Use case

Switch between "Needs attention" and "All".

## Steps

1. Open a Sync all result. Click "Needs attention", which is already active.

## Expected

It stays selected.

## Actual

Neither segment is highlighted (`type="single"` ToggleGroup allows an empty value); the list still
behaves as "Needs attention".

## Evidence

`.claude/qa/shots/QA-196-before-syncall-filter-nothing-selected.png`

## Repro

`node .claude/qa/scripts/QA-194.mjs` (second screenshot)

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

Root cause: a single ToggleGroup allows an empty value.

Files: projects/components/SyncAllPanel.svelte (function binding ignores the empty value).

Repro script not run (told not to touch the running app); cause confirmed by reading the code. Checks: pnpm check 0 errors; eslint 0 errors; shadcn check pass; prettier fails only on the untouched docs/status/RELEASE_NOTES.md.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

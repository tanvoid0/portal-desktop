---
id: QA-194
title: A long project name pushes "Resolve with AI" out of the Sync all sheet
status: fixed
severity: minor
kind: ux
area: projects (Sync all)
mode: tauri
found: 2026-11-26
---

## Use case

Read the Sync all result and resolve one row.

## Steps

1. Sync all result containing a project with a long name without spaces. Window 1024x600 or 1280x800.

## Expected

The name truncates or wraps; the button stays visible.

## Actual

The list scrolls sideways (scrollWidth 845 vs 725 at 1024 wide) and that row's button is off screen.
`SyncAllPanel.svelte`: the name link has no `min-w-0` / `truncate` / `break-all`.

## Evidence

`.claude/qa/shots/QA-194-before-syncall-long-name.png`

## Repro

`node .claude/qa/scripts/QA-194.mjs` (stubbed report, no sync runs)

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

Root cause: the name link could not shrink in its flex row.

Files: projects/components/SyncAllPanel.svelte (name truncates with a title, button shrink-0, step detail wraps).

Repro script not run (told not to touch the running app); cause confirmed by reading the code. Checks: pnpm check 0 errors; eslint 0 errors; shadcn check pass; prettier fails only on the untouched docs/status/RELEASE_NOTES.md.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

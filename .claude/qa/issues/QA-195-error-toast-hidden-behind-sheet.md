---
id: QA-195
title: Error toast is hidden behind the Sync all sheet, so Resolve all looks dead
status: fixed
severity: minor
kind: bug
area: projects (Sync all), toast layer
mode: tauri
found: 2026-11-26
---

## Use case

Click "Resolve all with AI" for repos that share no parent folder (different drives).

## Steps

1. Sync all result with stuck repos on `D:` and `E:`.
2. Click "Resolve all with AI (2)".

## Expected

"These projects share no parent folder. Resolve them one by one." is visible.

## Actual

Nothing visibly happens. The toast is in the DOM (rect 925,29) but `elementFromPoint` there hits the sheet
header. Container (`toast-container.svelte`) and sheet are both `z-50`; the sheet is later in the DOM.
Only seen with this sheet; other sheets and dialogs were not checked. `toast-container.svelte` is under
`components/ui/`, so check CLAUDE.md's vendored-file rule before editing it.

## Evidence

`.claude/qa/shots/QA-195-before-syncall-no-parent-toast.png`

## Repro

`node .claude/qa/scripts/QA-195.mjs` (stubbed report, no sync runs)

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

Root cause: toast container and sheet were both z-50 and the sheet came later in the DOM.

Files: routes/+layout.svelte, components/layout/app-shell.svelte (class="z-60" on ToastContainer; vendored file untouched). Applies above every sheet and dialog. Not checked: whether clicking a toast over an open sheet also closes the sheet.

Repro script not run (told not to touch the running app); cause confirmed by reading the code. Checks: pnpm check 0 errors; eslint 0 errors; shadcn check pass; prettier fails only on the untouched docs/status/RELEASE_NOTES.md.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

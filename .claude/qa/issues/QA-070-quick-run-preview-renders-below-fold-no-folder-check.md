---
id: QA-070
title: Quick Run "Preview" appears to do nothing - the result renders at the bottom of the page, below the long Block Library; nonexistent folder is accepted
status: verified
severity: minor
kind: ux
area: automation/run
mode: tauri
found: 2026-11-06
---

## Use case

Check what Quick Run would execute before running it.

## Steps

1. `/automation/run`, click preset "Test".
2. Type `C:\no\such\qa-dir` in Directory (or any real folder).
3. Click Preview.

## Expected

Visible response: the preview scrolls into view or opens next to the queue; a folder that does not exist is flagged.

## Actual

Nothing visible changes at the top of the page, no toast. The "Preview - Resolved commands for <dir>" card is rendered at the very bottom of the page, under the Block Library list, off screen. The nonexistent folder is previewed as if valid. The "Select a folder" hint under the field also stays while a folder is typed.

## Evidence

`.claude/qa/shots/QA-070-before-quickrun-preview-nonexistent-folder.png` (after clicking Preview, nothing visible).

## Repro

`node .claude/qa/scripts/QA-070.mjs`

## Fix

`QuickRunPage.svelte`: Preview card moved to directly under the header; Preview and Run now call the existing backend `validate_project_path` and toast if the folder is missing/not a dir. The stale "Select a folder" hint lives in vendored `ui/file-picker.svelte`, left alone. Frontend only (HMR).

## Verification

<!-- qa-tester -->

## Verification (2026-11-06)

Preview card now renders directly under the page header, above the two-column grid,
with the resolved commands and the folder it resolved them for:
`shots/QA-070-after-preview-valid-folder.png`. A nonexistent folder is rejected before
preview or run with "Working directory: Project path does not exist"
(`shots/QA-070-after-preview-above-grid.png`).

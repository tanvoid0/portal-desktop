---
id: QA-176
title: "Logs: the trash icon wipes all log lines with no confirmation and no feedback"
status: verified
severity: minor
kind: ux
area: logs
mode: tauri
found: 2026-11-22
---

## Use case

Clear the log view.

## Steps

1. /logs (11 lines).
2. Click the trash icon (`Clear logs`).

## Expected

Confirm dialog (CLAUDE.md: destructive actions use confirmAction/AlertDialog) or an undo toast.

## Actual

Instantly `Showing 0 of 0` / `No logs found`: no dialog, no toast. The page also has no page title row, just a toolbar.

## Evidence

`.claude/qa/shots/QA-176-before-logs-cleared.png`.

## Repro

`node .claude/qa/scripts/QA-176.mjs`

## Fix

Root cause: clearLogs wiped immediately. Fix: confirmAction (destructive wording) then toast.success('Logs cleared'). File: src/lib/components/logs/UnifiedLogViewer.svelte. Checks: pnpm check, eslint. Page-title-row note not addressed.

## Verification

2026-11-22, tauri. Trash opens 'Clear all log lines? This cannot be undone.'; Cancel keeps 11 lines (QA-176-after-logs-confirm.png). Toast after confirm not exercised, to keep the log buffer.

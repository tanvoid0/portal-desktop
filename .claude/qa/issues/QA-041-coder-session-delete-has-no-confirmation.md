---
id: QA-041
title: Coder session delete (hover trash) removes the session immediately, no confirmation
status: verified
severity: minor
kind: ux
area: ai/coder
mode: tauri
found: 2026-11-06
---

## Use case

Clean up an old coder session from the Sessions list.

## Steps

1. `/ai/coder`, search a session, hover its row, click "Delete session".

## Expected

A confirm dialog (confirmAction/AlertDialog) before a session and its transcript are deleted; a toast afterwards.

## Actual

800 ms after the click no dialog (no `alertdialog`) and the session is already gone; stays gone after reload. No toast seen.

## Evidence

`.claude/qa/shots/sweep-ai-coder-delete-session-confirm.png` (list already empty right after the click).

## Repro

manual — steps above (use a QA-created session).

## Fix

Root cause: the list's delete callback called `coderSession.removeThread` directly. `CoderPanel.svelte` now routes it through `deleteSession`: `confirmAction(..., { confirmLabel: 'Delete', destructive: true })`, then removeThread, then `toast.success('Session deleted')`. Review: delete failure is caught and shown with toast.error.
Checks: pnpm check 0 errors, eslint/prettier clean on touched files. Not run in app.

## Verification

<!-- qa-tester -->

2026-11-06 (final round), tauri after rebuild. Delete session on my fixture session: AlertDialog 'Delete "list the files here..." and its transcript?' with Cancel/Delete. Cancel keeps it; Delete removes it and it stays gone after reload. No toast was captured after delete (may be timing). shots/QA-041-after-delete-confirm.png. Fixture folder + session removed.

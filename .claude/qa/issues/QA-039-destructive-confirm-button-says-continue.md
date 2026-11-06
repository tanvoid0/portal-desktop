---
id: QA-039
title: Delete confirmations (task, document) use a generic "Continue" button
status: verified
severity: polish
kind: copy
area: tasks, documents (confirmAction)
mode: tauri
found: 2026-11-06
---

## Use case

Owner deletes a task or a document.

## Steps

1. Task detail: Delete. Or Documents list: trash icon.

## Expected

Confirm button names the action ("Delete") and is styled destructive.

## Actual

Dialog "Delete task / Are you sure you want to delete ...? This action cannot be undone." with buttons "Cancel" and "Continue". The document dialog is the same, without the "cannot be undone" line.

## Evidence

`.claude/qa/shots/sweep-tasks-delete-confirm.png`, `.claude/qa/shots/sweep-documents-delete-confirm.png`.

## Repro

Manual, steps above.

## Fix

Root cause: confirmAction already takes `{confirmLabel, destructive}`; task call sites did not pass it (default "Continue"). Passed `{ confirmLabel: 'Delete', destructive: true }` at task, subtask and bulk delete. Helper unchanged. Document delete sites (src/routes/documents/+page.svelte, documents/[id]) are not mine and still say Continue: same one-line option there.
Files: routes/tasks/[id]/+page.svelte, components/TaskManager.svelte. Checks as QA-025.
Documents: delete confirms in src/routes/documents/+page.svelte and documents/[id]/+page.svelte now pass { confirmLabel: 'Delete', destructive: true }.

## Verification

2026-11-06, tauri mode (:9222). Task delete dialog and document delete dialog both show 'Cancel' / 'Delete'. After shots: shots/QA-039-after-task-delete-confirm.png, shots/QA-039-after-doc-delete-confirm.png. Verified.

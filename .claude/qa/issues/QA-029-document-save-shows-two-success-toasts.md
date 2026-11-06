---
id: QA-029
title: Saving a document shows "Document saved successfully" twice
status: verified
severity: polish
kind: ux
area: documents/[id]
mode: tauri
found: 2026-11-06
---

## Use case

Owner edits a document and presses Save.

## Steps

1. Open `/documents/<id>`, press "Save Document".

## Expected

One success toast.

## Actual

Two identical stacked toasts. `DocumentEditor.handleSave` toasts, and `routes/documents/[id]/+page.svelte` `onSave` toasts again. (Create mode has the same pair once QA-026 is fixed: editor and `documents/create/+page.svelte` both toast "created".)

## Evidence

`.claude/qa/shots/QA-029-before-double-toast.png`

## Repro

`node .claude/qa/scripts/QA-029.mjs`

## Fix

Root cause / change: Editor and both pages toasted. Removed the page-level toasts (editor owns save/create toast); the link-to-task toasts stay.
Files: src/routes/documents/[id]/+page.svelte, src/routes/documents/create/+page.svelte
Gates: pnpm check 0 errors; eslint 0 errors and prettier clean on touched files. Not rerun against the app (told not to touch :9222); needs tester verification.

## Verification

2026-11-06, tauri mode (:9222). Saving an existing doc shows one 'Document saved successfully' toast; create shows one 'created' toast. After shot: shots/QA-029-after-single-toast.png. Verified.

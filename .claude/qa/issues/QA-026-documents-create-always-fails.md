---
id: QA-026
title: Creating a document fails: "Cannot read properties of undefined (reading 'id')"
status: verified
severity: blocker
kind: bug
area: documents/create
mode: tauri
found: 2026-11-06
---

## Use case

Owner writes a new document (also reached from a task's "Create Document for Task" and the AI generate flow's save).

## Steps

1. Open `/documents/create`.
2. Enter a title (content and tags optional), click the primary button.

## Expected

Document is created, toast "Document created successfully", navigate to `/documents/<id>`; the card title reads "Create New Document" and the button "Create Document".

## Actual

Error toast "Failed to save document: Cannot read properties of undefined (reading 'id')". Nothing is stored (list stays empty). The same page shows the title "Edit Document" and the button "Save Document", i.e. it is in edit mode.

## Evidence

`.claude/qa/shots/QA-026-before-create-fails.png`. Calling the `create_document` command directly works, so the backend is fine.
Root cause: `DocumentEditor.svelte` has `const isCreateMode = $derived(document === null)`, but `/documents/create/+page.svelte` does not pass `document`, so it is `undefined`, not `null`. The save path takes the update branch and dereferences `document!.id`. Use `!document`.

## Repro

`node .claude/qa/scripts/QA-026.mjs`

## Fix

Root cause / change: `DocumentEditor` used `document === null`; create page passes `undefined`. Now `!document`.
Files: src/lib/domains/documents/components/DocumentEditor.svelte
Gates: pnpm check 0 errors; eslint 0 errors and prettier clean on touched files. Not rerun against the app (told not to touch :9222); needs tester verification.

## Verification

2026-11-06, tauri mode (:9222). Reran QA-026 flow with the UI: /documents/create titled 'Create New Document', saved with title/content/tag, one 'Document created successfully' toast, navigated to /documents/<id>, content persisted. Fixture deleted after. After shot: shots/QA-029-after-single-toast.png (same session). Verified.

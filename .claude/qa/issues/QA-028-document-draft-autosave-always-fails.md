---
id: QA-028
title: Document draft auto-save and the Archived flag never reach the backend (snake_case invoke args)
status: verified
severity: major
kind: bug
area: documents/[id]
mode: tauri
found: 2026-11-06
---

## Use case

Owner edits a document and expects the draft to be auto-saved two seconds after typing stops.

## Steps

1. Open an existing document (`/documents/<id>`).
2. Change the content, wait 3 seconds.

## Expected

Draft is saved (`update_document_draft` succeeds); at worst a visible indication on failure.

## Actual

Console error `Auto-save failed: invalid args contentDraft for command update_document_draft: command update_document_draft missing required key contentDraft`. The user sees nothing: no toast, no "saved" indicator, so edits are only kept if they press Save.
Root cause: `documentService.updateDraft` invokes with `{ id, content_draft }`, but Tauri maps the Rust `content_draft` parameter to the camelCase key `contentDraft`.

## Evidence

`.claude/qa/shots/QA-028-before-autosave-silent-fail.png`, console line above.

## Repro

`node .claude/qa/scripts/QA-028.mjs`

## Fix

Root cause / change: `updateDraft` and `saveDocument` sent snake_case top-level args; Tauri wants `contentDraft` / `isArchived`. Other invokes checked: nested `command` objects (serde snake_case) and `id`/`query` are fine. Auto-save failure now shown under the content box ("Draft auto-save failed: ...").
Files: src/lib/domains/documents/services/documentService.ts, src/lib/domains/documents/components/DocumentEditor.svelte
Gates: pnpm check 0 errors; eslint 0 errors and prettier clean on touched files. Not rerun against the app (told not to touch :9222); needs tester verification.
Follow-up (review): removed the mount-time auto-save $effect (opening a doc no longer flags it Draft); the timer starts only from oninput, is cleared in handleSave and on unmount. saveDocument sends `content ?? null` so clearing content is not turned into null. Editor save/create error toast uses String(error) for Tauri string rejections.
Gates now: pnpm check shows 0 errors in documents files (4 errors currently in another agent's src/routes/github files); eslint and prettier clean.

## Verification


## Also: Archived checkbox is not saved (same root cause)

Open `/documents/<id>`, tick "Archived (soft delete)", Save Document: success toast, but `get_documents` still returns `is_archived: false`; the document stays in the list and the checkbox is unchecked after reload. `documentService.saveDocument` sends `is_archived`; the Rust command param `is_archived: Option<bool>` expects `isArchived`, and an unknown key silently becomes `None`. `updateDocument` and `createDocument` send a nested `command` object (serde snake_case), those are fine. Grep all `invoke(` calls with snake_case top-level keys (`content_draft`, `is_archived`, ...) in `src/lib/domains/documents/services/documentService.ts` and fix them together.

Evidence: `.claude/qa/shots/QA-028-before-archived-not-saved.png`. Repro: `node .claude/qa/scripts/QA-028b.mjs` (leaves the fixture's archived box ticked in the UI only, nothing stored).
2026-11-06, tauri mode (:9222). Opening a doc and waiting 3.5s: is_draft=false, content_draft=null (not flagged). Typing: after 3.5s is_draft=true, content_draft saved, no console auto-save error. Archived box ticked + Save: get_document is_archived=true. After shot: shots/QA-028-after-archived-saved.png. Verified.

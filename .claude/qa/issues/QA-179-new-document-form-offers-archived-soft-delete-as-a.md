---
id: QA-179
title: "New document form offers \"Archived (soft delete)\" as a field"
status: verified
severity: minor
kind: ux
area: documents/create
mode: tauri
found: 2026-11-22
---

## Use case

Create a document.

## Steps

1. /documents/create.

## Expected

A new document is never born archived, so the checkbox would be on the edit form only.

## Actual

`Archived (soft delete)` sits between Title and Content. Product call: intended for importing archived docs, or a leftover from sharing the edit form?

## Evidence

`.claude/qa/shots/QA-179-before-archived-on-create.png`.

## Repro

manual - steps above

## Fix

Root cause: DocumentEditor, shared by create and edit, always rendered the Archived checkbox. Wrapped it in {#if !isCreateMode}; edit form unchanged. Files: src/lib/domains/documents/components/DocumentEditor.svelte. Checks: pnpm check (0 errors).

## Verification

2026-11-22, tauri. No Archived checkbox on /documents/create (QA-179-after-documents-create.png); the created document's detail page still has 'Archived (soft delete)' (QA-179-after-document-detail-archive.png). Fixture deleted.

## Owner decision (2026-11-22)

Remove the "Archived (soft delete)" checkbox from the New document create form. Archive stays available from the document actions.

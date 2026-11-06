---
id: QA-027
title: Document cards cannot be opened by keyboard; delete icon button has no name
status: verified
severity: major
kind: a11y
area: documents
mode: tauri
found: 2026-11-06
---

## Use case

Keyboard-only owner opens or deletes a document from `/documents`.

## Steps

1. Open `/documents` with at least one document.
2. Tab through the page.

## Expected

Each card is focusable (button/link semantics), Enter opens `/documents/<id>`; the trash icon has an accessible name ("Delete document ...").

## Actual

Tab order in the page: Generate with AI, New Document, search input, New Document, then the unnamed trash button. The card itself is never focused (no role, no tabindex), so a document cannot be opened without a mouse. The trash button reads as empty (no `aria-label`, no title).

## Evidence

`.claude/qa/shots/QA-027-before-card-keyboard.png`; script output. `DocumentCard.svelte` has no role/tabindex/aria attributes.
Compare with the task cards, which expose `role="button"` and `aria-label="Select task: ..."`.

## Repro

`node .claude/qa/scripts/QA-027.mjs`

## Fix

Root cause / change: Card had no role/tabindex/name. `DocumentCard` now passes role=button, tabindex=0, aria-label and an Enter/Space handler to CardActionCard; trash button has aria-label "Delete document: <title>".
Files: src/lib/domains/documents/components/DocumentCard.svelte
Gates: pnpm check 0 errors; eslint 0 errors and prettier clean on touched files. Not rerun against the app (told not to touch :9222); needs tester verification.
Follow-up (review): role=button on the card nested the Delete button. Now stretched-link: the title is a Button (after:absolute inset-0) covering the card, Delete is its own button at z-10; no role/tabindex on the card.

## Verification

2026-11-06, tauri mode (:9222). Tab order now: Generate, New Document, search, card title button, 'Delete document: <title>' button. Enter on the title button opens /documents/<id>. After shot: shots/QA-027-after-card-keyboard.png. Verified.

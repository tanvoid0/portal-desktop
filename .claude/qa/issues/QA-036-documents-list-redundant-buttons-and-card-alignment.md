---
id: QA-036
title: Documents list: "New Document" shown three times; card tags/date row not aligned with the title
status: verified
severity: polish
kind: ux
area: documents
mode: tauri
found: 2026-11-06
---

## Use case

Owner scans `/documents`.

## Steps

1. Open `/documents` (empty, then with one document).

## Expected

One primary "New Document" action (header) plus at most the empty-state CTA; card content on one left edge.

## Actual

- The header has "New Document", the search row has a second full-width "New Document" button, and the empty state adds a third (all visible together when empty).
- Card: the title and preview start at x=296 but the tag chips and the date row start at x=312 (16 px inset), and the trash button floats mid-row.
- The preview shows raw markdown (`# Heading Some **bold** <img ...> text - a - b`).

## Evidence

`.claude/qa/shots/sweep-documents-list.png` (three buttons), `.claude/qa/shots/sweep-documents-list-with-qa-doc.png` (card), `.claude/qa/shots/sweep-documents-dark-min-window.png`.

## Repro

Manual, steps above.

## Fix

Root cause / change: Removed the search-row "New Document" button (header button + empty-state CTA remain). Card: removed the nested CardContent (CardActionCard already wraps children in one) so tags/date align with the title. Raw-markdown preview left as is (not requested).
Files: src/lib/domains/documents/components/DocumentList.svelte, src/lib/domains/documents/components/DocumentCard.svelte
Gates: pnpm check 0 errors; eslint 0 errors and prettier clean on touched files. Not rerun against the app (told not to touch :9222); needs tester verification.

## Verification

2026-11-06, tauri mode (:9222). Only the header New Document button remains (plus empty-state CTA); card title, preview, tags and date share one left edge. After shot: shots/QA-036-after-documents-list.png. Verified.

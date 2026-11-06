---
id: QA-081
title: Tasks multi-select: Ctrl+A in the search box selects tasks instead of text; Esc on the delete confirm also exits multi-select
status: verified
severity: minor
kind: bug
area: tasks (TaskManager handleKeydown)
mode: tauri
found: 2026-11-06
---

## Steps / Actual
1. Ctrl+M (multi-select on), click into the search box, type `QA Del`, press Ctrl+A. The text is NOT selected; the shortcut calls select-all and the bar shows `3 selected` (`scripts/QA-tasks-v2keys.mjs`). Ctrl+K/F/N/H are fine to fire while typing, but Ctrl+A must be left to the field (same guard the bare Delete key already has, TaskManager.svelte ~249-253).
2. Select 3 tasks, press Delete (confirm opens), press Esc. The confirm closes AND multi-select mode ends with the selection lost (`scripts/QA-tasks-v2esc.mjs`, `shots/QA-NEW-esc-confirm-leaves-multiselect.png`: no bar). The window keydown handler also receives the Escape that closed the dialog. Expected: Esc dismisses only the dialog and keeps the selection.

## Evidence
`shots/QA-NEW-esc-confirm-leaves-multiselect.png`

## Verification (REOPENED, half fixed)
2026-11-06, mode: tauri, `scripts/QA-081.mjs`, `QA-081b.mjs`.
Fixed: Ctrl+A in the search box selects the text (0 selected, selection "QA Cmp"); Ctrl+A outside a field selects all (21 selected); Esc closes the filter dialog and the shortcuts dialog without side effects; Esc with no dialog exits multi-select; Delete in a textarea/search field edits text.
Still broken: Delete -> confirm opens -> Escape closes the confirm AND multi-select ends, selection lost (bar gone). Clicking the confirm's Cancel button instead keeps `21 selected`, so only the keyboard path leaks. Shot `QA-081-after-esc-keeps-selection.png` (no bar).
Likely cause: `hasOpenDialog()` (TaskManager.svelte ~230) runs in the window keydown bubble handler, but the AlertDialog's own Escape handling has already closed it by then, so the query finds no `[role=alertdialog]` and execution falls through to `handleMultiSelectToggle()`. Check `event.defaultPrevented`, or compare against a flag set while confirm is pending, or handle Escape in capture phase.

## Verification (2)
2026-11-06, mode: tauri, `scripts/QA-081.mjs` + `scripts/QA-081c.mjs`. All pass, no console errors.
Esc on bulk-delete confirm: closes it, `21 selected` kept. Confirm's Cancel: kept. Esc on filter dialog and shortcuts dialog: closed, selection kept. Esc with no dialog: exits multi-select. Dialog Esc then Esc again: second press exits multi-select (flag does not stick; filter and delete-confirm paths). Ctrl+A in search selects text (0 selected); Ctrl+A outside a field selects all (21); Delete in a text field edits text. Confirm was never accepted. Shot `QA-081-after-esc-keeps-selection-2.png`.

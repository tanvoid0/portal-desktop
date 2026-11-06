---
id: QA-079
title: Advanced Filters dialog cuts off its Clear All / Cancel / Apply buttons, unreachable at 1024x600
status: verified
severity: major
kind: bug
area: tasks (TaskFilterModal)
mode: tauri
found: 2026-11-06
---

## Steps
1. `/tasks?project=all&view=board`, press Ctrl+F (or Filters).
2. At 1280x800 look at the bottom of the dialog; repeat at 1024x600 (minimum window).

## Expected
Dialog fits the window; its body scrolls and the footer buttons stay visible.

## Actual
`Dialog.Content` has `max-h-[80vh] overflow-hidden` (TaskFilterModal.svelte:131) but the inner `div.space-y-6.overflow-y-auto` (line 136) has no height bound / `min-h-0`, so nothing scrolls: dialog scrollHeight 698 vs clientHeight 638 (800px) / 478 (600px), overflow hidden. At 1280x800 the buttons are cut in half (Apply spans y 695-731, dialog ends at 720). At 1024x600 Apply is at y 675-711, dialog bottom at 540: Types list is cut mid-row and Apply is unreachable by mouse (`overflow:hidden` has no scrollbar or wheel scroll). Repro `scripts/QA-tasks-v2filt.mjs`.

## Evidence
`shots/sweep-tasks-v2-filter-1024x600.png`, `shots/sweep-tasks-v2-filter-1280x800.png`, `shots/QA-079-before-filter-dialog-1024x600.png`

## Verification
2026-11-06, mode: tauri, `scripts/QA-079.mjs`. 1280x800: dialog y 80-720, buttons y 643-679 (inside). 1024x600: dialog y 60-540, buttons y 463-499 (inside), sections scroll independently (516/288, scrolled to 228) above a fixed footer. Mouse-clicked Type > Bug then Apply Filters: dialog closed, board showed only QA Alpha, Bug chip + Filters badge 1. Shots `QA-079-after-filter-dialog-{1280x800,1024x600}.png`, `QA-079-after-filter-dialog-scrolled-1024x600.png`, `QA-079-after-applied-bug-filter.png`. Verified.

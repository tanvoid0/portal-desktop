---
id: QA-090
title: Log viewer has no way back to the live tail once you scroll up
status: open
severity: minor
kind: ux
area: logs
mode: tauri
found: 2026-11-07
---

## Use case

Tail the log stream, scroll up a few screens to read something that just went past, then get
back to the newest line and carry on watching.

## Steps

1. `/logs` with enough lines to overflow (34 in my run — client-side navigation around the app
   while `/logs` stays mounted, see the repro script).
2. Scroll the log list to the top.
3. Try to get back to the bottom without dragging the scrollbar: uncheck and re-check
   **Auto-scroll**.

## Expected

Something returns the view to the newest line on demand. Either Auto-scroll jumps to the bottom
the moment it is re-checked, or the control that used to do it is still there.

## Actual

- Re-checking Auto-scroll leaves `scrollTop` at 0 — nothing moves. `updateLogs()` is the only
  thing that scrolls, and it only runs when a new entry arrives
  ([UnifiedLogViewer.svelte:120-129](../../../src/lib/components/logs/UnifiedLogViewer.svelte#L120)),
  so during a quiet stream the checkbox does nothing visible at all.
- The footer that carried **Scroll to bottom** was deleted with the rest of the footer in UX-006,
  and nothing replaced it: `getByRole('button', { name: /scroll to bottom/i })` finds 0 matches.
- Dragging the scrollbar is the only remaining way down. The `ScrollArea` viewport is not a tab
  stop, so a keyboard user cannot press `End` on it either.

Related and arguably the bigger half of the same hole: `updateLogs()` scrolls to the bottom
unconditionally whenever `autoScroll` is true, with no "the user scrolled up, pause following"
check — so on a busy stream reading history requires unchecking Auto-scroll first, which is
exactly when you then need a way back down.

## Evidence

`node .claude/qa/scripts/ux006-verify2.mjs`:

```
after scrolling to top:        {"scrollTop":0,"scrollMax":105, ...}
after re-checking Auto-scroll: {"scrollTop":0,"scrollMax":105, ...}
"Scroll to bottom" control present? 0
```

`.claude/qa/shots/UX-006-after-logs-autoscroll-recheck.png`

## Repro

`node .claude/qa/scripts/ux006-verify2.mjs`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

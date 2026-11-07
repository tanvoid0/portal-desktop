---
id: UX-006
title: Log viewer shows 9 lines where a developer tool should show 25 — nested cards and 44px rows
status: verified
severity: major
kind: ux
area: logs
mode: tauri
found: 2026-11-06
---

## Use case

Watch the app's log stream while reproducing something.

## Measured (1280x800)

Nine log lines fit on screen. Everything else is padding:

| layer | px |
| --- | --- |
| page `p-6` + `h1 text-2xl` + description ([src/routes/logs/+page.svelte:5-10](../../../src/routes/logs/+page.svelte#L5)) | **96** |
| `CardHeader` "Unified Logs" + Clear/Export ([UnifiedLogViewer.svelte:163-180](../../../src/lib/components/logs/UnifiedLogViewer.svelte#L163)) | 68 |
| filter row | 56 |
| per-row `p-2 text-sm` + `space-y-1` ([UnifiedLogViewer.svelte:223-229](../../../src/lib/components/logs/UnifiedLogViewer.svelte#L223)) | **44 per line** |
| footer "Showing 9 of 9 logs" | 44 |

The viewer is also boxed three deep: the page says **Application Logs**, the Card header says
**Unified Logs**, and the rows sit in a third bordered box inside `CardContent`. Three frames
around one list.

The container is `h-[calc(100vh-12rem)]`, a hardcoded guess that does not know about the app
header or the page padding, so the box stops ~60px short of the bottom and leaves dead space
under the footer.

## Expected

A log viewer is the one screen in the app that should be edge-to-edge and monospace. At 22px
per row the same 1280x800 window shows ~25 lines instead of 9.

## Actual

9 lines, 44px each, inside three nested frames.

## Fix

1. [src/routes/logs/+page.svelte](../../../src/routes/logs/+page.svelte) — delete the `p-6`
   wrapper, the h1 and the description; register `/logs` as a fill-viewport route in
   [src/lib/config/layout-breakpoints.ts](../../../src/lib/config/layout-breakpoints.ts) the
   way `/terminal` is, and render `<UnifiedLogViewer />` at `h-full`. Drop
   `h-[calc(100vh-12rem)]`.
2. [UnifiedLogViewer.svelte:163-180](../../../src/lib/components/logs/UnifiedLogViewer.svelte#L163) —
   drop the `Card`/`CardHeader`/`CardTitle` wrapper. Put Clear and Export as `size="sm"
   variant="ghost"` icon buttons at the right end of the existing filter row; that row becomes
   the only chrome.
3. [UnifiedLogViewer.svelte:223-243](../../../src/lib/components/logs/UnifiedLogViewer.svelte#L223) —
   row to `px-3 py-0.5 font-mono text-xs leading-5`, drop `space-y-1`, drop the hover
   `rounded`. Level becomes a one-letter coloured glyph rather than an `INFO` badge; keep the
   timestamp monospace and the context chip.
4. Move "Showing N of M" into the filter row as plain text; delete the footer bar.

The three `all` selects already carry correct `aria-label`s ("Filter by level / source /
context") — keep those, but show the label in the trigger when a value is set so the row
does not read `all all all`.

## Evidence

`.claude/qa/shots/ux/ux-logs.png`, `.claude/qa/shots/ux/ux-min1024-logs.png`.

## Repro

`node .claude/qa/scripts/app.mjs logs ux-logs`

## Fix

<!-- coder -->

## Verification

2026-11-07 — tauri, dev instance 2 (vite :1421 / CDP :9223, own database).
Ran `node .claude/qa/scripts/ux006-verify.mjs` and `node .claude/qa/scripts/ux006-verify2.mjs`
(34 log lines in the buffer, reached by client-side navigation so the logger singleton survives).

Measured:

| | before | now |
| --- | --- | --- |
| full log lines visible, 1280x800 | 9 | **29** (33 incl. partials) |
| row height | 44px | **24px** |
| chrome above the first row, 1280x800 | ~220px | **89px** (one filter row) |
| chrome above the first row, 1024x600 | — | 129px (filter row wraps to two lines, cleanly) |
| full lines visible, 1024x600 | — | 19 |
| gap between scroller bottom and viewport bottom | ~60px | **0px** |

- `/logs` is in `VIEWPORT_FILL_ROUTES`; the route file is 5 lines, no page `h1`, no Card frame.
- Clear and Export are ghost icon buttons with `aria-label`/`title` "Clear logs" / "Export logs";
  Export downloaded `logs-2026-11-07.txt`; Clear emptied the list ("Showing 0 of 0", "No logs found").
- The three selects read "All levels" / "All sources" / "All contexts" when unset, and carry
  `aria-label` "Filter by level / source / context". Level, source and context filtering and the
  search box all narrow the list and the trigger then shows the chosen value: level WARN -> 0 of 9,
  source backend -> 0 of 9, context AppLayout -> 2 of 9, search "Theme" -> 1 of 9.
- The level glyph is a one-letter coloured glyph with `aria-hidden` on the letter plus an
  `sr-only` full level name and a `title` — it has a screen-reader name.
- Auto-scroll pins the view to the newest line on mount (`scrollTop == scrollMax`).
- `/terminal` still fills the viewport and renders normally (`sweep-terminal-i2-1280.png`).

Shots: `.claude/qa/shots/UX-006-after-logs-1280.png`,
`.claude/qa/shots/UX-006-after-logs-full-1280.png`,
`.claude/qa/shots/UX-006-after-logs-1024x600.png`.

One gap found and filed separately, not a regression of this issue: with the footer gone there is
no way to get back to the live tail after scrolling up during a quiet stream — re-checking
Auto-scroll does not jump to the bottom. See **QA-090**.


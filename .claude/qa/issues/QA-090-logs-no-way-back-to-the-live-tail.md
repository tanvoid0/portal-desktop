---
id: QA-090
title: Log viewer has no way back to the live tail once you scroll up
status: verified
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

Round 1: `updateLogs()` was the only thing that ever scrolled, and re-checking Auto-scroll
didn't trigger it; `autoScroll` also never paused itself when the user scrolled away. Added
`handleScroll()`/`isAtBottom` tracking, a `$effect` that scrolls to bottom when `autoScroll`
turns on, and a floating "Scroll to bottom" button.

Round 2 (reopened — the round-1 `<ScrollArea class="absolute inset-0">` wrapper never actually
positions absolutely, because `ScrollAreaPrimitive.Root` sets an inline
`style="position: relative"` and inline style always beats a class, so the pane kept growing
with content instead of scrolling, clipped by the page shell): dropped the attempt to make
`ScrollArea` itself absolute. It now stays sized the original way (`class="h-full"`, filling
the wrapper `div`'s `relative min-h-0 flex-1` box via normal flex layout); the floating button
is still `absolute` but positioned against that wrapper `div` (which *is* a real `position:
relative` ancestor, set by a plain class with no inline-style fight), not against `ScrollArea`.

File: `src/lib/components/logs/UnifiedLogViewer.svelte`.
Checks: `pnpm check && pnpm lint`.

## Verification

2026-11-09, mode: tauri (real app on :9222, restarted mid-session after it crashed — see
report). Reran `node .claude/qa/scripts/QA-090-regression.mjs` (generates ~90 log entries by
toggling the theme repeatedly, since client-nav history was too deep in this long-lived dev
session to rely on `history.go`).

**Reopened — the fix regresses the log pane's scrolling entirely, worse than the original bug.**

The new `<ScrollArea class="absolute inset-0">` wrapper never actually becomes absolutely
positioned: bits-ui's `ScrollAreaPrimitive.Root` sets an inline
`style="position: relative; --bits-scroll-area-corner-height: 0px; ..."` on its root element,
and an inline style always wins over a class-based `position` utility — `tailwind-merge`
correctly resolves the class string to `"absolute inset-0"` (confirmed, no stray `relative`
class survives), but the *inline* style is what the browser actually applies, so computed
`position` stays `relative` regardless.

Effect: the pane never gets `inset: 0`'s stretch-to-parent sizing, so the ScrollArea's
`Viewport` (`class="size-full"`) shrink-wraps to its own content height instead of the
available flex space. With enough entries it just grows past the window, and the page shell's
`<main class="... overflow-hidden ...">` silently clips everything below the fold — no
scrollbar, no overflow, nothing. Confirmed live:

```
inlineStyle: "position: relative; --bits-scroll-area-corner-height: 0px; ..."
computedPosition: "relative"
vpScrollHeight: 2184   vpClientHeight: 2184   (never overflows — grows 1:1 with content instead)
mainOverflow: "hidden"
totalRows: 91   showing: "Showing 91 of 91"
"Scroll to bottom" button present: 0   (isAtBottom never goes false — nothing ever scrolls to leave)
```

`.claude/qa/shots/QA-090-regression-entries-clipped-not-scrollable.png` — 91 rows "showing",
visibly running off the bottom of the window with no scrollbar and no floating button; rows
below the last visible one are simply gone, unreachable by any means (wheel scroll does
nothing, `Auto-scroll` is still checked and claims "Showing 91 of 91").

So on a busy stream (the exact scenario QA-090 was filed about) the fix is strictly worse than
before: previously you could still drag the native scrollbar to read history; now there is no
scrollbar and no way to see anything past the clipped edge at all, while the UI claims nothing
is wrong ("Showing 91 of 91", Auto-scroll checked).

Root-cause fix needs to size the pane a different way than `absolute inset-0` on `<ScrollArea>`
itself — e.g. keep the ScrollArea sized via flex (`min-h-0 flex-1`) as it was before, and give
the floating button its own positioning context by wrapping *that* in `absolute`, not the
scroll area.

Separately, independent of this: the Logs page itself also crashed to a SvelteKit 500 error
screen on several fresh loads this session (`each_key_duplicate` on the log list's keyed
`{#each}`, pre-existing key logic unrelated to this diff) — filed separately as QA-105, since
it blocked testing here and is clearly reproducible on its own.

### Round 2 — 2026-11-09, mode: tauri

`node .claude/qa/scripts/QA-090-regression.mjs`: `vpScrollHeight` (2184) now exceeds
`vpClientHeight` (1304) — the pane is properly height-constrained and genuinely scrollable
again, not growing unboundedly.

`node .claude/qa/scripts/QA-090.mjs` (full interactive run, 161 entries):

```
before scroll-up:              scrollTop:2560 scrollMax:2560 (i.e. at the bottom)
after scrolling to top:        scrollTop:0    scrollMax:2560
auto-scroll after scroll-up:   unchecked
"Scroll to bottom" present:    1
after clicking it:             scrollTop:2560 scrollMax:2560 (back at bottom), button gone
scrolled up again, re-checked Auto-scroll: data-state=checked, scrollTop jumps to 2560 immediately
```

Every behavior in the original issue now works: scrolling away pauses auto-scroll, the
floating button appears and returns to the bottom, re-checking the box jumps back
immediately. Verified.

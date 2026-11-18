---
id: QA-122
title: Pending-request capability grid overlaps itself inside the narrow pairing card
status: fixed
severity: major
kind: bug
area: devices/pairing
mode: tauri
found: 2026-11-11
---

## Use case

Approving a pending pairing request on `/devices`: reviewing which
capabilities (`agents`, `agents.exec`, `coder`, `dashboard`, `projects`,
`tasks`, `terminal`, `terminal.exec`) are about to be granted to an unknown
device before clicking Allow.

## Steps

1. Open `/devices` with at least one pending pairing request (two were
   already present from real pairing activity on this machine).
2. Scroll to "Pending requests".
3. Look at the capability toggle grid inside either request card.

## Expected

Each capability's switch and label are legible on their own line/cell, same
as the "Pair a phone" capability grid higher on the same page (which is wide
enough to hold 3 columns cleanly).

## Actual

In the second and third grid columns, the next column's `Switch` renders on
top of the tail of the previous label: `agents.exec` reads as `agents.e`
with a switch thumb sitting on the rest of the word, and `dashboard` reads
as `dashboa` with the next switch overlapping it. The underlying label text
is intact (confirmed via DOM text content — this is a pure visual/CSS
overlap, not truncation), which makes it worse: the control is there, just
unreadable and the hit targets visually collide. This happens in a
security-relevant approval step where the whole point is to see which
capabilities are being granted before clicking Allow.

Root cause: `CapabilityChecklist.svelte:34` hardcodes
`grid grid-cols-2 gap-2 sm:grid-cols-3`. `sm:` is a **viewport**-width
breakpoint (≥640px), not a container query, so once the app window is wider
than 640px the grid always goes 3-column — even when the component is
reused inside the much narrower "Pending requests" card
(`PendingRequestRow.svelte:66`) or the paired-device row
(`DeviceRow.svelte:107`), which have nowhere near 640px of width. The
"Pair a phone" instance (`PairDeviceCard.svelte:98`) only looks fine because
it sits in the full-width column, not a half-width card.

## Evidence

`.claude/qa/shots/QA-122-before-pending-capability-overlap.png` — both
pending-request cards show the same overlap in columns 2 and 3 of the grid.

## Repro

`node .claude/qa/scripts/QA-122.mjs` — opens `/devices`, screenshots the
pending section, and dumps each capability label's DOM text (shows full
words present despite the visual overlap). Needs an existing pending
pairing request to render the card at all; this machine already has two.

## Fix

Root cause: `CapabilityChecklist.svelte` laid its switches out with `grid-cols-2 sm:grid-cols-3`.
`sm:` is a viewport breakpoint and Tailwind's `grid-cols-N` tracks are `minmax(0, 1fr)`, so any
window over 640px forced three ~90px columns even where the component sits in a card a third of
that — `PendingRequestRow` (~290px inside `/devices`' `sm:grid-cols-2 xl:grid-cols-3` card grid) and
`DeviceRow`'s edit view — and the longer names (plus the Sensitive badge added since the
screenshot) spilled into the next column.

Fix: the grid now sizes to its own width, not the viewport's —
`grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))]` (the pattern `/sdk` uses) — and each
cell is `flex-wrap`. 13rem holds the widest cell (`agents.history` + Sensitive = 199px measured), so
a capability is one line at any width; a narrower box just gets fewer columns, down to one, and
`flex-wrap` drops a badge to a second line instead of past the cell edge. One shared component, so
the pending card, the paired-device edit view and "Pair a phone" are all fixed (that one now fits 4
columns where it showed 3). No other QA ids.

Files: `src/lib/domains/devices/components/CapabilityChecklist.svelte`.
Checks: `pnpm check` and `pnpm lint` (0 errors). Not driven live (no dev instance was run). Layout
measured in headless Chromium against a mock of the grid (Inter, the real Label/Switch/Badge
paddings, 17 capabilities) at 138/290/330/456/918/1400px: before, 7 sensitive cells overflow at
every width up to 456px; after, none overflow at any width and none wrap from 290px up.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

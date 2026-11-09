---
id: UX-009
title: Project audit shows zero triage rows at 1024x600 — the scan form sits permanently above the results
status: verified
severity: major
kind: ux
area: projects/audit
mode: tauri
found: 2026-11-06
---

## Use case

Audit the disk, then walk the triage list deciding keep / archive / delete / review for each
project. This is a repetitive keyboard workflow — the page even advertises
`j/k move · 1-4 keep/archive/delete/review · o open folder · x select · esc clear`.

## Measured (real data: 35 projects, 444 roots)

At **1280x800** the first triage row is at **y=676** of a 760px viewport — you see one row.

At **1024x600** (the minimum window) **no triage row is on screen at all.** The whole
viewport is header + the scan form + half the stat grid.

Breakdown:

| block | px | file |
| --- | --- | --- |
| PageHeader | 44 | |
| `Audit / AI tidy` tabs | 48 | |
| **"Folders to scan" Card** — root chips, path input, Add folder, Scan and import, hint line, two checkboxes | **250** | [ProjectAudit.svelte:455-530](../../../src/lib/domains/projects/components/ProjectAudit.svelte#L455) |
| 6 bespoke stat cards | 150 | see UX-002 |
| `Triage / At risk / Duplicates / Tree / Changes (139)` + `By size` + `Export` + search | 48 | [ProjectAudit.svelte:766-790](../../../src/lib/domains/projects/components/ProjectAudit.svelte#L766) |
| shortcut legend line | 24 | |

The 250px scan form is configuration you touch when you set the roots up and then once a
month. It occupies the top of the screen on every visit.

## Expected

Triage rows start near the top. The scan configuration is reachable but not in the way.

## Actual

One row visible at 1280x800; none at 1024x600.

## Fix — this is the strongest right-sidebar candidate in the app

Give `/projects/audit` a right rail with `ResponsivePanel side="right"`.
**No new primitive is needed:**
[src/lib/components/shell/responsive-panel.svelte:8,30](../../../src/lib/components/shell/responsive-panel.svelte#L8)
already takes `side: 'left' | 'right'` and collapses to a `Sheet` below `md`; it is already
used that way by
[CoderPanel.svelte:829](../../../src/lib/domains/coder/components/CoderPanel.svelte#L829) and
[ai/chat/+page.svelte:612](../../../src/routes/ai/chat/+page.svelte#L612).
`shell-sidebar-layout.svelte` does not need a right variant.

**Right rail (`desktopClass="w-80"`), top to bottom:**

1. **Scan** — the whole "Folders to scan" card body (root chips, path input, Add folder,
   Scan and import, the two checkboxes). Collapsed to a `Scan` button + root count when no
   scan is running.
2. **Selected project** — for the row under the `j/k` cursor: full path, size, last commit,
   unpushed count, remote, duplicate siblings. Today you have to leave the page to see any
   of this.
3. **Marked** — the count of `x`-selected rows and the four bulk decision buttons, which are
   currently only reachable after you scroll.

**Main column:** PageHeader, a one-line PageStats strip (UX-002), the tab + filter row, then
rows from ~y=180. That is **11 triage rows on screen at 1280x800 and 7 at 1024x600**, up
from 1 and 0.

Keep the keyboard map as it is — the rail is read-only except for the scan controls, so
`j/k/1-4/o/x/esc` keep working unchanged.

## Evidence

`.claude/qa/shots/ux/ux-projects-audit.png` (1280x800, one row visible),
`.claude/qa/shots/ux/ux-min1024-projects-audit.png` (1024x600, zero rows visible).

## Repro

`node .claude/qa/scripts/app.mjs projects/audit ux-projects-audit`

## Fix

Root cause: configuration (the 250px "Folders to scan" card) and the bulk actions were
stacked above the results in a single column, so every pixel of setup pushed the triage
list down.

- Moved the scan card, the cursor row's facts and the bulk decision buttons into a
  `ResponsivePanel side="right" desktopClass="... w-80 ..."` rail (sticky, `max-h-[calc(100vh-7rem)]`,
  folds into a Sheet below `md` with an `md:hidden` trigger button in the main column).
  The scan form itself is folded behind the Scan button and a `N folders to scan`
  disclosure, auto-opened on mount only when there are no roots.
- Replaced the six bespoke stat cards (~150px) with a one-line `PageStats` strip —
  that is UX-002 for this file, so it is covered here too. The per-card sub-lines
  became `PageStat.description`, which PageStats renders as the item's `title`.
- Removed the in-flow `{#if marked.size}` bulk-action card; it is the rail's Marked
  section now, always visible on desktop instead of needing a scroll.
- Rail "Selected project" shows the `j`/`k` cursor row (`filtered[cursor]` on
  triage/risk, `selected` on the other tabs): name + status badge, full abs path,
  reclaimable bytes, last commit, unpushed count, remote, duplicate siblings, risk
  flags. All from data already on `DiscoveredProject` — no new fetch. Note there is no
  total-project-size field in the scan output, so "size" is the reclaimable build-output
  figure (`regenerableBytes`), the same number the rows and the Reclaimable stat use.

Keyboard loop untouched: `onKeydown` is still the single `<svelte:window>` handler and
still bails when `e.target` is an INPUT/TEXTAREA/SELECT or contenteditable, which covers
the rail's path field exactly as it already covered the search box. The main column is
not its own scroll container, so `move()`'s `scrollIntoView({block:'nearest'})` behaves
as before; the rail is `sticky` and stays put. `Escape` still clears marks and
`selected`; on triage/risk the rail then falls back to the cursor row, which is correct.

Files: `src/lib/domains/projects/components/ProjectAudit.svelte` (only file touched; the
large diff is the `{#if report}` block re-indenting one level into the new main column).

Arithmetic (deltas off this issue's own measurements, 1280x800 → 760px viewport, first
row was y=676): −250 scan card, −16 its gap, −116 stat grid → strip (150 → 34), −4 from
`space-y-4` → `space-y-3` = **first row at y≈290**. At ~53px row pitch that is **8 rows
at 1280x800** (was 1) and, allowing the tab+filter row to wrap at that width,
**~4 rows at 1024x600** (was 0). Lower than the 11/7 in the Fix note above, which
assumed y=180 and ignored the inter-block gaps and the shortcut legend.

Checks: `npx eslint src/lib/domains/projects src/routes/projects/audit` → 0 errors
(69 pre-existing warnings elsewhere in the domain, unchanged); `pnpm check` →
`0 ERRORS`, and no warning in `ProjectAudit.svelte`; `prettier --check` clean.

Not done: the repro script could not be run — no app instance was listening on CDP 9222
and this session was told not to start the dev server, so the y-coordinates above are
arithmetic, not measured. Please re-measure at both sizes when verifying, and check the
below-`md` Sheet path (the `Scan and details` button) since that is new.

## Verification

2026-11-09 — tauri, real data (194 projects). At 1280x800 the first two triage rows
(`portal_desktop`, `pocket_portal`) are visible immediately below the tab/filter row, with a
one-line `PageStats` strip (`Projects 194 Active 46 At risk 19 Duplicates 18 Undecided 194
Reclaimable —`) replacing the old 6-card grid, and the right rail showing Scan (collapsed to a
`Scan and import` button + `2 folders to scan` disclosure), Selected project (full detail for
the cursor row) and Marked sections — matches the Fix exactly. At 1024x600 the first triage row
(`portal_desktop`) is on screen too (it was 0 before). Screenshots:
`.claude/qa/shots/QA-UX009-after-1280x800.png`, `.claude/qa/shots/QA-UX009-after-1024x600.png`.
Did not reach the below-`md` Sheet path or re-test `j/k/1-4/o/x/esc` live — the app went down
mid-sweep (QA-104's investigation surfaced a dead vite dev server, see report). **Verified** on
the row-visibility claim, the rail layout and both window sizes measured.

---
id: QA-001
title: Projects list filter selects each stack full-width on their own row
status: verified
severity: minor
kind: ux
area: projects
mode: browser
found: 2026-11-05
---

## Use case
Narrow the project list by framework / status / package manager and change the sort.

## Steps
1. Open `/projects` at 1280x800 (and 1024x600).

## Expected
Search box, the four selects and the Asc/Desc button share a wrapping toolbar row, each select at its `min-w-*` width.

## Actual
Search is 384px wide; All Frameworks, All Statuses, All Package Managers and Name each take the full content width on their own row; Asc sits alone on a fifth row. The toolbar eats ~300px of a 600px-high window before the list starts.

## Evidence
`.claude/qa/shots/QA-001-before-filters-stacked.png`. Cause: `src/lib/components/ui/select.svelte:92` root div is `relative w-full`; inside the `flex flex-wrap` of `src/lib/components/shell/page-filters.svelte` each `w-full` select wraps to its own line. `src/routes/projects/+page.svelte:293-331` passes only `min-w-[...]`. Any other page that puts `Select` in `PageFilters` likely has the same stacking (not checked).

## Repro
`node .claude/qa/scripts/QA-001.mjs`

## Fix
Root cause: Select's root is `w-full` (class goes through twMerge), so each select in the PageFilters flex-wrap took a full row. Fix: callers pass a width; `src/routes/projects/+page.svelte` selects now use fixed `w-[170px]`/`w-[150px]`/`w-[210px]`/`w-[160px]` (replacing `min-w-*`). PageFilters is unchanged (its only user is this page). Checks: prettier clean; repro at 1280x800 prints `314,314,314,366 ok`, at 1024x600 `314,366,366,366 ok` (wraps, no longer one per row). eslint skipped (broken repo-wide).

## Verification

2026-11-05, browser mode (HMR build), 1280x800 and 1024x600. Repro prints 'tops 314,314,314,366 ok'. Search stays 384px, selects at their min widths, Asc right edge = content edge (1256 / 1000). At 1024 the selects wrap onto a second row, acceptable. Verified.

2026-11-05 re-verify (reworked fix), browser mode, 1280x800 + 1024x600: search 384px, Asc at content right edge, no label truncated (All Package Managers fits 210px), 1024 wraps to a second row. Still verified.
Screenshots: .claude/qa/shots/QA-001-after-filters-1280.png, QA-001-after-filters-1024.png
Earlier verify rounds: QA-001-after-filters-1024-round-0/1/2.png, QA-001-after-filters-1280-round-1/2.png

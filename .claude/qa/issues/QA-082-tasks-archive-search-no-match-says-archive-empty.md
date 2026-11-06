---
id: QA-082
title: Archive search with no match says "Archive is empty"
status: verified
severity: minor
kind: ux
area: tasks (Archive view empty state)
mode: tauri
found: 2026-11-06
---

## Steps
`/tasks?project=all&view=archive` (2 archived), type `zzzz` in search.

## Actual
Empty state reads "Archive is empty / Tasks completed more than 7 days ago and cancelled tasks show up here." while the archive holds 2 tasks; the board view uses the correct "No matching tasks ... Clear filters". Expected the same no-match copy + Clear filters when a search/filter is active.

## Evidence
`shots/sweep-tasks-v2-archive-nomatch.png`

## Verification
2026-11-06, mode: tauri, `scripts/QA-082.mjs`. Search `zzzz` in Archive: "No matching archived tasks" with Clear filters action; clicking it restores the unfiltered view and empties the search box; with no filters and empty archive: "Archive is empty". Shots `QA-082-after-archive-nomatch.png`, `QA-082-after-archive-current.png`. Caveat: the archive held 0 tasks this run, so the no-match-with-items case rested on the hasActiveFilters branch, not on a non-empty archive. Verified.

## Verification (2, non-empty archive)
2026-11-06, mode: tauri, `scripts/QA-082b.mjs`. Archive held 3 cancelled fixtures (QA Vfy Alpha/Bravo/Charlie). Search `zzzz`: "No matching archived tasks" + "Clear filters" (not "Archive is empty", not blank); Clear filters restores all 3 and empties the search box. Search `Bravo`: shows only QA Vfy Bravo (November 2026 (1)). No console errors. Shots `QA-082-after-archive-nonempty*.png`. Fixtures deleted.

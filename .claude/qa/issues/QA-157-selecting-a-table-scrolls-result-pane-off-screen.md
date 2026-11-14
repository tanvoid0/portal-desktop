---
id: QA-157
title: Selecting a source/table scrolls the Data/Query result pane off-screen when the source is below the fold
status: fixed
severity: major
kind: bug
area: database
mode: tauri
found: 2026-11-14
---

## Use case

Browse a database source's tables in the Database Explorer (`/database`) — pick a source,
pick a table, see its rows.

## Steps

1. Open `/database`. The first source ("Portal (this app)") expands inline with 280+ tables,
   always expanded (no collapse control), so any other source (an SDK-discovered Postgres, an
   opened `.db` file, etc.) sits far below the fold in the left source list.
2. Scroll the source list down to reach a later source (e.g. a SQLite file opened via
   "Open .db file…").
3. Click the source, then click one of its tables.

## Expected

The right-hand Data pane shows the table's grid (columns, PK badges, rows, Insert row /
pagination controls) where it was before — i.e. visible in the viewport.

## Actual

The right pane goes blank ("Database Explorer" header still visible, but the Data/Query
tab area is empty white space). The grid *is* in the DOM (headers, PK badge, row text are all
present and correct via `innerText()`), but it is rendered **above** the current scroll
position — `table.boundingBox()` reported `y: -520` (520px above the top of the viewport).
Scrolling the page up manually (mouse wheel) brings it back into view at `y: 233` with
everything rendered correctly.

Clicking a source/table that is scrolled down appears to scroll the *whole page* (not just
the source-list column) to follow the click target, which leaves the main content pane
scrolled out of view above the fold. This reproduced identically three separate times with a
freshly opened SQLite fixture (`qa502.db`) registered via `db_open_sqlite`, every time it was
selected after scrolling the source list down to find it.

This is a major usability problem: with Portal's own DB table list always expanded and never
collapsible, almost *any* second source (every real-world Postgres/MySQL/extra-SQLite case
this verification pass covers) is below the fold, so selecting one of its tables routinely
lands the user on an apparently-blank page with no visual cue to scroll up.

## Evidence

- `.claude/qa/shots/QA-502-qa_items-noscroll.png` — blank result pane right after selecting
  `qa502.db` → `qa_items` (sidebar visibly scrolled down from the click).
- `.claude/qa/shots/QA-502-qa_items-after-wheelup.png` — same state, after scrolling the page
  up with the mouse wheel: the grid renders correctly (id/name/note columns, PK badge,
  Insert row button, Prev/Page 1/Next pager).

## Repro

`node .claude/qa/scripts/QA-502-editing.mjs` reproduces the `boundingBox` evidence (uses a
throwaway SQLite fixture it creates itself — point it at any `.db` file registered as a
source via `db_open_sqlite` and select a table while the source list is scrolled down).

## Fix

Root cause: the source list `<aside>` in `src/routes/database/+page.svelte` had no scroll
region of its own (`space-y-3 rounded-md border p-2`, no `overflow`/`max-h`), so browsing a
long source list meant scrolling the whole page — which dragged the result pane along with it,
landing it above the viewport after a click re-rendered it taller/shorter than before.

Fix: gave the `<aside>` its own scroll region —
`max-h-[calc(100vh-14rem)] overflow-y-auto` — so it scrolls independently and the result pane
never moves. Also fixes UX-022's "no way to reach a source below Portal's 280 tables" framing
(the list now has a bounded, scrollable height instead of pushing the whole page down), though
UX-022's own ask (collapse/search) is still open.

Files: `src/routes/database/+page.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

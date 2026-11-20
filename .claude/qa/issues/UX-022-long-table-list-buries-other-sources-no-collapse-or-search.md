---
id: UX-022
title: Portal's own 280+ table list is always expanded and un-searchable, burying every other source
status: verified
severity: minor
kind: ux
area: database
mode: tauri
found: 2026-11-14
---

## Use case

Pick a different source (a discovered Postgres service, a second SQLite file, Portal's own
DB) in the Database Explorer's left source list.

## Steps

1. Open `/database`. "Portal (this app)" is the first source, and its table list — 280+
   tables — renders fully expanded immediately, with no way to collapse it (clicking the
   source row/label does not collapse it; the switch next to it only toggles edit mode).
2. Try to find a second source (e.g. an SDK-discovered Postgres, or a `.db` file opened via
   "Open .db file…").

## Expected

Either the first source starts collapsed, or there is a search/filter for the source list
(the page already has a filter-style affordance pattern used elsewhere in the app), so
picking *any* other source doesn't require scrolling past hundreds of table rows first.

## Actual

There is no collapse control and no search box over the source list. Every other source sits
below ~280 rows of tables, found only by scrolling. This directly compounds QA-157 (selecting
a table that far down scrolls the result pane out of view) and makes Postgres/MySQL/extra
SQLite sources — the actual point of task #502 P1 — hard to even locate.

## Evidence

`.claude/qa/shots/qa502-sources-retry.png` (top of list, no collapse affordance on "Portal
(this app)"), `.claude/qa/shots/QA-502-qa_items-after-wheelup.png` (scrolled ~280 rows down to
reach `PostgreSQL` / `qa502.db`).

## Repro

Manual — open `/database`, scroll the source list.

## Fix

Root cause: `SourceNode.svelte` rendered every table of a SQL source unconditionally and its header
had only the edit `Switch` and the engine badge — nothing to collapse, nothing to search — so
Portal's own 280+ tables always sat above every other source.

Fix (SQL sources only; Mongo and Redis headers and bodies are untouched):

- The header gets a chevron button (shadcn `Button`, `aria-expanded`, `aria-controls` → the table
  list) that collapses and expands the list.
- A source with more than 30 tables starts collapsed (Portal's own DB loads as one header row);
  smaller ones start expanded, as before.
- Those big sources also get a filter box under the header: case-insensitive substring match,
  "N of M tables" while filtering, "No tables match" when nothing does. It stays visible while
  collapsed, an active filter forces the list open, and collapsing a filtered list clears the filter.
- Threshold, filter and which of filter/toggle wins live in a pure function,
  `utils/tableList.ts`, pinned by `utils/tableList.test.ts`.
- `src/routes/database/+page.svelte` is untouched: QA-157's `max-h`/`overflow-y-auto` on the source
  rail still bounds an expanded 280-row list.

Trade-offs: the chevron costs a SQL header's label ~28px (it was already truncating in the 16rem
rail; the hover `title` is unchanged), and sources of 30 tables or fewer get no filter box. No
other QA ids; UX-000 does not track UX-022.

Files: `src/lib/domains/database/components/SourceNode.svelte`,
`src/lib/domains/database/utils/tableList.ts`, `src/lib/domains/database/utils/tableList.test.ts`.
Checks: `pnpm check` and `pnpm lint` (0 errors), `vitest run --project server` (green, including
the new `tableList.test.ts`). Not driven live (no dev instance was run).

## Verification

2026-11-20, tauri (dev instance). `scripts/UX-022-verify.mjs`: Portal source (53 tables in the dev copy, >30) loads collapsed (`aria-expanded=false`, 0 rows, filter box visible); chevron expands to 53 rows; filter `agent_` -> "3 of 53 tables"; `zzzqq` -> "No tables match"; collapsing clears the filter. Shots: `.claude/qa/shots/UX-022-after-collapsed-initial-2.png`, `UX-022-after-expanded.png`, `UX-022-after-filter-agent.png`. `scripts/qa-database-explorer.mjs` now expands the Portal row first. Result: gone. Nit, not reopened: Portal's source label truncates to "Portal ..." (the accepted chevron trade-off).

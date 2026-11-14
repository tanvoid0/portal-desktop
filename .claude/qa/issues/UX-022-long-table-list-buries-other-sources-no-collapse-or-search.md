---
id: UX-022
title: Portal's own 280+ table list is always expanded and un-searchable, burying every other source
status: open
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

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

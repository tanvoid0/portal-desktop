---
id: UX-021
title: Database Explorer subtitle still advertises "(read-only)" after P2 added editing
status: fixed
severity: polish
kind: copy
area: database
mode: tauri
found: 2026-11-14
---

## Use case

A user opens the Database Explorer to understand what it can do, before trying anything.

## Steps

1. Open `/database`.
2. Read the page subtitle next to "Database Explorer".

## Expected

Copy reflects what the page actually does now that task #502 P2 shipped per-source editing
(cell edit, insert row, delete row, gated by an "Enable editing" toggle per source).

## Actual

Subtitle still reads: "Browse SQLite, Postgres & MySQL (read-only) — vault, SDK and Docker
sources auto-discovered." `(read-only)` is stale — it was true for P0/P1, not after P2.
A first-time user reading this would not realize editing exists at all until they notice the
small per-source toggle switch.

## Evidence

`.claude/qa/shots/qa502-sources-retry.png` — header visible top of screen.

## Repro

Manual — open `/database`, read the header subtitle.
`src/routes/database/+page.svelte` (PageHeader `description` prop) is where the string lives.

## Fix

Updated the `PageHeader` `description` to "Browse & edit SQLite, Postgres and MySQL — vault,
SDK and Docker sources auto-discovered", dropping the stale "(read-only)".

Files: `src/routes/database/+page.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

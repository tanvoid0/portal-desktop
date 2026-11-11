---
id: QA-133
title: "Projects: misleading loading states — /projects first paint shows \"Total 0 / Loading projects…\" on a 196-project DB, and /projects/[id] detail hangs on \"Loading project details…\" while /pipelines loads fine"
status: open
severity: minor
kind: bug
area: projects
mode: tauri
found: 2026-11-11
---

## Use case

A developer opens the Projects list and a project's detail page and expects them to show
their data (or a skeleton), not a state that reads as "empty / broken."

## Steps

1. Navigate `/projects`. Observe the stats row and list on first paint.
2. Navigate `/projects/201`. Observe the page.
3. Compare with `/projects/201/pipelines`.

## Expected

Real counts (or skeletons) immediately; the detail route resolves.

## Actual

- `/projects` first paint shows **"Total 0"** and **"Loading projects…"** even though the DB
  has **196** projects — the stats flash 0 while the list query is still pending, reading as
  an empty app.
- `/projects/201` (detail) stayed on **"Loading project details…"** (did not resolve in the
  observed window), while `/projects/201/pipelines` loaded normally.

## Why it matters

The empty/zero states on a populated instance are misleading, and a hanging detail route
blocks the primary project view.

## Suggested fix

Show skeletons with no fabricated "0"; load stats and list from one settled query; fix the
`/projects/[id]` detail route's pending/hang path (it may share the slow query the list hit).

## Evidence

- CDP `:9223`: `/projects` dump showed "Total 0 / Loading projects…" on first paint, "Total
  196" after; `/projects/201` body = "Loading project details…".

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

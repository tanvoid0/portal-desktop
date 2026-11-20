---
id: QA-133
title: "Projects: misleading loading states — /projects first paint shows \"Total 0 / Loading projects…\" on a 196-project DB, and /projects/[id] detail hangs on \"Loading project details…\" while /pipelines loads fine"
status: verified
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

2026-11-20, tauri (dev instance), live repro of 133b, `scripts/QA-133b-live.mjs`. NOT reproduced.
- `/projects` hard load: stats read "Total - / Active this week - ..." at ~0.8s (dashes, no fabricated 0), "Total 170" by 1.5s. Shot `.claude/qa/shots/QA-133b-live-projects-list.png`.
- `/projects/15` via client nav: resolved to the project page within 2.5s; cold hard-load shows "Loading project details..." at 0.5s and resolves by 2s, no hang over 10s. Shot `.claude/qa/shots/QA-133b-live-project-detail-id.png`.
Left `open` as instructed; 133a already fixed, this is the live-repro remainder. Caveat: dev copy of the DB (170 projects, not 196), release daemon Foreign.

2026-11-20, DB-perf disproof against the **real release DB** (`%APPDATA%\com.tanvoid0.portal-desktop\portal_desktop.db`, 21 MB, 172 projects — the "196" was an older state). Replayed the exact backend query pattern in-process (`.claude/qa/scripts/qa-133b-dbperf.py <copy-of-real-db>`):
- `get_all` (projects list) is N+1: 1 list query + 3 per project (frameworks / package_managers / languages, `project_repository.rs:63` `to_response`). On the real DB: **517 queries, 33.9 ms total**, every join lookup on `idx_project_*_project_id`.
- `get_by_id` (detail) = 4 queries = **0.35 ms**.
No slow query exists at this scale, so the "Total 0 / detail hang" was never DB-performance; 133a's skeleton/dashes fix covers the misleading-state symptom. **Verified — closed.**
- Latent, not a current bug: the N+1 would matter at thousands of projects; trivial fix is batching the 3 join reads with `WHERE project_id IN (...)`. Noted for later, not fixed (34 ms today).

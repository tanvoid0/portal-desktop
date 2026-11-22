---
id: QA-168
title: "Tasks project rail shows \"Project #N\" placeholders while projects load, and permanently for a dangling project link"
status: verified
severity: minor
kind: bug
area: tasks
mode: tauri
found: 2026-11-22
---

## Use case

Browse tasks by project in the left rail.

## Steps

1. Open /tasks and reload (F5 / `goto`) several times, watching the rail.
2. Wait for it to settle, read the rail rows.

## Expected

Rail rows always show a project name (or a neutral skeleton while loading); a task linked to a project that no longer exists gets a clear label such as "Unknown project".

## Actual

- During load the rail briefly lists `Project #228`, `#176`, `#61`, `#64`, `#17`, `#15`, `#14`, `#12`, `#10`, `#9` (this is the flash the end-user saw once; reproduced on 3 of 3 reloads).
- After load a row `Project #1` (3 tasks) stays forever: `get_all_projects` (171 rows) has no id 1, so these 3 tasks point at a missing project. Their cards even carry the tag `Portal Desktop`.
- Cause: ``nameOf = id => projectNames.get(id) ?? `Project #${id}` `` in `src/lib/domains/tasks/state/taskManager.svelte.ts:136`; `projectNames` is empty until `projectsQuery` resolves and never contains deleted projects.

## Evidence

`.claude/qa/shots/QA-168-before-project-placeholder.png` (settled rail with `Project #1`), `.claude/qa/shots/sweep-tasks-light-b.png`.

## Repro

`node .claude/qa/scripts/QA-168.mjs` (prints the placeholders seen per reload)

## Fix

Root cause: `nameOf` fell back to `Project #id` before projectsQuery resolved and for deleted projects. Fix: sidebar rows hidden while projects query is pending; fallback now `Unknown project`. File: src/lib/domains/tasks/state/taskManager.svelte.ts. Checks: pnpm check (0 errors), eslint/prettier on touched files.

## Verification

2026-11-22, tauri. node QA-168.mjs: no 'Project #N' seen during 3 reloads. Settled rail shows 'Unknown project' for the dangling link (QA-168-after-tasks-rail.png).

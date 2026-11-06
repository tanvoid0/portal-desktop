---
id: QA-019
title: /logs says "No logs found" while the footer counts 9 logs
status: verified
severity: major
kind: bug
area: logs
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens Application Logs to read frontend/backend logs.

## Steps

1. Open `/logs` (filters: level `all`, source `all`, empty search).

## Expected

The 9 entries are listed.

## Actual

List says "No logs found"; footer says "Showing 0 of 9 logs". The same session logged many `[FRONTEND] [ERROR]` lines
(visible in the console), so there is data. Also: only a level and a source filter are visible, the context filter is not.

## Evidence

`.claude/qa/shots/QA-019-before-logs-empty-but-9.png`.
Suspect: `UnifiedLogViewer.svelte` `filteredLogs = $derived.by(() => logger.getFilteredEntries(...))` reads a non-reactive
logger, so it is computed once at mount (empty) and never recomputed when `logs` updates.

## Repro

`node .claude/qa/scripts/QA-019.mjs`

## Fix

Root cause: `filteredLogs`/`availableContexts` $derived read the non-reactive logger only, so computed once at mount. Both now read `logs` (updated by logger.onLog) so they recompute; context filter appears once entries have contexts. File: `src/lib/components/logs/UnifiedLogViewer.svelte`. Checks: pnpm check 0 errors.

Review follow-up: UnifiedLogViewer.svelte and routes/logs/+page.svelte (previously gitignored, now untracked) switched `@/lib` imports to `$lib` and prettier-formatted; no raw controls or derived closures; eslint clean with --no-ignore. They are not in .prettierignore, so nothing there to remove.

## Verification

<!-- qa-tester -->

2026-11-06, tauri. /logs now lists entries: 'Showing 9 of 9 logs', 3 filters incl. context. Shot shots/QA-019-after-logs-list.png.

2026-11-06 (final round), tauri after rebuild. /logs lists entries, 'Showing 9 of 9 logs', 3 filters. shots/QA-019-after-final-logs.png.

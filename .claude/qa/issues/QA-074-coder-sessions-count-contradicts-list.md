---
id: QA-074
title: /coder sidebar says "2 sessions" while 11+ sessions are listed beneath it
status: verified
severity: polish
kind: ux
area: coder
mode: tauri
found: 2026-11-06
---

## Use case

Find an earlier coder session.

## Steps

1. `/coder`, read the Sessions header count, count the rows below (test-project group shows 9, plus others).

## Expected

Count matches what is listed, or says what it counts (e.g. "2 in portal_desktop").

## Actual

"2 sessions" under the search box; list shows 11 rows with message counts across "OTHER FOLDERS". Possibly the count is for the selected workspace (portal_desktop) while the list is "All" - either way the label does not say so.

## Evidence

`.claude/qa/shots/QA-074-before-sessions-count.png`

## Repro

`node .claude/qa/scripts/QA-074.mjs`

## Fix

`CoderSessionList.svelte`: count now spans all workspace groups (filtered by search/status/provider), matching the list's own `project: 'all'` scope, instead of only the active workspace. Collapsed project groups still hide their rows, so the count can exceed visible rows (22 vs 11 on this data).

## Verification

<!-- qa-tester -->

## Verification (2026-11-06, driver)

The count is no longer scoped to the active workspace — it now totals every session
across all workspace groups, so it reads "22 sessions". But collapsed project groups
hide their rows, so the header still does not match what is on screen (22 vs 11 rows
visible on load). Either label it as a total ("22 across 4 workspaces") or count only
the expanded/visible rows. Product call, hence `needs-decision`.

## Resolution (2026-11-06)

Decision taken: keep the count spanning every listed workspace, and say so, since collapsed
groups legitimately hide rows. The header now reads "22 sessions in 3 workspaces" (the
workspace clause only appears when the count spans more than one). Verified live:
`shots/QA-074-after-session-count-scope.png`. `CoderSessionList.svelte` derives the workspace
count from the same filtered list the session count comes from, so the two cannot drift.

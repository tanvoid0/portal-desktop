---
id: QA-053
title: Environment "Add variable" appends the row at the end of a ~45-row list with no scroll or focus
status: verified
severity: minor
kind: ux
area: utilities/environment
mode: tauri
found: 2026-11-06
---

## Use case

Owner adds a new user/system environment variable.

## Steps

1. Open `/utilities/environment` (User tab).
2. Click Add variable (nothing was saved while testing).

## Expected

The new empty row is scrolled into view and its Name input focused.

## Actual

The only feedback is a "1 unsaved change(s)" chip; the new row is off-screen at the bottom and focus stays on the Add button. Leaving the page through the sidebar with an unsaved change drops it silently (no guard). Related copy: searching for a name that matches nothing shows "No variables in this section yet." (should say nothing matches the search).

## Evidence

`.claude/qa/shots/QA-053-before-add-variable-offscreen.png`, `.claude/qa/shots/sweep-env-search-empty.png`

## Repro

`node .claude/qa/scripts/QA-053.mjs` (does not save)

## Fix

Root cause: addRow only appended; no focus, no navigation guard, empty-state copy ignored search.
File: src/lib/domains/utilities/components/EnvironmentVariablesPage.svelte. addRow clears that scope's search (an active search would hide the new empty row), then `tick()` and focuses `#name-<id>` (focus scrolls it into view). `beforeNavigate` guard: when dirtyCount>0 cancels and asks `confirmAction(..., { confirmLabel: 'Leave', destructive: true })`, then `goto(to.url)`; guard keys on `buildChanges(original, rows).length > 0` (not dirtyCount); skips unload and cross-origin navigations; on popstate uses `history.go(delta)` so Back history stays correct. Empty state shows `No variables match "<q>".` when searching.
Checks: pnpm check 0 errors, eslint 0 errors, prettier clean. Not exercised in the live app.

## Verification

2026-11-06, tauri. Add variable focuses the new Name input (in view); shot `.claude/qa/shots/QA-053-after-add-variable-focused.png`. Leaving via sidebar with a draft opens 'Unsaved changes - Leave and discard them? Cancel / Leave'; Cancel stays; Discard then leaving navigates without a prompt (shot `.claude/qa/shots/QA-053-after-unsaved-prompt.png`). A search with no match shows `No variables match "zzzqq".` Nothing was saved. Verified.

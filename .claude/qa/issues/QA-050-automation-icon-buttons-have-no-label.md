---
id: QA-050
title: Icon-only Edit/Download/Delete buttons on Scripts and Blocks cards have no accessible name
status: verified
severity: minor
kind: a11y
area: automation/scripts, automation/blocks, scripts/new
mode: tauri
found: 2026-11-06
---

## Use case

Keyboard / screen-reader user edits, exports or deletes a script or block.

## Steps

1. Open `/automation/scripts` and `/automation/blocks`.
2. Inspect the pencil, download and red trash buttons on each card (and the back arrow on `/scripts/new`).

## Expected

Each icon-only button has `aria-label` (or title): "Edit script", "Export script", "Delete script".

## Actual

No accessible name: 68 buttons on `/automation/scripts`, 82 on `/automation/blocks`, 1 on `/scripts/new` (back arrow). A screen reader announces just "button". Same class of problem as QA-046.

## Evidence

`.claude/qa/shots/QA-050-before-icon-buttons-unlabeled.png`

## Repro

`node .claude/qa/scripts/QA-050.mjs` (prints the count of unnamed buttons per route)

## Fix

Root cause: icon-only `Button`s had no aria-label.
Files: src/lib/domains/automation/components/ScriptsPage.svelte (Edit/Export/Delete/Copy, label includes script name), BlocksPage.svelte (block + template Edit/Export/Delete, remove-parameter), src/routes/scripts/new/+page.svelte (back arrow "Back to scripts", remove-parameter).
Checks: pnpm check 0 errors, eslint 0 errors, prettier clean on touched files. Not rerun against the live app.

## Verification

2026-11-06, tauri. Unnamed buttons now 0 on /automation/scripts, /automation/blocks, /scripts/new (was 68/82/1). Shot `.claude/qa/shots/QA-050-after-icon-buttons-labeled.png`. Verified.

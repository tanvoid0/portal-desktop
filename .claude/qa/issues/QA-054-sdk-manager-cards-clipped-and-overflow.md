---
id: QA-054
title: SDK Managers cards clip names and badges and the page scrolls sideways
status: verified
severity: major
kind: bug
area: sdk/manager, sdk/ai/ollama
mode: tauri
found: 2026-11-06
---

## Use case

Owner reviews installed SDK version managers (Conda, fnm, NVM, Pyenv, Rustup).

## Steps

1. Open `/sdk/manager` at 1280x800 (also 1024x600, also dark).

## Expected

Card titles wrap or ellipsize, status badges stay inside the card, no horizontal scroll.

## Actual

"Fast Node Manage" and "NVM (Node Versio" are cut off, Installed / Not Installed badges poke out of the card edge (Rustup's sits outside the card), a third column is cut by the viewport and a horizontal scrollbar appears (scroll container 875px in a 694px pane). At 1024 titles are cut further ("Fast Node M"). `/sdk/ai/ollama` Service tab has the same sideways scroll: the button row (Stop, Restart, View Logs, Check Updates, "Update Ollam...") is clipped. Possibly a regression from the narrowed domain sidebar / `shell-sidebar-layout` change.

## Evidence

`.claude/qa/shots/sweep-sdk-manager-loaded.png`, `.claude/qa/shots/sweep-sdk-manager-1024x600.png`, `.claude/qa/shots/sweep-sdk-manager-dark.png`, `.claude/qa/shots/sweep-sdk-ai-ollama-loaded.png`

## Repro

`node .claude/qa/scripts/app.mjs sdk/manager QA-054-rerun` (data takes ~10s, see QA-056; use s5c.mjs to wait)

## Fix

Root cause: manager grid used viewport breakpoints (`lg:grid-cols-3`) while the pane is narrower (sidebar), and card header had nowrap title/badge with no `min-w-0`, so content overflowed the cell and the scroll container. Not the QA-015 sidebar width. Fix: `/sdk/manager` grid is now `auto-fill minmax(min(100%,20rem),1fr)`, card `min-w-0`, header wraps, title wraps, badge `shrink-0`; Ollama Service/Config/Log button rows got `flex-wrap`. Files: src/routes/sdk/manager/+page.svelte, src/routes/sdk/ai/ollama/+page.svelte. Layout/app.css untouched. Checks: pnpm check 0 errors, eslint 0 errors, prettier on touched files. Not visually verified (app is under QA).

## Verification

2026-11-06, tauri. /sdk/manager at 1280, 1024x600 and dark: one card per row, titles and badges inside cards, no horizontal overflow (scrollWidth checked). Ollama Service buttons wrap, no sideways scroll. /settings and /utilities/disk unaffected. Shots `QA-054-after-sdk-manager-1280.png`, `-1024x600.png`, `-dark.png`, `QA-054-after-ollama-service.png`, `QA-054-after-neighbour-settings.png`. The /sdk overview still overflows sideways: filed as QA-062. Verified.

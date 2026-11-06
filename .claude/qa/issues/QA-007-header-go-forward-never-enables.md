---
id: QA-007
title: Header "Go forward" button stays disabled after "Go back"
status: verified
severity: major
kind: bug
area: shell/header
mode: tauri
found: 2026-11-06
---

## Use case

Go back to the previous page with the header arrow, then forward again.

## Steps

1. Open `/settings/general`, click Terminal in the sidebar (`/terminal`).
2. Click header "Go back" (returns to `/settings/general`).
3. Look at / click "Go forward".

## Expected

Forward is enabled and returns to `/terminal`.

## Actual

Forward is disabled (greyed) a moment after going back, so it can never be used. `history.forward()` does work, so the history entry exists; the button state is wrong.

Likely cause (`src/lib/components/layout/app-shell.svelte` ~L94-209): `goBack()` sets `canGoForward = true`, but the `$effect` that runs on page change reads `hasNavigatedBack`; the 100 ms timeout sets `hasNavigatedBack = false`, which re-runs the effect, which sets `canGoForward = false`. Reading real state (Navigation API `navigation.canGoForward`, or an index in `history.state`) avoids the flag race.

## Evidence

`.claude/qa/shots/QA-007-before-forward-disabled.png`

## Repro

`node .claude/qa/scripts/QA-007.mjs`

## Fix

Root cause: `canGoForward` was driven by a `hasNavigatedBack` flag whose 100 ms reset re-ran the page effect and cleared forward. Now `updateNavigationState()` reads the Navigation API (`navigation.canGoBack/canGoForward`, history.length fallback); flags removed. Files: `src/lib/components/layout/app-shell.svelte`. Checks: pnpm check 0 errors; eslint + shadcn check clean on touched files.

## Verification

2026-11-06, tauri. QA-007.mjs: Go forward enabled after back and history.forward works. Chain Overview > Terminal > back > forward lands on /terminal; forward disables after a fresh navigation. After-shot: shots/QA-007-after-header-nav.png

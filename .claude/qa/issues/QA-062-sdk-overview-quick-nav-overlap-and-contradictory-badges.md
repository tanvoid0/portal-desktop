---
id: QA-062
title: SDK overview Quick Navigation cards overlap, page scrolls sideways, and some cards say "Not installed" while green or Running
status: verified
severity: minor
kind: bug
area: sdk
mode: tauri
found: 2026-11-06
---

## Use case

Owner scans `/sdk` to see which SDKs, managers and databases are installed.

## Steps

1. Open `/sdk` at 1280x800, wait for load, scroll to Quick Navigation.

## Expected

Cards stay in their grid cell; names and version strings truncate; status text matches the colour.

## Actual

- SDK Managers row: "Fast Node Manager", "NVM (Node Version Manager)" and "Pyenv" run into each other (text overlaps across cards).
- Long version strings ("rustc 1.94.1 (e408947bf 2026-03-25)", "go version go1.26.2 windows/amd64") are not trimmed.
- The overview scroll container is wider than its pane (757 > 694 px), so there is a horizontal scrollbar.
- NVM card is green with a check but reads "Not installed" (`/sdk/manager` says Installed). Database Server shows "PostgreSQL Not installed Running" and "MySQL Not installed".

## Evidence

`.claude/qa/shots/QA-062-before-overview-quicknav-overlap.png`

## Repro

`node .claude/qa/scripts/v9.mjs`

## Fix

Root cause: Quick Navigation cards sat in a viewport-breakpoint grid (`lg:grid-cols-4`) in a narrower pane, buttons were nowrap with no `min-w-0`/overflow clip, and the status text said 'Not installed' whenever a version string was missing (NVM has none), while a running service with a failed binary probe (PostgreSQL) showed 'Not installed Running'. Fix: auto-fill grid, cards `min-w-0 overflow-hidden`, inner rows `w-full min-w-0` so version strings truncate; text is 'Installed' when installed without a version; installed = sdk_installed || service_running. Files: src/routes/sdk/+page.svelte. pnpm check 0 errors, eslint 0 errors, prettier clean. Not viewed in the app.

## Verification

<!-- qa-tester -->

2026-11-06 (final round), tauri after rebuild. /sdk: no horizontal overflow (1280=1280), Quick Navigation grid uses auto-fill cards with no overlap, badges consistent (PostgreSQL 'Installed' + Running, NVM-style cards no longer 'Not installed' while green). shots/QA-062-after-quicknav.png, QA-062-after-quicknav-scrolled.png.

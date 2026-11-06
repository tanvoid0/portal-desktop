---
id: QA-010
title: Breadcrumb shows only "Home" after opening Settings from the sidebar footer gear
status: verified
severity: minor
kind: bug
area: shell/breadcrumbs
mode: tauri
found: 2026-11-06
---

## Use case

Orient yourself with the header breadcrumb after navigating.

## Steps

1. From Overview (`/`), click the gear icon at the bottom of the sidebar.
2. URL becomes `/settings/general`; read the header breadcrumb.

## Expected

`Home > Settings > General`.

## Actual

Only `Home` (stale). Direct load of the same URL shows the full crumb. On that in-app visit the page header also gains `Configured: 4/7 / Theme / Language` chips a moment after load, pushing everything down ~40 px (layout shift).

## Evidence

`.claude/qa/shots/QA-010-before-settings-crumb-home.png`

## Repro

`node .claude/qa/scripts/QA-010.mjs`

## Fix

Stale-breadcrumb part only (settings-header chip layout shift belongs to the settings coder, not touched). Root cause: `breadcrumbItems` (svelte-store `derived`) read rune state `page.url` without tracking it, and pages never clear custom breadcrumbs, so stale ones persisted across routes. Fix in `src/lib/domains/shared/stores/breadcrumbStore.ts`: `setBreadcrumbs` records `page.url.pathname`, the derived uses custom items only while the path matches, and tracks navigation via `toStore(() => page.url.href)` as an input. Checks: pnpm check 0 errors; eslint clean.

## Verification

2026-11-06, tauri. QA-010.mjs: footer gear gives Home > Settings > General. Reviewer case: /projects (crumb Home > Projects) then footer gear gives Home > Settings > General; Back restores Home > Projects; /projects/create and /projects/audit keep their custom crumbs. The settings-header chip shift (second half of the original report) did not reproduce on retest (h2 top stable at 136px). After-shot: shots/QA-010-after-settings-crumb-from-projects-2.png

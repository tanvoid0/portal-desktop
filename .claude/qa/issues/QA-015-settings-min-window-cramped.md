---
id: QA-015
title: Settings at 1024x600: content pane is 448px wide, buttons clip behind horizontal scroll
status: verified
severity: minor
kind: ux
area: settings
mode: tauri
found: 2026-11-06
---

## Use case

Use Settings on a small laptop window (minimum 1024x600).

## Steps

Open `/settings/ides` at 1024x600.

## Expected

Usable content area; action buttons reachable without sideways scrolling.

## Actual

The 280px Settings nav (two-line items) plus the wrapped page header (Save Changes drops to its own row) leave a 448x428 content pane. On IDEs the "IDE Configuration" title wraps to two lines and "Add IDE" is clipped off the right edge behind a horizontal scrollbar. Suggest a top tab strip or narrow rail below ~1100px.

## Evidence

`.claude/qa/shots/QA-015-before-settings-min-window.png`; also `sweep-settings-autonomy-dark-1024.png`.

## Repro

`node .claude/qa/scripts/QA-015.mjs`

## Fix

Root cause: 20rem domain sidebar plus header that went to a row at md. Domain sidebar var is 14rem under 1280px (src/app.css; also affects cloud/sdk/utilities), settings header stacks until xl so actions wrap instead of pushing content.
Files: src/app.css, src/routes/settings/+layout.svelte.
pnpm check: 0 errors; eslint 0 errors; prettier clean on touched files (remaining prettier warnings: CLAUDE.md, projects/generated/UnavailableProject.ts, pre-existing); shadcn check passed. Not run in-app (tester verifies).

## Verification

2026-11-06, tauri, 1024x600. Header actions now wrap above the content and Add IDE is fully in view on /settings/ides; no document-level horizontal scroll; /sdk /cloud/workloads /utilities render without overflow. Residual, not reopened: the pane still shows a thin inner horizontal scrollbar and the IDE Configuration title wraps (polish). After-shot: shots/QA-015-after-settings-min-window.png

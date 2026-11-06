---
id: QA-009
title: Settings "Save Changes" and "Export Settings" stay disabled after a reload / direct open
status: verified
severity: major
kind: bug
area: settings
mode: tauri
found: 2026-11-06
---

## Use case

Open any Settings page, change a value, click Save Changes.

## Steps

1. Open `/settings/general` directly, or click the header Refresh button while on any Settings page.
2. Change Time Format (or anything).
3. Try to click Save Changes / Export Settings.

## Expected

Both buttons enabled once settings have loaded.

## Actual

Both are permanently disabled; edits cannot be saved until you leave Settings and come back through in-app navigation (that path works: toast "Settings saved successfully", value persisted; I restored 24h afterwards). No hint why they are disabled.

Cause (`src/routes/settings/+layout.svelte` ~L151 and L199): `disabled={isSaving || !get(settings)}` is a one-off, non-reactive read; on a cold load the store is still null at first render so the attribute is never updated. Use `$settings`.

## Evidence

`.claude/qa/shots/QA-009-before-save-disabled-after-reload.png`

## Repro

`node .claude/qa/scripts/QA-009.mjs`

## Fix

Root cause: `!get(settings)` is a one-off non-reactive read in the template. Now `!$settings`.
Files: src/routes/settings/+layout.svelte.
pnpm check: 0 errors; eslint 0 errors; prettier clean on touched files (remaining prettier warnings: CLAUDE.md, projects/generated/UnavailableProject.ts, pre-existing); shadcn check passed. Not run in-app (tester verifies).

## Verification

2026-11-06, tauri. QA-009.mjs: after the header Refresh on /settings/general Save and Export are enabled. After-shot: shots/QA-009-after-save-enabled.png

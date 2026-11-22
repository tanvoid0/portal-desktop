---
id: QA-174
title: "Automation > Utilities tab says \"No scripts found\" while the Scripts tab lists 20+ scripts, many of category utility"
status: verified
severity: minor
kind: ux
area: automation/utilities
mode: tauri
found: 2026-11-22
---

## Use case

Find and run a utility script.

## Steps

1. /automation/scripts: many scripts, e.g. Lint with ESLint (utility).
2. Click the Utilities tab.

## Expected

Either the same utility scripts, or an empty state that explains what this tab holds and how it differs from Scripts.

## Actual

"No scripts found - Get started by creating your first custom script" and a second New Script button. The name also collides with the top-level Utilities nav item (disk cleanup, env vars, uninstaller). Product call: what is this tab for?

## Evidence

`.claude/qa/shots/QA-174-before-utilities-tab-empty.png`.

## Repro

manual - steps above

## Fix

Utilities tab (/automation/utilities) now renders ScriptsPage, the same list as the Scripts tab. Root cause: tab rendered UtilitiesPage, which reads the separate custom_scripts store (empty) while Scripts reads blocks. Files: src/routes/automation/utilities/+page.svelte. Checks: pnpm check (0 errors). UtilitiesPage.svelte left in place, now unrouted.

## Verification

2026-11-22, tauri. /automation/utilities now renders the same list as Scripts (QA-174-after-automation-utilities.png). Note: its heading and breadcrumb read 'Scripts' under the Utilities tab.

## Owner decision (2026-11-22)

Show the scripts: Utilities tab must list the same scripts the Scripts tab does (or merge into one list).

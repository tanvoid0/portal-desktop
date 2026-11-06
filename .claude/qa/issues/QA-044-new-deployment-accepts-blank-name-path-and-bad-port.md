---
id: QA-044
title: New deployment: Deploy enables for whitespace-only name/path and an out-of-range port
status: verified
severity: minor
kind: bug
area: deployments/new
mode: tauri
found: 2026-11-06
---

## Use case

Fill the "Create New Deployment" form and press "Deploy (build & run)". The submit runs a Docker build and starts a container, so bad input should be stopped before it.

## Steps

1. Open `/deployments/new`.
2. Name = three spaces, Project Path = three spaces, Exposed Port = `99999` (also `-5`).
3. Look at "Deploy (build & run)". (Never clicked in testing.)

## Expected

Button stays disabled or an inline error appears: blank name/path rejected after trim, port limited to 1-65535, path checked for existence.

## Actual

Button enabled, no inline message. `handleSubmit` only checks `!name || !projectPath` (`src/routes/deployments/new/+page.svelte:104`), so whitespace passes and the port is sent as is. The path field also has no folder picker and a unix-style placeholder (`/path/to/project`) on Windows. Errors only surface later as a toast.

## Evidence

`.claude/qa/shots/QA-044-before-deploy-enabled-with-invalid-input.png`

## Repro

`node .claude/qa/scripts/QA-044.mjs`

## Fix

Root cause: submit/disable only checked truthiness. Now trim name/path (also sent trimmed), port 1-65535 via native min/max plus `portValid` gate, inline message, aria-invalid. Not done: folder picker / path existence check / placeholder.
Files: routes/deployments/new/+page.svelte. Checks: pnpm check, eslint, prettier clean.
Follow-up (review): CLI `command` is trimmed too (check, disable, request).

## Verification

2026-11-06, tauri, never submitted. Blank name/path + port 99999: Deploy disabled with inline 'Port must be between 1 and 65535'; valid values enable it. QA-044.mjs prints true. After: shots/QA-044-after-deploy-disabled-invalid-input.png. Result: gone.

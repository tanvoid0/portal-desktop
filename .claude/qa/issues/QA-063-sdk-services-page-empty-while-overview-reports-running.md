---
id: QA-063
title: /sdk/services says "No SDK services found" while the overview reports 3 running and Ollama is Running
status: verified
severity: major
kind: bug
area: sdk/services
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens SDK Services to see which services (Ollama, databases) are running.

## Steps

1. Open `/sdk`: Services card reads "3/5, 3 running"; `/sdk/ai/ollama` shows Running on port 11434.
2. Open `/sdk/services` and wait 10s.

## Expected

The running services are listed.

## Actual

"No SDK services found - Install and configure SDKs to see their services here", although loading finishes and Refresh is enabled. The page lists services via `get_all_available_sdks` + `get_service_status(sdk.id)` per SDK (`src/routes/sdk/services/+page.svelte`), a different source from the overview, which now uses `get_all_sdk_configs`.

## Evidence

`.claude/qa/shots/QA-056-after-services-cold.png`, `.claude/qa/shots/QA-055-after-sdk-overview.png`

## Repro

`node .claude/qa/scripts/v8.mjs`

## Fix

Root cause: /sdk/services used `get_all_available_sdks` + per-SDK `get_service_status`, a different source from the sidebar/overview. Fix: page rewritten on the shared `sdkConfigsQuery` (services = configs with a service), PageLoading/PageError states, Start/Stop via `start_service`/`stop_service` then invalidate `sdk.configs`. Files: src/routes/sdk/services/+page.svelte. Checks as QA-062.

## Verification

<!-- qa-tester -->

2026-11-06 (final round), tauri after rebuild. /sdk/services lists PostgreSQL/MySQL/MongoDB/Ollama/Nginx with Running/Stopped, ports and Start/Stop, matching overview 3/5 running. shots/QA-063-after-services-page.png.

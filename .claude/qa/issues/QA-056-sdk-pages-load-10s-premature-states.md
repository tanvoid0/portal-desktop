---
id: QA-056
title: SDK area takes ~10s to load; premature empty/error states and a stray "?tab=" breadcrumb
status: verified
severity: major
kind: perf
area: sdk
mode: tauri
found: 2026-11-06
---

## Use case

Open any SDK page.

## Steps

1. Open `/sdk/services`, `/sdk/node/models` or `/sdk/software-installer`; look at the page in the first ~8s.
2. Open `/sdk/ai/ollama` and click the Models tab; read the breadcrumb.

## Expected

Skeletons, then data; stable counters; breadcrumb without query strings.

## Actual

- Domain sidebar shows "Loading SDKs..." (hourglass) for ~10s; the count goes "0 of 0", "3 of 5", finally "11 of 16 SDKs".
- `/sdk/services` renders "No SDK services found - Install and configure SDKs" and `/sdk` shows 0/6, 0/0 and "Offline" until data arrives.
- `/sdk/node/models` (typed URL) shows "Failed to load models - Model management is not available for this SDK" with a Retry that cannot succeed.
- Breadcrumb on Ollama tabs has a literal last crumb `?tab=models`.

## Evidence

`.claude/qa/shots/sweep-sdk-services-light.png`, `.claude/qa/shots/sweep-sdk-node-models-light.png`, `.claude/qa/shots/sweep-sdk-versions-light.png`, `.claude/qa/shots/sweep-sdk-ollama-tab-models.png`, `.claude/qa/shots/sweep-sdk-light.png`

## Repro

`node .claude/qa/scripts/s5a.mjs`

## Fix

Root cause (frontend): sidebar and each page fetched on their own with per-component `$state` flags (footer counted 0 of 0 while loading, services page rendered the empty card with `loading` true, overview defaulted `loading=false`). Fix: shared TanStack queries `queryKeys.sdk.configs`/`managers` used by sidebar, overview and /sdk/manager (deduped, 5 min stale); sidebar footer shows 'Loading SDKs...' until both are not pending; /sdk/services shows PageLoading; overview loading starts true; /sdk/[sdk]/models no longer offers Retry for 'not available for this SDK'; breadcrumb drops the `?tab=` crumb (breadcrumbStore.ts filters `tab`). RUST-SIDE SLOWNESS (not edited): src-tauri/portal-core/src/domains/sdk/services/language_config_service.rs `get_all_sdk_configs` (l.111-120) awaits `process_config` for every SDK in a sequential for loop, and `process_config` (l.~190-290) itself sequentially spawns a process per manager, per package manager, one for SDK detection and one for the service check; `get_all_sdk_managers` (l.138-) likewise awaits `check_manager_status` serially. Change: run configs with `futures::future::join_all` and, inside `process_config`, `tokio::join!` the manager/package-manager/sdk/service probes (join_all per vec); same for managers. Also cache check_manager_status by binary since the same manager (e.g. nvm) is probed once per SDK that lists it. Frontend note: overview `loadData` still fires N `get_service_status` and N `fetch_available_versions` (network) invokes on mount for the Services/Versions tabs; could be lazy per tab. Checks as QA-054.

Review round: queries set `refetchOnWindowFocus: false`; overview `loadData` (detect managers, N service statuses) is gone, only Versions tab fetches on demand, so the extra N+1 invokes noted above no longer run on mount.

## Verification

2026-11-06, tauri. Cold reload: sidebar shows 'Loading SDKs...' then '11 of 16 SDKs' (about 4s, was about 10s); /sdk counters show a dash while pending; /sdk/node/models no longer offers Retry; Ollama breadcrumb is Home > SDK > AI > Ollama, no `?tab=` crumb. /sdk/services never flashed empty while loading, but it finishes empty: see QA-063. Shots `QA-056-after-services-cold.png`, `QA-056-after-node-models.png`. Verified.

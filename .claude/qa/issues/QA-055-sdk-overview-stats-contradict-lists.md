---
id: QA-055
title: SDK overview counters contradict the installed SDKs and services shown elsewhere
status: verified
severity: major
kind: bug
area: sdk
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens `/sdk` to see what is installed and running.

## Steps

1. Open `/sdk`, wait ~10s for load.

## Expected

Counters match the lists: Node.js 24.14.1, Rust 1.94.1, Java 25.0.4, Go go1.26.2 are shown installed.

## Actual

- Managers "0/6, 0 installed" while `/sdk/manager` shows Conda and Rustup Installed.
- Versions "1/32 installed", Active "1", while four SDKs show green.
- System Status "SDK Managers 0 installed", "Services 0 running", Status badge "Offline", while `/sdk/ai/ollama` shows Ollama Running on 11434.
- Recommended Versions is hardcoded ("Node.js LTS 18.17.0", "Python 3.11") with Install buttons; Quick Actions has Start All / Stop All / Install Latest with no explanation (not clicked).
- Quick Navigation "Language & Runtime" shows Node, Rust, Java, Go but no Python, while the sidebar lists Python.

## Evidence

`.claude/qa/shots/sweep-sdk-after-wait.png`, `.claude/qa/shots/sweep-sdk-bottom.png`

## Repro

`node .claude/qa/scripts/s5b.mjs`

## Fix

Root cause: counters came from a hardcoded 6-manager list probed one by one, and from `fetch_available_versions` (remote release lists), while the sidebar/lists use `get_all_sdk_configs` and `get_all_sdk_managers`. Fix: /sdk overview now reads those two shared queries (src/lib/domains/sdk/queries/sdkQueries.ts): Managers installed/total, Services running/total (configs with a service), SDKs installed/total (replaces Versions and Active cards), Status from running services, Quick Navigation built from configs (Python now present) and managers, System Status uses the same numbers. Hardcoded Recommended Versions section removed (no backend data for it). Counters show an em dash while pending. Start All/Stop All/Install Latest untouched (they only switch tabs). Files: src/routes/sdk/+page.svelte. Checks as QA-054.

Review round: Managers and Services tabs now render from the same queries (hardcoded manager list, `checkManagerInstalled` probes and per-SDK `get_service_status` loop removed); Recent Activity card (never populated) removed; query errors show PageError instead of 0/0; every mutation calls `invalidateQueries(['sdk'])`; Refresh gates on query pending/fetching; the Versions tab data loads lazily only when that tab is opened.

## Verification

2026-11-06, tauri. /sdk counters now Managers 3/5, Services 3/5, SDKs 8/11, Status Online; Recommended Versions gone; Python present in Language & Runtime. Shot `.claude/qa/shots/QA-055-after-sdk-overview.png`. Remaining problems on the same page are QA-062 / QA-063. Verified.

Recheck 2026-11-06 (final round), tauri after rebuild. Managers 3/5, Services 3/5, SDKs 8/11; Managers and Services tabs show the same data (Conda/NVM/Rustup installed; PostgreSQL/MongoDB/Ollama running, MySQL/Nginx stopped). shots/QA-055-after-managers-tab.png, QA-055-after-services-tab.png.

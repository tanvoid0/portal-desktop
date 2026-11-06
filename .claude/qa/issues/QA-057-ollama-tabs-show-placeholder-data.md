---
id: QA-057
title: Ollama tabs show placeholder data: "0.0 MB" versions, "Unknown" badges, fake log, "coming soon" viewer
status: verified
severity: minor
kind: ux
area: sdk/ai/ollama
mode: tauri
found: 2026-11-06
---

## Use case

Owner inspects Ollama versions, models, config and logs.

## Steps

1. Open `/sdk/ai/ollama`, visit Version, Models, Log, Configuration tabs (read only).

## Expected

Real data, or the feature hidden or disabled.

## Actual

- Version: every row "0.0 MB".
- Models (Local): every model has an "Unknown" badge next to its size.
- Log: one invented line "Ollama service is running." plus "[Note] Real-time log viewing will be implemented soon", with Clear and Download enabled.
- Configuration: "Config file: ollama/ollama.conf" (relative stub path) with Save Configuration and Reset to Default enabled.

## Evidence

`.claude/qa/shots/sweep-sdk-ollama-tab-models.png`, `.claude/qa/shots/sweep-sdk-ollama-tab-version.png`, `.claude/qa/shots/sweep-sdk-ollama-tab-log.png`, `.claude/qa/shots/sweep-sdk-ollama-tab-configuration.png`

## Repro

`node .claude/qa/scripts/s5p.mjs`

## Fix

Root cause: the Ollama page invented placeholder content where no backend command exists, and passed through backend defaults. Fix: versions hide the size when the backend reports '0.0 MB' (Rust takes the size of the first release asset, which is often 0; ollama_manager.rs:211-217 should pick the platform asset); model family 'Unknown' (Rust defaults it because /api/tags nests family under `details`, ollama_manager.rs:358-362) is no longer rendered as a badge; Log tab shows 'Log viewing is not available yet.' with Clear/Download disabled and no invented lines; Configuration tab drops the stub path/default content, shows 'not available yet', textarea/Save/Reset disabled. Files: src/routes/sdk/ai/ollama/+page.svelte. Note: the 'Unknown size' text in the shared ModelCard.svelte only shows when size and parameter_size are absent; not touched. Checks as QA-054.

Review round: stub `loadConfiguration`/`loadLogs`, Clear/Download/Save/Reset controls, the textarea and the `OLLAMA_HOST=` placeholder were deleted; Configuration and Log tabs now show only the honest 'not available yet' text (plus the settings reference list).

## Verification

2026-11-06, tauri. Version rows show no '0.0 MB'; Models have no 'Unknown' badge; Log tab says 'Log viewing is not available yet.' with no fake line; Configuration says editing is not available yet, no stub path, no Save. Console clean on a fresh load (one transient `loadConfiguration is not defined` appeared mid-HMR, gone after reload). Shots `QA-057-after-ollama-version.png`, `-configuration.png`, `-log.png`, `-models.png`. Verified.

---
id: QA-048
title: AI provider "probe capabilities" / live catalog flags are silently dropped (snake_case invoke arg)
status: verified
severity: minor
kind: bug
area: ai/providers
mode: tauri
found: 2026-11-06
---

## Use case

Load the live model catalog with capability probing from Settings > AI or a model picker.

## Steps

Code path, found in review (same class as QA-028):
`src/lib/domains/ai/services/aiProviderService.ts:78` sends `probe_capabilities` as a
top-level invoke arg. Tauri commands take top-level args in camelCase
(`#[portal_rpc_macro::rpc]` expands to a plain `#[tauri::command]`), so the value never
arrives. `getCatalogLive` (line 89) also never sets `live`.

## Expected

The probe and live flags reach the backend (`probeCapabilities`, `live`).

## Actual

Silently dropped; the backend always sees the default.

## Evidence

Reviewer finding, 2026-11-06. Not reproduced in the UI: the agent-platform token is
currently rejected (QA-018), so the catalog can't load at all on this machine.

## Repro

manual: code path above.

## Fix

Root cause: top-level invoke args must be camelCase; `probe_capabilities` was dropped and getCatalogLive never set `live`. `aiProviderService.ts` now sends `probeCapabilities` and `live: true`. pnpm check 0 errors.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

2026-11-06 (final round), tauri after rebuild. Code-level only (catalog cannot load while token is rejected): aiProviderService.ts:78 sends probeCapabilities (camelCase), getCatalogLive passes live:true (line 89).

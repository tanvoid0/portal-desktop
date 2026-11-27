---
id: QA-192
title: Settings > AI sometimes fails with "Failed to parse AI settings JSON" and stays on Loading
status: fixed
severity: major
kind: bug
area: settings/ai/providers
mode: tauri
found: 2026-11-26
---

## Use case

Open Settings > AI > Providers to check or change the Agent Platform connection.

## Steps

1. Open `/ai/providers` (redirects to Settings > AI > Providers). Repeat; it is intermittent.

## Expected

The provider form loads every time.

## Actual

Sometimes (seen twice, in a handful of loads of that route; a later run of 15 had none) a toast says
"Failed to load configuration: Failed to parse AI settings JSON: EOF while parsing a value at line 1
column 0" and the card stays on "Loading..." until the page is opened again. Installed release 0.24.0.

Cause, from reading the code: `AISettingsService::load_settings`
(`src-tauri/portal-core/src/domains/ai/services/ai_settings_service.rs`) calls `save_settings` on every
read, and `save_settings` is a plain `fs::write` (truncate, then write). The page fires several reads in
parallel (`aiProviderSettings.svelte.ts` `loadProvider`, plus `ai_platform_health`), so one can read the
file while another has just truncated it.

Related, from code only, not observed: `platform_admin::config` turns that error into a default config
(no token), which the health check could read as "needs a token"; an app kill between truncate and write
would leave the settings file, token included, empty. In the UI, `getDefaultProvider` has no catch and
`AIProviderSettings.svelte` shows Loading while `providerConfig` is null, with no retry. The uncommitted
changes in the tree on 2026-11-26 do not touch `ai_settings_service.rs`.

## Evidence

`.claude/qa/shots/QA-192-before-providers-parse-error.png`

Console: `Failed to load Agent Platform configuration: Failed to parse AI settings JSON: EOF while parsing a value at line 1 column 0`

## Repro

`node .claude/qa/scripts/QA-192.mjs` (15 loads, prints how many failed; intermittent)

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

Root cause: load_settings rewrote the file on every read with a truncating fs::write.

Files: ai/services/ai_settings_service.rs (saves on load only when a migration changed something; save_settings is now temp file + rename), ai/platform_daemon.rs (write_atomic shared, unique temp name). The UI has no retry on a failed load; not changed.

Checks: cargo test --lib ai_settings (3 pass), platform_daemon (4 pass), cargo check and clippy.

Repro script not run (told not to touch the running app); cause confirmed by reading the code. Checks: pnpm check 0 errors; eslint 0 errors; shadcn check pass; prettier fails only on the untouched docs/status/RELEASE_NOTES.md.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

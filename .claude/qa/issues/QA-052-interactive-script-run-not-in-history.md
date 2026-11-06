---
id: QA-052
title: Script run in default Interactive Mode is never recorded in History
status: verified
severity: minor
kind: bug
area: scripts/[id]
mode: tauri
found: 2026-11-06
---

## Use case

Owner runs a script and later checks its History.

## Steps

1. Open a script detail page (Interactive Mode switch is ON by default).
2. Click Run; the terminal prints output and "Process exited with code: 0".
3. Click History.

## Expected

The run is listed.

## Actual

"No executions yet". With Interactive Mode off, the same run is recorded (status, time, duration). The default mode leaves no audit trail and nothing says so. (Delete confirm for the script also uses the generic button label "Continue" rather than "Delete".)

## Evidence

`.claude/qa/shots/sweep-scripts-run-output.png`, `.claude/qa/shots/sweep-scripts-history.png` (empty), `.claude/qa/shots/sweep-scripts-history-after-run.png` (non-interactive is recorded), `.claude/qa/shots/sweep-scripts-delete-confirm.png`.

## Repro

`node .claude/qa/scripts/s5j.mjs`, then `node .claude/qa/scripts/s5k.mjs` (edit the script id inside).

## Fix

Root cause: only `ScriptExecutionService::execute_script` wrote history rows; the interactive terminal path never recorded.
Applied: `ScriptExecutionService::record_interactive(block_id, command, wd, exit_code)` (create + update_status success/failed), Tauri command `record_interactive_execution` (rpc macro) registered in lib.rs, `scriptExecutionService.recordInteractive` wrapper, and a call in `handleInteractiveComplete` before `loadExecutionHistory()` (errors logged, not toasted). No new types, so `gen:types` not needed.
Files: src-tauri/portal-core/src/domains/scripts/services/script_execution_service.rs, src-tauri/src/domains/scripts/commands.rs, src-tauri/src/lib.rs, src/lib/domains/scripts/services/scriptExecutionService.ts, src/routes/scripts/[id]/+page.svelte (Delete confirm label done earlier).
Checks: cargo check 0 errors, pnpm check 0 errors, prettier clean, eslint 0 errors (2 pre-existing any warnings). Needs rebuild; repro not rerun.
Review follow-up: exit code None (user cancel / lookup failure) now records "cancelled"; frontend passes `startedAt` (captured in handleRun) and `record_interactive` backdates started_at so duration is real. Checks: cargo check 0 errors, pnpm check 0 errors, eslint 0 errors, prettier clean.

## Verification

2026-11-06, tauri. Ran the fixture in default Interactive Mode (exit 0), opened History: entry 'success, Duration 12ms' listed. Shot `.claude/qa/shots/QA-052-after-interactive-history.png`. Verified.

Recheck 2026-11-06 (final round), tauri after rebuild. Fixture 'QA script' (echo QA): interactive run recorded 'success', Duration 1s (real). Changed to 'ping -n 30 127.0.0.1', Run then 'Stop process': recorded 'cancelled', Duration 3s. Note: 'Close Terminal' while a run is active recorded nothing (run was not stopped via Stop). Shots QA-052-after-final-history.png, QA-052-after-final-stopped-run.png. Nit: 'success06/11/2026' has no space between status and date. Fixture deleted (confirm dialog shown), gone after reload.

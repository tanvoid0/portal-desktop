---
id: QA-071
title: Quick Run always fails: "Failed to create script execution ... FOREIGN KEY constraint failed" (any folder, even a registered project)
status: verified
severity: blocker
kind: bug
area: automation/run
mode: tauri
found: 2026-11-06
---

## Use case

Quick Run: pick a folder, queue blocks, Run ("no pipeline setup required").

## Steps

1. `/automation/run`, preset "Test" (queues "Run NPM Tests").
2. Directory = an existing empty folder (also tried a nonexistent path, and a folder registered as a QA project via "Or pick a registered project").
3. Click Run.

## Expected

The block runs in that folder and a result is shown.

## Actual

Error toast: `Failed to create script execution: Execution Error: error returned from database: (code: 787) FOREIGN KEY constraint failed`. Same in all three cases, so the cause is not the folder. Suspect (not confirmed): `automation.run` creates a `script_execution` row whose `script_id` / block reference points at a row that does not exist (built-in blocks vs scripts table). Error text comes from `script_execution_repository.rs:71`; frontend `QuickRunPage.svelte` `handleRun`.

## Evidence

`.claude/qa/shots/QA-071-before-quickrun-unregistered-folder.png`, `sweep-automation-quickrun-run-bad-folder.png`, `sweep-automation-quickrun-registered-project.png`; console `[ScriptExecutionService] Failed to execute script`.

## Repro

`node .claude/qa/scripts/QA-071.mjs`

## Fix

Root cause: `script_executions.block_id` has an FK to `blocks`, but built-in blocks (e.g. `test-npm`) live in frontend code, not the table, so every insert referencing one hit FK 787. Fix in `ScriptExecutionRepository::create` (all callers route through): keep `block_id` only if the block row exists, else store NULL (ad-hoc). Rust change, needs rebuild.

## Verification

<!-- qa-tester -->

## Verification (2026-11-06)

After a Rust rebuild, `QA-071.mjs` no longer hits code 787. Quick Run now executes the
queued block for real: against a folder with no package.json the run reaches npm and
reports its ENOENT, which is the correct failure for that folder.

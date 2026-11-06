---
id: QA-073
title: Action Runs lists "Release: ship-dry" as running ("Duration: In progress") three days after it started
status: verified
severity: minor
kind: bug
area: pipeline-runs
mode: tauri
found: 2026-11-06
---

## Use case

Read global run history to see what is active and what failed.

## Steps

1. `/pipeline-runs`, look at the run "Release: ship-dry" (pocket_portal, 03/11/2026 14:26:32).

## Expected

Runs orphaned by an app restart/crash are marked failed/cancelled at startup; the status reflects reality.

## Actual

Still `running`, "Duration: In progress", with a spinner, on 2026-11-06 (app was restarted since). Cannot prove no process is alive from the UI, but a 3 day dry-run is not plausible. Other rows (cancelled "Self build (daemon dogfood)") show the cancel path exists. Related: QA-003 (run loads forever).

## Evidence

`.claude/qa/shots/QA-073-before-stale-running-run.png`

## Repro

manual - open `/pipeline-runs`.

## Fix

At startup (`src-tauri/src/lib.rs` setup, after migrations, before the UI reads) executions the previous session left unfinished are settled as `failed`, `finished_at` set, error "Interrupted by app restart".
- Pipelines: new `ExecutionService::reconcile_interrupted` (pending/queued/running rows, plus their running steps -> failed). `failed` already renders in `executionDisplay.ts`; no frontend change.
- Scripts: existing `sync_running_executions` now also covers `pending` rows and runs synchronously in setup (it was a spawned task, which could race with new runs); no-pid reason reworded to "Interrupted by app restart".
- Needs an app rebuild/restart to take effect.

## Verification

<!-- qa-tester -->

## Verification (2026-11-06)

After a rebuild and restart, `/pipeline-runs` shows zero `running` rows, zero "In progress"
durations and no spinners. "Release: ship-dry" (pocket_portal, 03/11) now reads `failed`
with "Interrupted by app restart" and a real duration. Script executions left `pending` are
covered by the same startup pass. Evidence: `shots/QA-073-after-interrupted-runs-reconciled.png`.

Note: a reconciled row's duration is start → reconcile time (ship-dry reads 4571m), which is
truthful but reads oddly; a stopped-at-restart duration would read better. Cosmetic, not filed.

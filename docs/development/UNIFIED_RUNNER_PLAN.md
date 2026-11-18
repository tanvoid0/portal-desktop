# Unified runner — plan and status

Goal: an Action (script) run is a one-step pipeline execution on the pipeline
runner, so Actions, Pipelines and Releases share one store, one runner, one
event stream and one progress model.

PCORE = `src-tauri/portal-core/src`.
ES = `PCORE/domains/projects/pipelines/services/execution_service.rs`.
SES = `PCORE/domains/scripts/services/script_execution_service.rs`.

## Status

| step | what | state |
|---|---|---|
| Phase 1 | step progress, ETA from history, `::portal-progress` protocol, bar | done (06b16b40) |
| Phase 2 | one Action Runs list (`get_recent_runs` / `get_running_runs`) | done (837a9ffa) |
| C1 | batched log persistence, process-group kill (TERM then KILL on unix) | done |
| C2 | `pipeline_executions` rebuilt: nullable ids, `kind`, action columns | done |
| C3 | `script_executions` history copied as `kind='action'`, same ids | done |
| C4 | SES becomes a facade over `ExecutionService`; catch-up migration | **todo** |

C1-C3 verified on a copy of the real DB: 30 pipeline rows kept, 32 script rows
became 32 action rows with the same ids, no FK violations, every step blob
deserializes (`cargo test real_db_copy_migrates -- --ignored`, env
`PORTAL_MIGRATION_DB_COPY=<copy of portal_desktop.db>`). Never point it at the
live DB: `%APPDATA%/com.tanvoid0.portal-desktop/portal_desktop.db`.

## C4 — what to do

1. **Catch-up migration `m20261119_000052`.** 000051's copy is a snapshot and
   SES keeps writing `script_executions` until this step lands. Call
   `copy_script_history(db, true)` (INSERT OR REPLACE) from a new migration so
   rows written between 000051 and the cutover are not lost. Do not drop
   `script_executions`; a later migration does that once C4 has shipped.
2. **Extract** ES `execute_pipeline`'s run body into
   `ExecutionService::execute_steps(spec)`. Spec: optional `pipeline_id` and
   `project_id`, cwd, steps, variables, `triggered_by`, action metadata
   (`kind`, `command`, `working_directory`, `block_id`, `workflow_*`, `step_id`,
   `workflow_plan_json`), `timeout: Option`. `execute_pipeline` calls it and
   stops hardcoding `"user"`.
   `kind='action'` deltas: no deadline (dev-server actions run forever), skip
   `normalize_package_manager_command` (QA-129), no `[stderr] ` prefix.
3. **SES keeps its type name and public methods** but delegates to a shared
   `Arc<ExecutionService>`; delete its spawn, `pump`, `kill_tree`, buffers and
   `sync_running_executions` body (SES ~:238-479, :648-723). Callers in
   `relocation/usage.rs` and `mover.rs` compile unchanged.
   Mapping: `execute_script` -> one-step `execute_steps`; `get_execution` -> row
   to `ScriptExecutionInfo` (a `cancelled` row with no error reports "Stopped by
   user", crew prints that string); `get_live_output` -> the step's `logs`, each
   element ending in `\n` (crew joins with `''`); `record_interactive` -> insert
   a finished action row; `delete_execution` -> `ExecutionRepository::delete`;
   by-block / by-workflow-run / running / recent -> queries on the new columns;
   `sync_script_executions` -> `Ok(())` (`reconcile_interrupted` at `lib.rs`
   ~:312 covers startup).
4. **Wiring.** `ScriptExecutionState` (`src-tauri/src/domains/scripts/commands.rs`
   ~:12-31) and `lib.rs` ~:322-328 must be built from the managed
   `ExecutionService`; a second instance has its own `running` map and Stop finds
   nothing. `execute_script` (~:51) gains `events: State<Events>`.
5. **Collapse the Phase 2 merge.** Remove the `kind='pipeline'` filter in
   `ExecutionRepository::get_all` and the scripts branch in `runs.rs` together;
   `recent_runs` / `running_runs` become one query. Emit `kind` and the action
   fields from `execution_to_json`.
6. **Frontend.** `app-shell.svelte` ~:255-266 must ignore `kind==='action'`, or
   every action and workflow step toasts "Pipeline run finished".
   `ActionRunDetail` may take progress from the event. Terminal
   (`ScriptRunPanel.svelte`), Automation (`automationExecutionService.ts`) and
   the workflow loop (`actionsService.ts`) need no change; they go through SES.
7. Frontend-orchestrated workflows stay one execution per step, grouped by
   `workflow_run_id`. GitHub, n8n and interactive steps spawn nothing, so they
   cannot be pipeline steps.

## Must not break (names, casing, shapes; additive JSON only)

- Tauri/daemon commands: `execute_script` (`{params:{command, workingDirectory,
  env, triggeredBy, blockId, parameters, workflowRunId, workflowId, stepId,
  workflowPlan}}` -> id string), `record_interactive_execution`,
  `get_script_execution` (`{execution_id}` -> camelCase `ScriptExecutionInfo`),
  `get_script_execution_live_output` (-> `string[]`), `cancel_script_execution`,
  `get_script_executions_by_block`, `get_script_executions_by_workflow_run`,
  `get_running_script_executions`, `get_recent_script_executions`,
  `sync_script_executions`, `delete_script_execution`.
- Pipeline/release: `execute_pipeline`, `get_pipeline_execution(s)`,
  `get_project_pipeline_executions`, `get_all_pipeline_executions`,
  `cancel_pipeline_execution`, `release_*`. `release_ship` stays unreachable over
  the daemon (`rpc.rs`).
- Capabilities: `execute_script` / `execute_pipeline` = `terminal.exec`; reads =
  `actions` / `pipelines`.
- Events: `pipeline-execution-update` (payload has `progress`, add `kind`) and
  `pipeline-step-log` `{executionId, stepId, line, stream}`.
- Status vocabulary: pending, queued, running, success, failed, cancelled.
- The crew plugin talks to the INSTALLED release daemon on :18420, so the facade
  must be byte-compatible on the fields above.

## Risks

- Stop semantics: today a clean exit after Stop stays `success` (SES ~:434);
  the pipeline `cancel_execution` writes `cancelled` immediately (ES ~:911).
  Decide in review.
- `\r`-only progress output no longer streams live (reader is line-based);
  upgrade path: split on `\r` too.
- Windows orphans that outlive their parent survive Stop; a Job object is the
  upgrade.
- Event payload size with big action logs: keep the 1 s throttle; strip `logs`
  from progress-tick emits if it hurts.

## Check that must exist before C4 is done

`script_facade_runs_on_pipeline_runner` in SES tests: on a temp DB (1) insert a
legacy `script_executions` row, run migrations, assert `get_execution(<same id>)`
returns its output; (2) `execute_script("echo hi")`, poll until finished, assert
status `success`, output contains `hi`, `get_live_output` joined with `''`
contains `hi\n`, and a `pipeline_executions` row with `kind='action'` exists.

Gates: `cargo test` in `src-tauri/portal-core`; `cargo check && cargo test` in
`src-tauri`; `pnpm check`; then a dev instance (`node scripts/dev-instance.mjs 2`)
against a copy of the release DB to see old runs listed. Review the adapter
contract (`crew:reviewer`) before committing.

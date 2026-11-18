# Unified runner — plan and status

Goal: an Action (script) run is a one-step pipeline execution on the pipeline
runner, so Actions, Pipelines and Releases share one store, one runner, one
event stream and one progress model.

PCORE = `src-tauri/portal-core/src`.
ES = `PCORE/domains/projects/pipelines/services/execution_service.rs`.
SES = `PCORE/domains/scripts/services/script_execution_service.rs`.

## Status

| step    | what                                                                 | state                 |
| ------- | -------------------------------------------------------------------- | --------------------- |
| Phase 1 | step progress, ETA from history, `::portal-progress` protocol, bar   | done (06b16b40)       |
| Phase 2 | one Action Runs list (`get_recent_runs` / `get_running_runs`)        | done (837a9ffa)       |
| C1      | batched log persistence, process-group kill (TERM then KILL on unix) | done                  |
| C2      | `pipeline_executions` rebuilt: nullable ids, `kind`, action columns  | done                  |
| C3      | `script_executions` history copied as `kind='action'`, same ids      | done                  |
| C4      | SES is a facade over `ExecutionService`; catch-up migration 000052   | done (see "As built") |

C1-C3 verified on a copy of the real DB: 30 pipeline rows kept, 32 script rows
became 32 action rows with the same ids, no FK violations, every step blob
deserializes (`cargo test real_db_copy_migrates -- --ignored`, env
`PORTAL_MIGRATION_DB_COPY=<copy of portal_desktop.db>`). Never point it at the
live DB: `%APPDATA%/com.tanvoid0.portal-desktop/portal_desktop.db`.

## C4 — as built

Landed as two commits: the runner half (`execute_steps`, repo methods,
migration 000052) and the facade after it. **Release them together.** 000052
copies script history once, so a build that has it but still runs the old SES
would keep writing `script_executions` rows nobody copies.

- `RunSpec` has no `kind`: `action: Some(ActionMeta)` is an action run. It has
  `env` (merged onto each step's env, no `$` expansion) and `expected` (ETA
  history; `execute_pipeline` fills it from the pipeline's runs, `execute_script`
  from the action's own, see "ETA for action runs" below).
- Action deltas: no deadline, no package-manager rewrite, no `[stderr] ` prefix,
  no `$ cmd` / `cwd:` header or package.json warnings (a step's logs are the
  process output, so `ScriptExecutionInfo.output` is what crew prints),
  `error` = `action_failure_message` ("Exit code N" + last 8 lines), pid
  written, `move_lock` checked in `execute_steps`.
- SES: `new(db, exec)`, `execute_script(req, events)`, and `get_live_output` is
  `async` (reads the step logs, so up to ~250 ms behind). `output` is "" while a
  run is active, as before. `queued` is now a possible script status; crew and
  the UI treat every non-terminal status as running.
- `ExecutionRepository::get_all` stays pipeline-only (the plan said to drop the
  filter): `get_all_pipeline_executions` and the Pipelines pages must not start
  listing actions. `recent_runs` is `get_recent(None, n)` and `running_runs` is
  `get_non_terminal()`, both over every kind.
- `running_under` is pipelines-only, `running_actions_under` the action half,
  so a relocation reports one blocker per run.
- `block_id` is stored as given: `pipeline_executions` has no FK on it, the old
  table nulled unknown block ids.

- ETA for action runs: the key is `block_id` when the run has one, else equal
  `(command, working_directory)`. `ExecutionRepository::get_action_history`
  returns the last 20 successful `kind='action'` rows for it, newest first;
  `ExecutionService::action_expected` maps them to `HistoryRun`s and
  `expected_durations` takes the median per step id.
  `ScriptExecutionService::execute_script` passes the result as `RunSpec.expected`
  (step id `request.step_id` or "run"), so a first run has no estimate and later
  runs start from their own median. Best effort: a query or parse error means no
  ETA, never a failed run. Rows copied from `script_executions` count too: 000051
  writes the step `duration` when both timestamps parse, and `action_expected`
  falls back to `finished_at - started_at` when it is missing. The query is not
  indexed on `block_id` / `command`; fine at today's table size.
- `queued` renders like `pending` in `executionStatus.ts` (it fell to the default
  clock icon and muted colour), and `runs.rs` has a test that a cancelled action
  row with no error reads "Stopped by user" while a cancelled pipeline row does not.

Not done:

- **Dropping `script_executions`: deferred by the user (2026-11-18, "keep it for
  now"); revisit after C4 has shipped and run once.** It is irreversible against a
  real DB and the table is the rollback copy the user was told stays. When it is
  time: a migration `DROP TABLE IF EXISTS
script_executions` (its indexes go with it), no re-copy and no "everything was
  copied" gate (a row missing from `pipeline_executions` is one the user deleted
  through the facade after 000052; a REPLACE copy would resurrect it), `down` a
  no-op. Ship it with or after the C4 release: an old build cannot run against a
  migrated DB anyway (sea-orm refuses unknown applied migrations). Then delete the
  legacy entity and `ScriptExecutionRepository` (keep `WorkflowLink`) and replace
  the facade test's raw legacy-row insert with a migration-level one.
- `\r` streaming; a Windows Job object.

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

- Stop semantics, **decided: Stop wins.** `cancel_execution` writes `cancelled`
  at once and a `cancelled` row is final (`update_status` never overwrites it,
  `finalize_step` leaves a cancelled step alone), so a process that exits
  cleanly inside the race window still reads `cancelled`. The old script runner
  kept `success` there. Cost: the last <=250 ms of log lines can land after
  the status flips.
- Startup: `reconcile_interrupted` fails every non-terminal action row with
  "Interrupted by app restart". The old sync kept a `running` row whose pid was
  still alive; an orphan left by a crashed app is now listed as failed while it
  may still run (Stop could not kill it before either).
- Log growth: `modify_steps` rewrites the whole step blob on every log batch, so
  a dev-server action that prints for hours is O(n^2). Upgrade path: cap or
  ring-buffer action logs.
- A missing working directory is rejected up front (an `Err`, no row). Any other
  spawn failure is a failed run with the reason in `error`, which crew's
  `run_status` does not print (it shows output only).
- `record_finished_action` (terminal runs, no-spawn workflow steps) emits no
  `pipeline-execution-update`; the Action Runs list refreshes through its own
  callers (`onChanged`, `onExecuted`, `loadHistory`).
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

# Design brief — AI Pipeline-Doctor for Portal Desktop

Start a fresh session with this file. Big new feature the user asked for; architect-led
(crew:architect) planning first, then staged build. This is the product version of what the
2026-11-11 trial did by hand (see `.claude/handoff.md` and `.claude/trials/portal-trial-expo-todo.md`).

## The vision (user's words)

Portal's AI should be smart enough to:

1. **Detect pipeline/action failures** automatically.
2. **Spawn agents to investigate and fix** — either the app's code, or the pipeline itself.
3. **Fix the pipeline** when the pipeline (not the app) is wrong — including Portal Desktop's
   own pipeline-generation logic.
4. **Build/customise pipelines from app requirements** — not just static per-stack defaults.

## Where this plugs into the existing code (found during the trial)

- **Action/pipeline catalog:** `src/lib/domains/actions/defaults.ts`
  (`buildDefaultActions` / `buildDefaultWorkflows` — now Expo-aware after QA-128),
  `src/lib/domains/actions/{profile,types,resolver}.ts`, and the project profile
  (`ProjectAutomationProfile`). Also the older builder under
  `src/lib/domains/projects/pipelines/` (`PipelineBuilder.svelte`, `executionService.ts`,
  `executionStore.ts`).
- **The runner / run history:** actions execute commands and (should) record Actions-runs;
  this is where failure signals come from. **Blocked by QA-130** (today a Run produces no
  visible output / no run-history row) and **QA-129** (pnpm preflight/lockfile-policy and
  non-TTY make runs fail before the real command; needs per-project PM override + `CI=true`).
- **Agent/coder infra already exists:** the built-in coder (Claude-CLI engine) and the Agents
  domain already spawn agents that edit files and run commands, with an approval/Agent-changes
  gate. That IS the "spawn agents to fix" primitive — reuse it, don't build a new runtime.
  (This session used exactly that pattern: 3 `coder` agents fixed QA-128/131/133/134/135.)

## Prerequisites (Stage 0 — must land first)

- **Fix the E0277 build break** (`error[E0277]: Caller: serde::de::DeserializeOwned`, from
  `795e3e88` security gating; `src-tauri/src/rpc.rs`, `rpc-macro/src/lib.rs`,
  `src-tauri/src/domains/{coder,agents}/commands.rs`). Nothing runs until the backend builds.
- **QA-130** — structured run results: `{ command, exitCode, stdout/stderr, failing step }`,
  persisted and visible. The doctor needs trustworthy failure signals.
- **QA-129** — reliable non-interactive runner (PM override + `CI=true`).

## Staged design

- **Stage 1 — Detect:** on a non-zero action/workflow run, emit a structured _failure event_
  (command, exit code, stderr tail, step, project profile). Persist + surface.
- **Stage 2 — Diagnose & fix (agent):** spawn a coder agent with the failure event + repo
  context. It classifies: **app-code failure** (fix code → re-run) vs **pipeline failure**
  (wrong/missing command — e.g. the Expo `pnpm build`/`pnpm lint` gap this trial hit → fix the
  action's command). Apply, re-run to confirm green. Bounded retries; respect permission mode;
  diff review before apply.
- **Stage 3 — Fix the pipeline itself:** when the fault is the pipeline, edit the project's
  pipeline/action definition; when it's a Portal _default_ (like the Expo mis-mapping), propose
  a change to `defaults.ts` as a gated suggestion/PR — i.e. the doctor can repair both a
  project's pipelines and Portal's own generation logic.
- **Stage 4 — Requirement-driven generation:** given a detected stack + an NL requirement
  ("add an EAS release and a lint/typecheck gate"), an agent generates/customises the action
  catalog + CI workflow beyond the static defaults, using stack detection + the coder to write
  any needed config/scripts.

## Guardrails

Bounded retry loops (never thrash on a flaky failure); permission-mode respected;
human-in-loop via the existing Agent-changes/approval gate; never auto-commit/push; **no
`Co-Authored-By: Claude` / "Generated with Claude Code"** anywhere (CLAUDE.md). Multi-subsystem
(actions + agents/coder + project detection + Rust runner) → get an architecture plan before
building.

## Status of the related QA fixes (done this session, gated green)

QA-128/131/133/134/135 are implemented and pass `pnpm check` (0 errors) + `pnpm lint` (0
errors). Uncommitted. QA-129/130 + E0277 are the Stage-0 prerequisites above.

## Build status — foundation slice landed (2026-11-11, session 2)

Architect-led plan executed (crew:architect). **Stage 0 + Stage 1 done**, gated green
(`pnpm check` 0 errors, `pnpm lint` clean, `cargo check` clean). All uncommitted.

- **E0277 is already fixed** — `cargo check` is clean on committed `main`. The handoff's
  "NOT STARTED" was stale. No Rust build work needed.
- **Stage 0 / QA-130 (runs visible):** `onStepStart` callback surfaces the run row + live
  output on spawn (not on finish); 1s `$effect` poll streams `getLiveOutput`/`getExecution`;
  Rust runner `stdin(Stdio::null())` kills the pnpm-prompt hang. → `status: fixed`.
- **Stage 0 / QA-129 (reliable runner):** `CI=true` injected for Action runs via a new
  additive `env` field on `execute_script` (TS `ExecuteScriptParams` → Rust
  `ExecuteScriptRequest`); stdin null. PM-override **UI deferred** (edit page already carries
  `package_manager_ids`; open Q: does QA-131 resync clobber a user-set value?). → `status:
open` (partial — deadlock fixed).
- **Stage 1 / Detect:** `src/lib/domains/actions/failure.ts` — `toFailure(result)` +
  `classifyFailure(text, exitCode)` → `pipeline | environment | unknown` + hint. Attached as
  `failure?` on `ActionRunResult` in `actionsService`. Panel shows a "Run failed" card
  (kind + hint + **disabled** "Diagnose & fix"). Test: `failure.test.ts` (8 passing, vitest).

### Decisions worth keeping

- **No new DB table, no migration, no new agent runtime** for Stages 0–3. `script_executions`
  IS the failure record; the coder claude-cli engine IS the fix runtime.
- `CI=true` is scoped to Action runs (sent from `actionsService`), not hard-coded globally in
  Rust, so script-block runs keep their behaviour.
- Output ordering fix (interleave stdout/stderr in persisted `combined_output`) was **skipped**
  — errors already land at the tail (stderr last), which is what the classifier reads. Add
  only if a consumer needs true interleaving.

### Stage 2 (diagnose/fix agent) — DONE (2026-11-11, session 2), gated green

Click "Diagnose & fix" on a failed run → spawns a coder thread per attempt
(`runCoderTask` in `actions/doctor.ts`) on **the user's configured coder backend**
(`coderSession.selectedBackendProvider` — Ollama/local = free, or whatever they've set; NOT
forced `claude-cli`), parses `DOCTOR_VERDICT: app|pipeline|environment|flaky`
from `final_text`, re-runs the action to confirm green, bounded ≤2 attempts/(project,action)/
session (`actions/state/doctor.svelte.ts`). Reuses the existing coder approval gate
(review-after-write) — the panel links to the thread ("Review changes in Coder" →
`coderSession.selectThread` + `goto('/ai/coder')`). Guardrails verified: prompt forbids git
commit/push, never calls `setPermissionMode` (global), never auto-approves, environment failures
never spawn an agent. On a green fix the verify result replaces `lastResult` so the failure card
clears + a success toast fires. Tests: `actions/doctor.test.ts` (7, vitest). **Not committed.**
Ceilings: doctor state is in-memory (failure↔thread link lost on restart); `runCoderTask` has no
timeout (user stops a hung thread in Coder); `triggeredBy:'doctor'` plumbing + auto-spawn
skipped by design.

### Stage 3 (pipeline self-repair) — DONE (2026-11-11, session 3), gated green

`pipeline` verdict → the agent writes/overrides the action in `.portal/pipeline.yml` (same id
beats the default). `validatePortalPipelineFile` now really validates (non-empty `run`; workflow
steps reference a known action = file actions + reserved default ids) and rejects bad files. New
`reloadAndValidatePortalPipelineFile` surfaces an invalid agent edit (the old loader swallowed
parse errors → re-ran the wrong default → looped). `buildDiagnosePrompt(failure, action?)` adds the
file schema + current command on a pipeline-kind failure. `diagnose` reloads+validates then
verify-runs; on green, when the wrong command was a Portal _default_ (`source==='local'`), it
attaches a `PipelineFixReport` and the panel shows a gated amber "Portal default needs a fix" card
(profile fingerprint + default vs green command + Copy report) — `defaults.ts` is never auto-edited.
Gates: vitest 26 passed, `pnpm check` 0, `pnpm lint` 0. **Not committed.**
Skipped (ceiling): the optional "Open fix in Coder" button (needs Portal-repo-is-a-project lookup).

- **Stage 4 (requirement-driven gen):** NL textarea → `buildGeneratePrompt` → coder writes
  `.portal/pipeline.yml` + scripts; validate + `preview()` dry-check.

Full file-level plan: ask crew:architect output from session 2, or re-derive from this brief.

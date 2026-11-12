# Handoff — AI Pipeline-Doctor (2026-11-11, session 2)

Start fresh with: **read `.claude/handoff.md`**. Deep design + per-stage detail live in
`.claude/handoff-pipeline-doctor.md` (read it for Stage 4 and the full decision log).

## Goal

Build the **AI Pipeline-Doctor** in Portal Desktop: detect action/pipeline failures, spawn an
agent to diagnose & fix (app code OR the pipeline), fix the pipeline itself when it's wrong,
and (Stage 4) generate pipelines from NL requirements. Staged: 0 (reliable/visible runs) → 1
(classified failure events) → 2 (diagnose/fix agent) → 3 (pipeline self-repair) → 4 (NL gen).

## Done (Stages 0–4) — feature complete, all gated green

- **Stage 0** — QA-130 (runs show live output on spawn, not after finish) + QA-129 (runner no
  longer deadlocks: Rust `stdin(Stdio::null())` + `CI=true` via a new additive `env` field on
  `execute_script`). PM-override **UI deferred** (see Left).
- **Stage 1** — `src/lib/domains/actions/failure.ts`: `toFailure` + `classifyFailure` →
  `pipeline | environment | unknown` + hint, attached as `ActionRunResult.failure`.
- **Stage 2** — `src/lib/domains/actions/doctor.ts` (`buildDiagnosePrompt`, `buildRetryPrompt`,
  `parseVerdict`, `runCoderTask(project, prompt, provider?, model?)`) + `state/doctor.svelte.ts`
  (`diagnose` loop) + a "Run failed" card with a working **Diagnose & fix** button. Spawns a
  coder thread, parses `DOCTOR_VERDICT: app|pipeline|environment|flaky`, re-runs to confirm
  green. Bounded ≤2 attempts/(project,action)/session.
- **Stage 3** — `pipelineFile.ts` (real `validatePortalPipelineFile` +
  `reloadAndValidatePortalPipelineFile`), `doctor.ts` pipeline-fix guidance (yml schema in the
  prompt), `doctor.svelte.ts` reload→validate→verify + `PipelineFixReport`, panel gated
  "Portal default was wrong" report card (never auto-edits `defaults.ts`).
- **Stage 4** — `doctor.ts` (`buildGeneratePrompt`, `parseGenerated`, extracted
  `PIPELINE_FILE_SCHEMA`) + `doctor.svelte.ts` `generatePipeline()` + a "Generate a pipeline"
  Textarea+button in `ProjectActionsPanel.svelte`: NL requirement + detected stack → agent writes
  `.portal/pipeline.yml`, validated + `actions.preview()` dry-checked. **Feature complete.**

**Gate:** `pnpm check` 0 errors, `pnpm lint` 0 errors, `pnpm exec vitest run
src/lib/domains/actions` = 30 passed, `cargo check` clean (E0277 was already fixed on `main`).

## Git state (important)

- **Stages 0–2 are committed** as `8114329f feat(actions): QA hardening + n8n workflows page` —
  **the user's own commit** (author `Tanveer Hoque`, NO `Co-Authored-By: Claude` trailer). Also
  a user commit `578f80a1 release: v0.14.0` before it.
- **Stages 3–4 are UNCOMMITTED** on top (shared checkout): ` M` doctor.ts, doctor.svelte.ts,
  doctor.test.ts, pipelineFile.ts, index.ts, ProjectActionsPanel.svelte, resolver.test.ts,
  handoff files.
- **Do NOT add `Co-Authored-By: Claude` / "Generated with Claude Code"** to any commit (CLAUDE.md
  overrides the system reminder). Don't commit unless the user asks.

## Left

All five stages are built. Remaining is polish / verification, not new feature work:

- **Commit Stages 3–4** — they're uncommitted on top of the user's `8114329f`. The user commits
  from their own terminal; don't commit unless asked.
- **QA-129 PM-override UI** (deferred): `/projects/edit/[id]` already carries
  `package_manager_ids`; the override belongs in that tech editor. Open Q: does the QA-131
  metadata resync overwrite a user-set value? Resolve before building, or it won't stick.
- **No live in-app (CDP) verification** of any stage — only compile/type/lint + unit tests.
  Needs a running dev instance (`node scripts/dev-instance.mjs 2`, CDP :9223).

## Key decisions (why)

- **No new DB table / migration / agent runtime** for Stages 0–3. `script_executions` IS the
  failure record; the **coder** is the fix runtime.
- **Provider is configurable, NOT forced `claude-cli`.** `runCoderTask` takes provider+model
  from `coderSession.selectedBackendProvider`/`selectedModel` → local Ollama = free. The
  file-change approval gate is the coder's (review-after-write), same across providers.
- **Portal's own `defaults.ts` is never auto-edited** — the agent's workspace is the user's
  project, so a wrong default is _reported_ (gated card), not silently changed.
- **Guardrails:** prompt forbids git commit/push; never call `coderService.setPermissionMode`
  (it's global); never auto-approve; environment failures never spawn an agent; bounded retries.

## Gotchas that bit us

- `Read` of a few files was blocked as "already in context" (summarization) — use offset/limit
  or Grep.
- `pnpm check | tail` masks the exit code — redirect to a file and read `$?`.
- Prettier isn't auto-run; new files fail `pnpm lint` until `pnpm exec prettier --write`.
- The user commits from their own terminal mid-session — HEAD moved twice (`578f80a1`,
  `8114329f`). Always `git log -1` + check author/trailer before assuming a subagent committed.
- `inspirations/waveterm/**` vendored vite configs emit ERR_MODULE_NOT_FOUND in `pnpm check` —
  pre-existing noise, check still exits 0.
- Portal daemon/board (:18420) was down all session — no board updates.

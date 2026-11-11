---
id: QA-129
title: "Projects > Actions: the package manager is auto-chosen (pnpm) with no per-project override, and every action dies at pnpm's pre-install preflight (lockfile-policy rejection; non-TTY also needs CI=true) before the real command runs"
status: open
severity: major
kind: bug
area: projects/actions
mode: tauri
found: 2026-11-11
---

## Use case

A developer runs any Action (Test/Build/Lint) on a project Portal detected as pnpm. They
expect the action's command to run.

## Steps

1. Project "Expo Todo" (id 201), detected PM = pnpm, so all actions are `pnpm …`.
2. Run Test / Build / Lint from Projects > Actions (or the same commands in a non-TTY shell).

## Expected

The action runs, or Portal clearly reports the preflight failure and offers an alternative
(e.g. switch to npm).

## Actual

- pnpm auto-runs a deps-status check (`pnpm install`) before every script. On this machine
  that fails:
  > "The lockfile contains entries that the active policies reject … run `pnpm clean
  > --lockfile` and then `pnpm install`."
  So **Test, Build and Lint all fail at the pnpm gate before the real command runs** (all
  exit 1). In a non-TTY runner pnpm additionally refuses without `CI=true` ("set the CI
  environment variable to true, or confirmModulesPurge to false").
- There is **no UI to pick/override the package manager per project**; detection's choice is
  final, so the whole Actions surface is dead-on-arrival here.
- The coder only succeeded earlier because it used `npm test` / `npx tsc`, bypassing pnpm.

## Why it matters

A single machine-level PM policy makes Portal's entire CI/CD surface unusable for a project,
with no in-app escape hatch. The failure is also opaque (see QA-130 — no run output).

## Suggested fix

- Let the user override the package manager per project (and persist it).
- Run non-interactive actions with `CI=true` (or `--frozen-lockfile=false` / the pnpm
  equivalent) so the runner doesn't deadlock on prompts.
- When the detected PM's preflight fails, surface the error and offer to fall back to npm.

## Evidence

- Ground-truth runs in `D:\projects\trials\expo-todo`: `pnpm run test|build|lint` → lockfile
  policy rejection (exit 1); `CI=true` did not help (policy, not just the TTY prompt).

## Fix (partial)

The **dead-on-arrival deadlock is fixed**; the per-project PM picker is deferred.

- **Non-interactive runs.** Action runs now pass `CI=true` to the spawned process
  (`actionsService` → `execute_script` `env` passthrough), and the Rust runner sets
  `stdin(Stdio::null())` so pnpm/npm can never block on an unanswerable prompt — a failing
  preflight now exits fast and records a real failure row instead of hanging silently.
  (`src/lib/domains/actions/actionsService.ts`,
  `src/lib/domains/scripts/services/scriptExecutionService.ts`,
  `src-tauri/src/domains/scripts/commands.rs`,
  `src-tauri/portal-core/src/domains/scripts/services/script_execution_service.rs`)
- **Surfaced, not silent.** The lockfile-policy rejection is classified `environment` by the
  Stage-1 failure classifier and shown with a hint to switch the package manager to npm
  (`src/lib/domains/actions/failure.ts`).

### Deferred

- **Per-project PM override UI.** `/projects/edit/[id]` already carries `package_manager_ids`
  through its update form, so the override belongs in that tech-stack editor, not a second
  control in the Actions panel. Open question first (flagged by the architecture pass): does
  the QA-131 metadata resync overwrite a user-set `package_manager_ids`? If so, resync must
  skip a user-chosen value or the override won't stick. Tracked as a follow-up.

`CI=true` evidence caveat holds: it does **not** clear pnpm's lockfile-policy rejection (that's
a policy, not a TTY prompt) — the real escape hatch is switching to npm, which the hint now
points at.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
Code + gates green (`pnpm check`/`pnpm lint`/`cargo check`). Live verify pending: run an
Action on a pnpm project detected to hit the lockfile policy → expect a fast `environment`
failure with the switch-PM hint, not a hang.

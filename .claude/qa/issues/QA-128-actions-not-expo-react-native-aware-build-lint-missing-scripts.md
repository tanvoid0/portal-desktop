---
id: QA-128
title: "Projects > Actions: auto-generated CI/CD actions are not Expo/React-Native aware — Build maps to `pnpm build` and Lint to `pnpm lint`, neither of which exists in an Expo app, so the default CI workflow (Install→Test→Build) is red out of the box"
status: fixed
severity: major
kind: bug
area: projects/actions
mode: tauri
found: 2026-11-11
---

## Use case

A developer creates a project for a fresh Expo (React Native) app and opens
Projects > Actions. They expect the smart-default actions and the generated CI
workflow to run against that app's real scripts.

## Steps

1. Scaffold an Expo app (`create-expo-app --template blank-typescript`), register
   it as a Portal project (here: "Expo Todo", id 201, `D:\projects\trials\expo-todo`).
2. Open `/projects/201/pipelines` (Actions tab). Read the generated actions and the
   CI workflow.

## Expected

Build/Lint/Test resolve to commands that exist for an Expo app
(`expo export` / `eas build`, `expo lint`, the jest `test` script), and the CI
workflow is green on a clean scaffold.

## Actual

- The project is detected as **"React"** only (Details > "What it's built with:
  React, TypeScript, pnpm") — Expo / React Native is not distinguished.
- Generated actions (from `buildDefaultActions`, `src/lib/domains/actions/defaults.ts`):
  - `Build → pnpm build` — **no `build` script** in Expo (`package.json` scripts are
    `start/android/ios/web` + the coder-added `test`). `npm run build` → "Missing
    script: build" (exit 1), confirmed.
  - `Lint → pnpm lint` — **no `lint` script** either (exit 1).
  - `Dev → pnpm run start` is correct (resolved from the detected start command).
  - `Test → pnpm run test` is correct once a test script exists.
- The default **CI workflow** (`buildDefaultWorkflows`: Install → Test → Build) therefore
  fails at its Build step on any clean Expo project.
- There is **no typecheck action** at all (`tsc --noEmit`), which is the natural "build"
  gate for a TypeScript RN app.

## Why it matters

The headline CI/CD value — "smart defaults from project metadata" — produces a broken
pipeline for one of the most common mobile stacks. A developer's first run is red for a
reason unrelated to their code.

## Suggested fix

Detect Expo/React-Native (presence of `expo` dep / `app.json` / `expo` in package.json)
and map: build → `expo export` (or `eas build` when EAS is configured), lint → `expo lint`,
and add a `Typecheck → tsc --noEmit` default action for TS projects. Fall back to the
generic script only when the script actually exists in package.json.

## Evidence

- Live Actions catalog over CDP `:9223`; project 201.
- Ground truth: `npm run build` / `npm run lint` → Missing script (exit 1).

## Fix

The reported symptom (a default CI workflow that is red because `pnpm build` / `pnpm lint` do
not exist) was fixed by two commits that had already landed; no code changed for this close-out.

- `8114329f`: Expo awareness in `src/lib/domains/actions/defaults.ts`. `isExpoLike` (`:58-65`)
  switches Build to `expo export` and Lint to `expo lint`, run through the package manager
  (`pmExec`, `:171-206`), and a new `Typecheck` action runs `tsc --noEmit` (`isTypeScript`
  `:67-70`, action `:208-218`).
- `865e06f2`: Build/Lint/Test/Dev are offered only when package.json defines the script
  (`defaults.ts:137-142`, fed by `resolver.ts:151` `loadPackageScripts`). A clean Expo scaffold
  therefore gets no `pnpm build` / `pnpm lint` rows, and `buildDefaultWorkflows` (CI needs
  install + test + build) does not emit a CI workflow that would fail at Build.

Honest residual: the Expo-specific branch is dead for a registered app. `isExpoLike` needs
"expo" in the build/start command or an Expo / React Native framework name, but
`stack.rs:44-80` (`DEP_FRAMEWORKS`) never emits Expo or React Native (the project is detected as
React) and the start command comes out as `pnpm run start` (`stack.rs:276`). So `expo export`,
`expo lint` and the Expo Typecheck do not show for a registered Expo app today; what removes the
red CI is the script gate in `865e06f2`.

Follow-up, not implemented here: add `(&["package.json"], "\"expo\"", "Expo")` to
`DEP_FRAMEWORKS` in `stack.rs`, with a unit test in its `#[cfg(test)]` module. Names must match
`frameworks` table rows (`store_stack` looks them up by name, `project_service.rs:463`), so check
an `Expo` row exists.

Docs only, nothing run. Not re-tested against a live Expo project.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

---
id: QA-193
title: Resolve all with AI opens the coder in an unrelated project, not the repos' shared folder
status: fixed
severity: major
kind: bug
area: projects (Sync all) -> ai/coder
mode: tauri
found: 2026-11-26
---

## Use case

Several repos need a hand after Sync all; open one coder session above them with the prompt pre-filled.

## Steps

1. Sync all, with 2+ stuck repos whose shared parent folder is not itself a Portal project
   (tested: two repos under `D:\projects\production\devstrail`).
2. Click "Resolve all with AI".

## Expected

Coder workspace is the shared parent (`D:\projects\production\devstrail`).

## Actual

The prompt is pre-filled correctly, but the workspace is the first project in the list
(`Devstrail - D:\projects\production\devstrail\devstrail`, branch develop). The same happened for fixture
paths under `D:\QA fixtures\`, which are unrelated to that project, and the previously selected workspace
is replaced. Sending would run the agent (mode Auto) in the wrong repository.

Cause, from reading the code: `openCoderSession` sets the workspace, then
`ProjectWorkspaceList.svelte` onMount calls `resolveProjectFromList` (`coderUi.svelte.ts`), which drops any
path that is not a project and falls back to `projects[0]`. The uncommitted changes in the tree on
2026-11-26 (`launch.ts`, `coderUi.svelte.ts`, `commonDir.ts`) do not change that fallback.

Same session: repos sharing only `D:\` also went through to the coder
(`QA-193-before-coder-drive-root.png`). The uncommitted `isTooBroad` guard looks aimed at that; not
verified.

## Evidence

`.claude/qa/shots/QA-193-before-coder-wrong-workspace.png`,
`.claude/qa/shots/sweep-projects-syncall-coder-prefill.png`

## Repro

`node .claude/qa/scripts/QA-193.mjs` (fake Sync all report answered in the page, no sync runs; never sends)

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

Root cause: resolveProjectFromList dropped any non-project path, including one just opened on purpose.

Files: coder/state/coderUi.svelte.ts (openWorkspace remembers the folder for this run), coder/utils/launch.ts, coder/components/CoderSessionCard.svelte. Drive root and home are refused by isTooBroad (projects/utils/commonDir.ts). After an app restart a non-project folder is still dropped (QA-134).

Checks: vitest coderUi and commonDir (8 pass).

Repro script not run (told not to touch the running app); cause confirmed by reading the code. Checks: pnpm check 0 errors; eslint 0 errors; shadcn check pass; prettier fails only on the untouched docs/status/RELEASE_NOTES.md.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

---
id: QA-134
title: "AI > Code: the coder's cached default workspace is a stale cross-platform path (`/media/tan/...` Linux mount) pre-selected on Windows, so a new session defaults to a non-existent directory"
status: verified
severity: minor
kind: bug
area: ai/coder
mode: tauri
found: 2026-11-11
---

## Use case

A developer opens AI > Code to start a new coder session and expects the default workspace to
be a valid directory on this machine.

## Steps

1. Open `/ai/coder` (New session). Observe the workspace selector's default value.

## Expected

The default workspace is a path that exists on this OS, or empty if none is valid.

## Actual

- The workspace selector defaults to `localStorage['portal-coder-last-workspace']`, which here
  is a **Linux path `/media/tan/Workspace/production/portal/portal_desktop`** on a Windows box
  (a leftover from a previous OS/session). `CoderProjectSelector` even pushes it into the
  options list as a selectable entry because it isn't among the detected projects.
- A developer who just hits send would start a session against a directory that doesn't exist.

## Why it matters

The pre-selected default silently points the agent at a non-existent cwd; at best it errors,
at worst it's confusing on first use.

## Suggested fix

Validate the cached last-workspace against the filesystem before defaulting to it; if it
doesn't exist, fall back to `projects[0].path` (as the code already intends) or leave it
unset, and don't surface a dead path as a selectable option.

## Evidence

- `/ai/coder` workspace combobox showed `/media/tan/Workspace/.../portal_desktop` on Windows;
  source `src/lib/domains/coder/components/CoderProjectSelector.svelte` (`STORAGE_KEY =
  'portal-coder-last-workspace'`).

## Fix

Root cause: `8114329f` only guarded `CoderProjectSelector`'s `!value` default
(`CoderProjectSelector.svelte:45-51`), and that code never runs on this flow. `CoderPanel`
onMount calls `coderUi.initFromStorage()` (`CoderPanel.svelte:246-249`), which reads the same
`portal-coder-last-workspace` key unvalidated, seeds `coderUi.activeProjectPath` and then
`coderSession.workspaceRoot`. Everything downstream trusted it: the selector's default never
fired and listed it as an option (`:87-89`) and re-saved it (`:62-64`),
`ProjectWorkspaceList.onMount` re-selected the unmatched path, and `CoderSessionList.svelte:83`
used it as `workspace_root` for new threads.

Fix, once at the source (`state/coderUi.svelte.ts`): `resolveProjectFromList` now drops the
saved path and its localStorage key when the project list is non-empty and none matches.
`ProjectWorkspaceList.onMount` then falls through to `selectProject(projects[0])`, and
`CoderPanel.handleProjectSelect` overwrites `workspaceRoot` (only while no thread is open). An
empty list (not loaded yet, or a load error) keeps the saved path, so a valid saved choice is
never wiped. Test: `state/coderUi.test.ts` (known project kept, unknown path dropped, list not
loaded kept); mutation-checked, each case fails if its branch is removed.

Checks: `pnpm vitest run src/lib/domains/coder` 14/14, `pnpm check` 0 errors, `pnpm lint` exit 0.

To verify (tauri): in the app's devtools/CDP set `localStorage['portal-coder-last-workspace'] =
'/media/tan/Workspace/production/portal/portal_desktop'`, reload, open `/ai/coder`. The workspace
selector must show the first project the list returns (not the `/media/...` path), its dropdown
must have no `/media/...` option, and the stored key must now hold that project's path. Also
plant a real project's path: it must stay the default.

Known limits: a real folder that was never registered as a project is dropped too (same
known-project rule as `8114329f`; `validate_project_path` would keep it). Below 768px the sessions
panel is a Sheet, which by reading is not mounted until opened, so there the default only heals
once it is opened (not run; the desktop window's `minWidth` is 1024, so only a narrow
browser/client build is affected).

## Verification

2026-11-20, tauri (dev instance). `scripts/QA-134-verify.mjs`: planted `localStorage['portal-coder-last-workspace']='/media/tan/Workspace/production/portal/portal_desktop'`, loaded `/ai/coder` with the real project list (170 projects). Result: key rewritten to the first project's path, no `/media/tan` text anywhere in the page, a real project path replanted stays. Shot: `.claude/qa/shots/QA-134-after-stale-dropped.png`. Original localStorage value restored afterwards. Known tradeoff (never-a-project folder is dropped too) not reopened.

---
id: QA-131
title: "Projects: metadata is captured once at create-time and never auto-resynced — a project created on an empty folder still shows \"not a git repository\" and the wrong stack after it's been scaffolded"
status: fixed
severity: minor
kind: bug
area: projects
mode: tauri
found: 2026-11-11
---

## Use case

A developer creates a project pointing at a folder, then populates it (scaffold, git init,
add deps). They expect the project's detected stack / git status to reflect the current
folder.

## Steps

1. Create a project on an **empty** dir (`analyze_project_directory` runs against empty).
2. Scaffold into it (here: the coder ran `create-expo-app`, which also `git init` +
   "Initial commit").
3. Reopen `/projects/201/pipelines` > Details.

## Expected

Details reflect the now-populated repo (git repo present, Expo/React-Native, test script).

## Actual

- Details still show **"This folder is not a git repository"** though the folder has a git
  repo with a commit.
- Stack shows only "React / TypeScript / pnpm" (detection from a later point, but git state
  is stale). There is no automatic re-analysis when the folder changes; "Sync" exists only in
  the edit form and is easy to miss.

## Why it matters

Stale metadata drives the wrong smart-default actions (see QA-128) and misleads the developer
about project state.

## Suggested fix

Re-run `analyze_project_directory` on project focus / after a coder session touches the
folder, or show a "metadata looks stale — Sync" nudge on the project page with a one-click
re-detect.

## Evidence

- `/projects/201` Details panel: "This folder is not a git repository" vs. `git log` showing
  "Initial commit".

## Fix

Fixed as the issue's second suggestion (a nudge with a one-click re-detect) by `8114329f`; no code
changed for this close-out.

`ProjectGitCard.svelte:34-47`: when the card has no branch and no repository it now reads "This
folder is not a git repository — or the details are out of date." with a **Re-detect** link
button (new `refreshing` / `onRedetect` props). `routes/projects/[id]/+page.svelte:598-603` wires
it to `handleRedetect` (`:332-349`), which calls `projectService.refreshProjectMetadata` ->
`ProjectService::refresh_project_metadata` (`project_service.rs:285`): it re-reads size, git info
and the stack from disk, then invalidates the project queries. The stack card, the commands card
and the page's actions menu use the same handler, so the action is no longer easy to miss.

NOT built: automatic re-analysis on project focus / open, or after a coder session touches the
folder (the issue's first suggestion). Metadata stays stale until the user clicks Re-detect; the
only other caller of `refreshProjectMetadata` is the unavailable-projects relink flow.

Docs only, nothing run. Not re-tested live.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

---
id: QA-131
title: "Projects: metadata is captured once at create-time and never auto-resynced — a project created on an empty folder still shows \"not a git repository\" and the wrong stack after it's been scaffolded"
status: open
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

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

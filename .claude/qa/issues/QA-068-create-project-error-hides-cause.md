---
id: QA-068
title: Create Project with a non-existent path shows the generic "Failed to save project. Please try again."
status: verified
severity: minor
kind: ux
area: projects/create
mode: tauri
found: 2026-11-06
---

## Use case

Create a project by typing / pasting a path.

## Steps

1. `/projects/create`, path `C:\Users\tanvo\no-such-qa-dir`, name `QA Throwaway`, Create Project.

## Expected

The page says why: "Project path does not exist", ideally inline on the Project Path field. No retry loop for a path that will never work.

## Actual

Red banner "Failed to save project. Please try again." The real reason (`Project path does not exist`) only appears in the console log (`[ProjectService] Failed to create project Project path does not exist`). The Path field is not marked invalid.

## Evidence

`.claude/qa/shots/QA-068-before-create-error-hides-cause.png`

## Repro

`node .claude/qa/scripts/QA-068.mjs`

## Fix

`ProjectForm.svelte` submit catch now shows the thrown error's message (backend: "Project path does not exist") in the error banner, falling back to the generic text only when there is none. Not marked inline on the Path field. Not re-driven through the UI (no QA-068 interaction run).

## Verification

<!-- qa-tester -->

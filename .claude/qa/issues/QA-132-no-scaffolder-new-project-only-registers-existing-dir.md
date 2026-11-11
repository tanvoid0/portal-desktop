---
id: QA-132
title: "Projects: no scaffolder — New Project only registers an existing directory, so a brand-new app can't be bootstrapped from Portal; there is no project→scaffold→open-in-coder chain"
status: open
severity: minor
kind: enhancement
area: projects
mode: tauri
found: 2026-11-11
---

## Use case

A developer wants to start a *new* app (e.g. an Expo app) and have Portal drive it end to end.

## Steps

1. Projects > New Project. The path field (`FolderPicker`) picks/accepts an **existing**
   directory; submit calls `analyze_project_directory` on it.

## Expected

An option to create a new app from a framework starter (e.g. `create-expo-app`,
`create-next-app`) that makes the folder, runs the starter, and opens the coder on it.

## Actual

- New Project only *registers* an existing dir — there is no "create app from framework"
  path. A new app must be bootstrapped elsewhere first (in this trial the built-in coder
  filled the gap by running `create-expo-app` itself).
- The three surfaces (Project ↔ coder ↔ Actions) converge on one dir once the project exists,
  but there's no single flow from "I want a new Expo app" to a coder session on it.

## Why it matters

The "new project" moment is where a dev tool can remove the most friction; today Portal
starts one step too late.

## Suggested fix

Add "Scaffold with…" to New Project (reuse the framework starter catalog), creating the dir,
running the starter action, and opening the coder — one chain from intent to a working repo.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

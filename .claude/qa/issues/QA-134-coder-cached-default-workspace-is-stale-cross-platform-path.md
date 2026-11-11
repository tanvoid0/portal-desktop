---
id: QA-134
title: "AI > Code: the coder's cached default workspace is a stale cross-platform path (`/media/tan/...` Linux mount) pre-selected on Windows, so a new session defaults to a non-existent directory"
status: open
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

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

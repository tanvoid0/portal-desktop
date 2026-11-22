---
id: QA-171
title: "Project header path chip prints the literal text `${parts.slice(-3).join('')}` for long paths"
status: verified
severity: minor
kind: bug
area: projects/detail
mode: tauri
found: 2026-11-22
---

## Use case

Read the project's folder in the header of the project page.

## Steps

1. Open a project whose path is longer than 56 chars with forward slashes, e.g. /projects/233 (`zz-tree-fixture`, a leftover fixture from an earlier session).
2. Look at the grey path text next to the status badge.

## Expected

A shortened path such as `C:\...\scratchpad\fixture-proj`.

## Actual

The chip shows `C:...${parts.slice(-3).join('')}`. In `src/lib/domains/projects/components/ProjectSummary.svelte:21-24` the backslashes in the template literal escape the interpolation (`\${`), and the split regex `/[\/]/` has lost its backslash, so Windows paths (`D:\...`) are never shortened (they just CSS-truncate).

## Evidence

`.claude/qa/shots/QA-171-before-path-chip-garbled.png`.

## Repro

`node .claude/qa/scripts/QA-171.mjs`

## Fix

Root cause: template literal had escaped `\${` and the split regex lost its backslash. Fixed in ProjectSummary.svelte: split on `[\/]`, join with the path's own separator. Checks: pnpm check (0 errors), prettier, eslint on touched files.

## Verification

2026-11-22, tauri. node QA-171.mjs: ok, /projects/233 path chip no longer shows the literal template.

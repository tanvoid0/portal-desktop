---
id: QA-005
title: Project pages use different page shells (left inset, widths, heading size)
status: verified
severity: polish
kind: ux
area: projects
mode: browser
found: 2026-11-05
---

## Use case
Move between project screens without the layout jumping.

## Steps
1. Screenshot at 1280x800: `/projects`, `/projects/audit`, `/projects/create`, `/projects/edit/1`, `/projects/1`, `/projects/1/pipelines/new`, `/projects/1/pipelines/run/1`.

## Expected
Same content left edge and heading style per `.cursor/rules/page-shell-conventions.md` (PageHeader).

## Actual
Content left edge: list/audit/detail/pipelines-new at x=280, create and edit at x=296, run at x=304. Headings: list/audit/create are large PageHeader titles; the run page title "Pipeline Run" is small. The create page has no header actions and, in the form, the Project Name input is narrower (~440px) than the Project Path input (full width). Run page uses `container mx-auto h-[calc(100vh-4rem)] p-6`, which also sizes off the viewport instead of the shell.

## Evidence
`.claude/qa/shots/sweep-projects-list-empty-1280.png`, `sweep-projects-create-1280.png`, `sweep-projects-pipelines-run-stuck-loading-1280.png`.

## Repro
`node .claude/qa/scripts/app.mjs projects/create x`

## Fix
Root cause: create/edit used `container mx-auto px-4 py-8`, run used `container ... h-[calc(100vh-4rem)] p-6`; app shell already pads. All now `space-y-6` like the list page; run monitor header uses `PageHeader`; ProjectForm Name input no longer inside a 2-col grid. Files: `routes/projects/create`, `edit/[id]`, run page, `ExecutionMonitor.svelte`, `ProjectForm.svelte`. Run left edge verified at 280 by screenshot; create/edit not re-shot.

## Verification

2026-11-05, browser mode (HMR build), 1280x800 and 1024x600. Content left edge 280 on create, edit and run (shots QA-005-after-create-left-edge.png, QA-005-after-edit-left-edge.png, QA-005-after-run-left-edge-error-state.png). Project Name input is full width; frameworks/package managers/languages row still lays out in 3 columns. Verified.

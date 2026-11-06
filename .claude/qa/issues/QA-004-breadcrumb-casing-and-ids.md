---
id: QA-004
title: Breadcrumbs on project sub-pages are ALL CAPS and show raw ids/segments
status: verified
severity: polish
kind: ux
area: projects
mode: browser
found: 2026-11-05
---

## Use case
Orient yourself with the top breadcrumb.

## Steps
1. Open `/projects`, `/projects/create`, then `/projects/edit/1`, `/projects/1/pipelines`, `/projects/1/pipelines/new`, `/projects/1/pipelines/run/1`.

## Expected
Consistent casing ("Projects > Create Project") on every project route; the project name or a readable label rather than a bare id/URL segment.

## Actual
List, create and audit show "Projects". The other routes show `PROJECTS > EDIT > 1`, `PROJECTS > 1 > PIPELINES > NEW`, `PROJECTS > 1 > PIPELINES > RUN > 1` (upper-cased URL segments; the project name only appears once the project loads, and the run page never sets the name when the project is missing). "Pipelines" and "Run"/"New" are not clickable-looking links either.

## Evidence
`.claude/qa/shots/QA-004-before-edit-caps-crumb.png`, `sweep-projects-pipelines-new-empty-1280.png`, `sweep-projects-pipelines-run-stuck-loading-1280.png`.

## Repro
`node .claude/qa/scripts/app.mjs projects/1/pipelines/new bc`

## Fix
Root cause: store fallback upper-cased raw segments and the edit crumb linked to `/projects/<name>`. Fallback now humanizes (`Pipelines`, `#1`); `setProjectEditBreadcrumbs` takes the id; new `setProjectSubpageBreadcrumbs` used by pipelines/new and run pages (Projects > name > Create Pipeline / Pipeline Run). Files: `breadcrumbStore.ts`, `routes/projects/edit/[id]`, `pipelines/new`, `pipelines/run/[executionId]` pages. Note: custom crumbs are never cleared on navigation (pre-existing, not touched). Project name crumb needs the project to load (browser mode has no backend, so shows fallback).

## Verification

2026-11-05, browser mode (HMR build), 1280x800 and 1024x600. Crumbs: Projects > Create Project; Projects > Edit > #1; Projects > #1 > Pipelines; ... > Pipelines > New; ... > Run > #1. No caps, no pageerrors. Project-name crumb needs the real app. Verified.

2026-11-05 re-verify, browser mode: run/<full UUID> shown whole; tasks/abc-def_1 -> "Tasks > abc-def_1"; github/repos/some-owner/my_repo -> "Github > Repos > some-owner > my_repo"; pipelines/new -> sidebar Projects leaves just "Home > Projects" (no stale crumbs). Still verified.
Screenshot: .claude/qa/shots/QA-004-after-uuid-crumb.png
Also: QA-004-after-detail-crumb.png, QA-004-after-pipelines-crumb.png, QA-004-after-pipelines-new-crumb.png, QA-004-after-tasks-param-crumb.png, QA-004-after-uuid-crumb-round-1.png

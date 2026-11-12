---
id: QA-137
title: Pipelines created in the builder are unreachable - no list, edit, update or run UI; /pipelines is a 404
status: verified
severity: major
kind: bug
area: projects/pipelines
mode: tauri
found: 2026-11-12
---

## Use case

Create a pipeline from a template, then locate it again, edit steps and re-save, run it.

## Steps

1. `/projects/14/pipelines/new` > All Templates > Next.js Full Pipeline > Save Pipeline.
2. App redirects to `/projects/14/pipelines`, which redirects to `/projects/14?tab=actions`.
3. Look for the saved pipeline on the Actions tab; open `/pipelines`.

## Expected

The saved pipeline is listed (Workflows or its own list), can be opened in the builder, updated and run.

## Actual

- Actions tab shows only the generated workflow "CI" and defaults; the saved pipeline is nowhere
  (the DB has it: `get_pipelines` returned it). No success toast either.
- `/pipelines` (global) returns "404 Page not found".
- The only mount of `PipelineBuilder` is the `new` page, so there is no edit/update entry point in the UI.
- Builder lists steps only as `Block: template-command-lint`; no command is shown and the only
  per-step action is **Remove** (a command cannot be edited, only the step deleted).
- Data is DB-only: no file written in the repo (`git status` clean after save).

## Why needs-decision

The route comment says "Legacy pipeline management route - redirects to the unified Actions tab", so the old
pipeline store may be intentionally superseded while `/projects/:id/pipelines/new` was left live. Owner decides:
wire saved pipelines into Actions (list/edit/run) or retire the Create Pipeline flow.

## Evidence

`.claude/qa/shots/QA-137-before-after-save-no-pipeline-visible.png`,
`.claude/qa/shots/QA-137-before-pipelines-route-404.png`,
`.claude/qa/shots/QA-139-before-nextjs-template-in-builder.png`.

## Repro

manual - steps above.

## Fix

Root cause: legacy template builder (DB-stored pipelines) left live beside `.portal/pipeline.yml`. Fix = retired per owner decision; `/projects/:id/pipelines` already redirects to the Actions tab and stays.

Deleted: `src/routes/projects/[id]/pipelines/new/`, `PipelineBuilder.svelte`, `BlockLibrary.svelte` (only used by builder), `pipelineTemplateService.ts`, `utils/customTemplates.ts`, `stores/pipelineStore.ts`, and automation `TemplateList.svelte` / `TemplateEditDialog.svelte`.
Changed: `automation/components/BlocksPage.svelte` (Templates tab removed, Blocks only), `projects/pipelines/index.ts` (dropped re-exports).
Kept: `frameworkStarterService` (+test; imported by `actions/defaults.ts`), `pipelineService` (automationExecutionService), execution/run-history components, run route, `/pipeline-runs`, block library service/store, types.
Checks: prettier + eslint on touched files clean; `svelte-check --threshold error` 0 errors; grep shows no dangling imports.
Follow-up: Rust `get_pipelines`/create/update pipeline commands and DB rows now unused by the UI except automationExecutionService; `PipelineTemplate` type left in types. Also covers QA-138 and QA-139 (builder/template symptoms) if they are builder-only.

## Verification

2026-11-12, mode tauri (dev build :9223). /projects/14/pipelines/new -> 404 page (shot QA-137-after-pipelines-new.png); /projects/14/pipelines -> redirects to ?tab=actions; /pipeline-runs loads; /automation/blocks has no Templates tab (QA-137-after-blocks-page.png). Script: qa-v137.mjs. Gone as intended.

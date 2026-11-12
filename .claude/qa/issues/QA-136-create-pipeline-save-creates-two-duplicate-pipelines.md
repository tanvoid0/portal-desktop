---
id: QA-136
title: Create Pipeline > Save Pipeline inserts the pipeline twice (duplicate rows)
status: wontfix
severity: major
kind: bug
area: projects/pipelines
mode: tauri
found: 2026-11-12
---

## Use case

Create a pipeline from a template (`/projects/<id>/pipelines/new`), edit it, click Save Pipeline.

## Steps

1. `/projects/14/pipelines/new` > All Templates > pick any template > rename to `QA dup`.
2. Click **Save Pipeline** once.
3. List rows: `invoke('get_pipelines', { projectId: 14 })`.

## Expected

One pipeline row.

## Actual

Two rows with the same name and different ids (seen ids 11 and 12; 2 steps each). Reproduced twice.

## Evidence

Release v0.14.0 (`578f80a1`). Cause in code: `PipelineBuilder.svelte` `handleSave()` calls
`pipelineService.createPipeline(...)` and then `onSave?.(newPipeline)`; the page's
`handleSavePipeline` (`src/routes/projects/[id]/pipelines/new/+page.svelte`) calls
`pipelineService.createPipeline(pipeline)` again with the already-created pipeline. The builder also
swallows save errors (console.error only, no toast). Fixtures deleted afterwards via `delete_pipeline`.

## Repro

`node .claude/qa/scripts/QA-136.mjs` (self-cleaning; prints `BUG: 2 rows` until fixed).

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

## Decision (owner, 2026-11-12)

wontfix: the old template builder is being retired (QA-137). Stack-aware commands belong in the Actions defaults (QA-140).

---
id: QA-038
title: "Update Task with AI" shows "Confidence: 100% | Model: existing-task" before anything is generated
status: verified
severity: polish
kind: copy
area: tasks/[id]/generate
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens "Generate Tasks with AI" from a task to refine it.

## Steps

1. Open `/tasks/<id>/generate`.

## Expected

The Preview card shows the loaded task without AI metadata, or labels it "Loaded from existing task".

## Actual

The Preview header carries a badge "Confidence: 100% | Model: existing-task" although nothing was generated; reads as a real AI result. The chat panel also repeats the QA-030 header overflow.

## Evidence

`.claude/qa/shots/sweep-tasks-id-generate.png`

## Repro

Manual, steps above.

## Fix

Root cause: TaskGenerationPage seeds generatedData with model_used 'existing-task' and the badge always rendered. Now hidden for that value.
Files: src/lib/domains/tasks/components/TaskGenerationPage.svelte. Checks as QA-025. The chat header overflow (QA-030) is out of scope here.

## Verification

2026-11-06, tauri mode (:9222). /tasks/<id>/generate no longer shows 'Model: existing-task' badge. After shot: shots/QA-038-after-update-task-ai.png. Verified.

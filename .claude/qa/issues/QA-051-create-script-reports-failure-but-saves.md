---
id: QA-051
title: Creating a script shows "Failed to create script" but the script is saved
status: verified
severity: major
kind: bug
area: scripts/new
mode: tauri
found: 2026-11-06
---

## Use case

Owner creates a reusable script from `/scripts/new`.

## Steps

1. Open `/scripts/new`, fill Name `QA script`, Description, Command `echo QA`.
2. Click Save Script.

## Expected

Success toast, redirect to the script list.

## Actual

Red toast "Failed to create script"; page stays on `/scripts/new` with the form still filled. Console: `Failed to create block: RecordNotFound Error: Failed to find inserted item`. The script WAS created (it appears in `/automation/scripts`). A user who trusts the toast retries and creates duplicates.

## Evidence

`.claude/qa/shots/sweep-scripts-after-save.png` (toast, form still filled), `.claude/qa/shots/QA-051-before-script-saved-despite-error.png` (record exists).

Likely cause: `PipelineService::create_block` (`src-tauri/portal-core/src/domains/projects/pipelines/services/pipeline_service.rs:193`) -> `block_repo.create` hits SeaORM's string-PK `insert()` RecordNotFound. `script_execution_repository.rs:46-65` already has a "verify the insert then succeed" workaround; the block repo does not.

## Repro

`node .claude/qa/scripts/s5g.mjs` (creates a `QA script` fixture; delete it afterwards).

## Fix

Root cause: `BlockRepository::create` used `.insert()` on a string-PK model; SQLite makes SeaORM return RecordNotFound after a successful insert. Applied the verify-after-insert workaround from `script_execution_repository.rs`.
Files: src-tauri/portal-core/src/domains/projects/pipelines/repositories/block_repository.rs (frontend double-submit guard in src/routes/scripts/new/+page.svelte was done earlier).
Other `.insert(` sites: string-PK entities are only block, execution (execution_repository.rs, already workaround), script_execution (already), credential_service (already), ai conversation commands (already). Custom_script, pipeline, project, document, saved_view, task_attachment, etc. are i32 PKs, unaffected. Nothing else to fix.
Checks: cargo check 0 errors (no new warnings); clippy shows nothing for touched files. Repro not rerun (no app driving allowed); needs rebuild.

## Verification

2026-11-06, tauri. Created fixture `QA script` (echo QA) with a double-click on Save: no failure toast, redirected to /automation/scripts, exactly 1 card (no duplicate). Shot `.claude/qa/shots/QA-051-after-script-save.png`. Verified.

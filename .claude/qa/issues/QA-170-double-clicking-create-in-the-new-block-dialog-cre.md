---
id: QA-170
title: Double-clicking Create in the new-block dialog creates two blocks
status: verified
severity: minor
kind: bug
area: automation/blocks
mode: tauri
found: 2026-11-22
---

## Use case

Create a block quickly.

## Steps

1. /automation/blocks, click Create.
2. Fill Name `QA block dbl`, Description, Command.
3. Double-click the dialog's Create button.

## Expected

One block. The button disables (or the handler ignores re-entry) while the request is in flight; Documents, Tasks, Projects and Scripts all produced a single record for the same double-click.

## Actual

Two identical blocks are created and two `Block created successfully` toasts stack.

## Evidence

`.claude/qa/shots/QA-170-before-duplicate-blocks.png`.

## Repro

`node .claude/qa/scripts/QA-170.mjs` (prints the count, deletes its fixtures)

## Fix

Root cause: handleSaveBlock had no re-entry guard (dialog stays open until request resolves). Fix: `saving` flag with try/finally. File: automation/components/BlocksPage.svelte. Checks: pnpm check, eslint.

## Verification

2026-11-22, tauri. node QA-170.mjs: ok, 1 block from a double-click.

---
id: QA-178
title: Delete confirmations for scripts and blocks do not say which item is being deleted
status: verified
severity: polish
kind: copy
area: automation/scripts, automation/blocks
mode: tauri
found: 2026-11-22
---

## Use case

Delete one script or block among many.

## Steps

1. /automation/scripts or /automation/blocks, click the red trash on a card.
2. Read the dialog.

## Expected

`Delete "QA block dbl"?` as Documents and Projects do.

## Actual

`Are you sure you want to delete this script?` / `...this block?` with no name.

## Evidence

`.claude/qa/shots/QA-178-before-delete-dialog-generic.png`.

## Repro

manual - steps above

## Fix

Root cause: generic confirm copy. Fix: `Delete "<name>"?` in Scripts list, Blocks list and script detail page (name looked up from loaded list). Files: ScriptsPage.svelte, BlocksPage.svelte, src/routes/scripts/[id]/+page.svelte. Checks: pnpm check, eslint.

## Verification

2026-11-22, tauri. Block dialog reads Delete "Install NPM Dependencies"? (QA-178-after-block-delete-dialog.png); script dialog read Delete "QA script long xxx"? during the QA-169 rerun.

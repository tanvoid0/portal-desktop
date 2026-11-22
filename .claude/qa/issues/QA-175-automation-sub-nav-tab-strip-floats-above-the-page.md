---
id: QA-175
title: Automation sub-nav tab strip floats above the page title, is indented 26px from it, and the n8n title truncates before its description
status: verified
severity: polish
kind: ux
area: automation/*
mode: tauri
found: 2026-11-22
---

## Use case

Move between Quick Run, Blocks, Scripts, Utilities and Workflows.

## Steps

1. Open /automation/scripts, /automation/blocks, /automation/workflows at 1280x800.
2. Compare the tab strip with the page content below.

## Expected

The strip aligns with the content's left edge (x=280) and has space before the heading; page titles never truncate before their description.

## Actual

- Tab strip starts at x=306 while title and cards start at x=280, and it sits flush on the `h1` row with no gap.
- Workflows: the title renders `n8n Workfl...` while the long description next to it renders almost in full.
- Blocks has no title at all (no `h1`), unlike Scripts, Workflows and Utilities.

## Evidence

`.claude/qa/shots/QA-175-before-n8n-title-tabstrip.png`, `.claude/qa/shots/sweep-automation-light-b.png`, `.claude/qa/shots/sweep-blocks-light-b.png`.

## Repro

manual - steps above

## Fix

Root cause: automation +layout added its own gutter on top of the page container (26px indent) and no gap below; n8n description was long enough to squeeze the h1; BlocksPage had no header. Files: src/routes/automation/+layout.svelte (drop px gutter, mb-4), src/routes/automation/workflows/+page.svelte (short description), src/lib/domains/automation/components/BlocksPage.svelte (PageHeader 'Blocks'). Checks: pnpm check, prettier, eslint on touched files. Not visually rerun.

## Verification

2026-11-22, tauri. Tab strip aligned at x=280 with a gap above the heading; Workflows title is full 'n8n Workflows'; Blocks has a 'Blocks' header (QA-175-after-automation-workflows.png, QA-175-after-automation-blocks.png).

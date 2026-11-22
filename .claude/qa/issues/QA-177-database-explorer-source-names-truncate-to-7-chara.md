---
id: QA-177
title: "Database Explorer: source names truncate to ~7 characters and the \"Enable editing\" switches have no visible label"
status: verified
severity: minor
kind: ux
area: database
mode: tauri
found: 2026-11-22
---

## Use case

Pick which database source to browse and decide whether to allow edits.

## Steps

1. /database at 1280x800 (also 1024x600, 2560x1080).
2. Read the source rows in the tree.

## Expected

Readable source names (`MongoDB (SDK service)`, `Portal (this app)`, `PostgreSQL (SDK service)`) and a visibly labelled edit toggle; the tree panel scales with the window.

## Actual

Rows read `MongoD...`, `Portal ...`, `Postgr...` because the name shares one line with a switch and a type badge in a fixed 256px panel. The switch's only label is an aria-label (`Enable editing`), so sighted users see three unexplained toggles. At 1024px the page title becomes `Database E...`; at 2560px the data pane is a large empty area beside the fixed panel.

## Evidence

`.claude/qa/shots/QA-177-before-db-sources-truncated.png`, `.claude/qa/shots/sweep-database-light-s.png`, `.claude/qa/shots/sweep-database-light-u.png`.

## Repro

manual - steps above

## Fix

Root cause: label, switch and badge shared one row in a fixed 16rem panel; switch had only aria-label. Files: src/lib/domains/database/components/SourceNode.svelte (label wraps, switch on its own row with visible Label), src/routes/database/+page.svelte (panel clamp(16rem,22vw,24rem), shorter header description so title no longer truncates at 1024). Checks: pnpm check, prettier, eslint. Not visually rerun.

## Verification

2026-11-22, tauri. Source names read 'MongoDB (SDK service)', 'Portal (this app)'; each source has a visible 'Enable editing' switch; title no longer truncates (QA-177-after-database.png).

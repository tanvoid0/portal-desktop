---
id: QA-072
title: Script and Block "Import JSON" report "JSON imported - review and save" for JSON that is not a script/block, leaving the form empty
status: verified
severity: minor
kind: bug
area: automation/scripts, automation/blocks
mode: tauri
found: 2026-11-06
---

## Use case

Paste a script/block JSON and review it before saving.

## Steps

1. `/scripts/new?import=true` (Scripts > Import JSON), paste `[1,2,3]` or `{"foo": 1}`, click Import & Preview.
2. Same in Blocks > Create Block > "Import from JSON".

## Expected

Validation error naming the missing fields (name, command), form untouched, import card stays open.

## Actual

Success toast "JSON imported - review and save", the import card collapses, Name/Command stay empty. Malformed JSON is handled correctly ("Invalid JSON format"); only valid-but-wrong JSON slips through.

## Evidence

`.claude/qa/shots/QA-072-before-import-array-says-imported.png`

## Repro

`node .claude/qa/scripts/QA-072.mjs` (scripts page; the blocks dialog behaves the same by hand).

## Fix

New shared `domains/automation/utils/importJson.ts` (`parseImportJson`, with `importJson.test.ts`): requires an object with string `name` and `command`, else returns an error naming the missing fields; used by both `scripts/new/+page.svelte` and `BlocksPage.svelte`, which keep the form and import card untouched on error. QA-072.mjs: no toast, error shown.

## Verification

<!-- qa-tester -->

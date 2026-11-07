---
id: QA-088
title: Block and template import were dead — both handlers clicked a file input that did not exist
status: verified
severity: major
kind: bug
area: automation
mode: tauri
found: 2026-11-07
---

## Use case

Import a pipeline block, or import a pipeline template, from a `.json` file on disk.

## Problem

Both import paths in `src/lib/domains/automation/components/BlocksPage.svelte` ended at an
element that was not in the document:

```js
handleImportBlock()    → document.getElementById('block-import-input').click()
handleImportTemplate() → document.getElementById('template-import-input').click()
```

Neither `block-import-input` nor `template-import-input` existed anywhere in the file, so
clicking **Select File** on the block import dialog and **Import Template** on the Templates
tab did nothing at all — no file picker, no error, no toast. `handleFileSelect` and
`handleTemplateFileSelect` were therefore unreachable code.

The likely cause is an earlier `portal/no-raw-ui-elements` lint sweep removing the two hidden
`<input type="file">` elements without following the `getElementById` calls that depended on
them. The lint rule permits a hidden file input, so there was nothing to gain by removing
them.

## Expected

Clicking Select File / Import Template opens a file picker; choosing a `.json` file creates
the block or template.

## Actual (before the fix)

Nothing happens. No picker, no error, no toast.

## Not a regression from the UX work

Pre-existing. Found while doing UX-007 and UX-018's `BlocksPage` split, not caused by them.

## Fix

Both imports now use a `bind:this` hidden `<input type="file" accept=".json">` instead of
`getElementById`, so the element is guaranteed to exist. They route to the same service calls
the dead handlers would have reached:

- block import → `JSON.parse(fileText)` → `blockLibraryService.createBlock(...)`
- template import → `pipelineTemplateService.importTemplate(...)`

Landed as part of the UX-007 / UX-018 pass on `BlocksPage.svelte` (1088 → 311 lines, with
`BlockImportDialog.svelte` extracted). Not committed yet.

## Repro

1. Open `/automation/blocks`.
2. Click **Import**, then **Select File** in the dialog. Before the fix: no file picker.
3. Switch to the **Templates** tab and click **Import Template**. Before the fix: no picker.

## Verification

2026-11-07 — tauri, dev instance 2. Ran `node .claude/qa/scripts/qa088-verify.mjs`,
`qa088-verify2.mjs`, `qa088-verify3.mjs`, `qa088-persist.mjs`.

Both file inputs now exist and both pickers open:

- **Block import** — the Import dialog contains one `input[type=file][accept=".json"]` bound with
  `bind:this`, no `getElementById` anywhere. Importing a valid block JSON created the block:
  42 → 43 cards, toast "Block imported successfully", "QA Imported Block" rendered, and it was
  still there after a reload (it persists). Deleting it put the count back to 41.
- **Template import** — the Templates tab's Import Template button opens a picker from a real
  file input. A valid template JSON (`key`/`name`/`description`/`steps`) imported: 11 → 12 cards,
  toast "Template imported successfully".

Bad input is reported, not swallowed (the QA-072 concern):

| file | block import | template import |
| --- | --- | --- |
| `bad.txt` (not JSON) | toast "YAML import not yet supported. Please use JSON format." | toast "Invalid JSON format" |
| `wrong-shape.json` (`{"hello":"world"}`) | toast "Failed to import block" + console "Block name is required" | toast "Invalid template format: missing required fields" |

Neither created anything. Count unchanged in all four cases.

Two notes, neither reopening this issue:

- The block importer's message for a non-JSON file is "YAML import not yet supported. Please use
  JSON format." — it is keyed off `!file.name.endsWith('.json')`, so a `.txt` file is reported as
  a YAML problem. Misleading copy, worth a one-line fix next time this file is open.
- The imported *template* does not survive a reload and can never be deleted — filed as **QA-100**.
  That is a separate defect in `pipelineTemplateService`, not in the import wiring this issue is
  about.

Shots: `.claude/qa/shots/QA-088-after-block-import.png`,
`.claude/qa/shots/QA-088-after-valid-template-import.png`,
`.claude/qa/shots/QA-088-after-templates-query-empty-state.png`.

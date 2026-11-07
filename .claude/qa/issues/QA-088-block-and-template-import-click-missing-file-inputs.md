---
id: QA-088
title: Block and template import were dead — both handlers clicked a file input that did not exist
status: fixed
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

<!-- qa-tester: confirm both pickers open and that importing a valid block JSON and a valid
     template JSON each create the item; also confirm a non-JSON and a wrong-shape JSON file
     report an error rather than failing silently (compare QA-072, which filed exactly that
     for script import). -->

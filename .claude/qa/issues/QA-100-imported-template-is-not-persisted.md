---
id: QA-100
title: An imported pipeline template vanishes on reload and can never be deleted or edited
status: open
severity: major
kind: bug
area: automation/blocks (Templates)
mode: tauri
found: 2026-11-07
---

## Use case

Import a pipeline template from a JSON file and keep using it.

## Steps

1. `/automation/blocks` → **Templates** → **Import Template**.
2. Pick a valid template JSON (`key`, `name`, `description`, `steps` — the shape
   `importTemplate` validates).
3. Toast says "Template imported successfully" and the card appears: 11 templates → 12.
4. Reload `/automation/blocks` and open Templates again.

## Expected

The imported template is still there, and can be edited or deleted like any other custom one.

## Actual

- After the reload it is gone — back to 11 cards, "QA Imported Template" nowhere in the body.
  `importTemplate` ends at `this.addTemplate(template)`
  ([pipelineTemplateService.ts:1041](../../../src/lib/domains/projects/pipelines/services/pipelineTemplateService.ts#L1041)),
  and `addTemplate` only pushes into the in-memory `this.templates` Map
  ([line 852-856](../../../src/lib/domains/projects/pipelines/services/pipelineTemplateService.ts#L852)).
  Nothing is written anywhere, so the import is lost with the page.
- While it is on screen its card carries exactly one button:

  ```
  custom template card buttons: [ 'Export template QA Imported Template' ]
  ```

  Edit and Delete are gated on `template.id`
  ([TemplateList.svelte:93-108](../../../src/lib/domains/automation/components/TemplateList.svelte#L93)),
  and an imported template has no `id`, so it renders as if it were built-in. Even within the
  session there is no way to remove it.

The success toast is therefore misleading: it reports a durable import that did not happen.

## Evidence

```
valid template import toast: Template imported successfully | after: 12 | present: true
after reload, QA Imported Template present: false
cards: 11
```

`.claude/qa/shots/QA-088-after-valid-template-import.png`

## Repro

`node .claude/qa/scripts/qa088-verify3.mjs` then `node .claude/qa/scripts/qa088-persist.mjs`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

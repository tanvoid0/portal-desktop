---
id: QA-100
title: An imported pipeline template vanishes on reload and can never be deleted or edited
status: verified
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

Round 1: persistence via `localStorage` (see below — unchanged, confirmed working by the
tester).

Round 2, the reopened bug: `addTemplate` bucketed by `template.framework || 'other'` while
`deleteTemplate`/`updateTemplate` looked the bucket up via `template.framework || ''` — two
different defaults for the exact same "no framework" case, so a framework-less template (a
legitimate shape — `importTemplate` never requires `framework`) got filed under `'other'` but
searched for under `''`, missing every time. Added one `frameworkKey(template)` helper and
made `addTemplate`/`deleteTemplate`/`updateTemplate` all call it — one default, used
everywhere this map is keyed.

Also fixed the secondary bug the tester flagged in the same report: `BlocksPage.svelte`'s
delete handler ignored `deleteTemplate`'s boolean return and always toasted success. It now
toasts an error and bails when the delete actually returns `false`.

Files: `src/lib/domains/projects/pipelines/services/pipelineTemplateService.ts`,
`src/lib/domains/automation/components/BlocksPage.svelte` (+ round-1's new util, unchanged).
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-100.mjs` (import → reload → check) then
`node .claude/qa/scripts/QA-100-cleanup.mjs` / `QA-100-edit-check-and-cleanup.mjs` (delete and
edit the imported fixture).

**Import + persistence itself works now**: "QA Imported Template" imports, shows the toast,
survives a full reload (confirmed: still present and `id`-bearing after reload, Edit/Delete
buttons present where only Export existed before). That half of the issue is fixed.

**Reopened — edit and delete on an imported template are both broken, one silently.**

- **Delete**: clicking the row's Delete button shows a confirm ("Delete template | Are you sure
  you want to delete template "QA Imported Template"?"), confirming shows a **success** toast
  ("Template deleted successfully") — but the card never leaves the list, and it is still
  present after a completely fresh reload. The delete is a no-op that reports success.
- **Edit**: clicking Edit, changing the name and saving throws visibly —
  console error `Failed to update template` from
  [pipelineTemplateService.ts:1070](../../../src/lib/domains/projects/pipelines/services/pipelineTemplateService.ts#L1070)
  (`updateTemplate`'s own `throw new Error('Failed to update template')`), plus a matching
  error toast. The rename never applies.

Root cause (one bug, both symptoms): `addTemplate` buckets templates by
`template.framework || 'other'`
([pipelineTemplateService.ts:860](../../../src/lib/domains/projects/pipelines/services/pipelineTemplateService.ts#L860))
— defaults to the **string `'other'`** — but `deleteTemplate`
([line 1078](../../../src/lib/domains/projects/pipelines/services/pipelineTemplateService.ts#L1078))
and `updateTemplate` (line ~1112) both look the bucket up via `template.framework || ''` —
defaults to **empty string**. Any imported template with no `framework` field (not required by
`importTemplate`'s own validation, which only requires `key`/`name`/`description`/`steps`) gets
added under `'other'` but looked up under `''` on delete/update, so `frameworkTemplates` is
`undefined` and both silently miss: `deleteTemplate` falls through its `if (frameworkTemplates)`
guard straight to `return false` (and the caller shows a success toast regardless of the return
value — a second, smaller bug), `updateTemplate` falls through to its own `throw`.

Minimal repro template used here had no `framework` key at all — that's a legitimate shape per
the importer's own validation, not an edge case.

Cleaned up this fixture by removing it directly from `localStorage`
(`portal.pipelines.customTemplates`) since the in-app Delete path is exactly what's broken;
confirmed gone after a fresh reload.

### Round 2 — 2026-11-09, mode: tauri

`node .claude/qa/scripts/QA-100.mjs` then `QA-100-edit-recheck.mjs` — same template shape
(no `framework` field) that broke delete/update last round:

```
import:  toast "Template imported successfully", present: true, survives reload
edit:    toast "Template updated successfully", renamed card visible: true
delete:  present after delete: false; present after fresh reload: false
```

Both halves work now and the delete actually persists this time (unlike round 1's silent
false-positive). Fixture cleaned up via the in-app Delete button (no localStorage workaround
needed this round). Verified.

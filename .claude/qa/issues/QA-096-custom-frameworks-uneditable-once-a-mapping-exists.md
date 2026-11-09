---
id: QA-096
title: Custom frameworks cannot be edited or deleted once any mapping exists
status: verified
severity: major
kind: bug
area: settings/ides
mode: tauri
found: 2026-11-07
---

## Use case

Fix a typo in a custom framework, or remove one you no longer want.

## Steps

1. `/settings/ides` with no mappings. Create a custom framework ("QA Framework"). Its chip carries
   the edit/delete overlay — hover it and the pencil and X appear.
2. Create any framework → IDE mapping.
3. Go back to the framework chips ("Add More Frameworks") and hover "QA Framework".

## Expected

The framework stays editable and deletable.

## Actual

The overlay is gone — there is no edit or delete affordance for custom frameworks anywhere on the
page, and no other screen offers one:

```
with a mapping present, does the custom framework chip still carry edit/delete? [ false ]
after deleting the mapping                                                     [ true ]
```

Two different chip renderers: the no-mappings branch renders `frameworksByCategory` wrapped in
`<div class="group relative">` with the edit/delete overlay
([FrameworkIdeMappings.svelte:243-285](../../../src/lib/domains/settings/components/FrameworkIdeMappings.svelte#L243)),
while the has-mappings branch renders `unmappedByCategory` as bare buttons
([line 355-382](../../../src/lib/domains/settings/components/FrameworkIdeMappings.svelte#L355)).
The workaround — delete every mapping, edit the framework, recreate the mappings — is not
discoverable, and a framework that is itself mapped never appears in `unmappedByCategory` at all,
so a *mapped* custom framework can never be edited or deleted at all.

Pre-existing (the two branches exist on `main` too), carried through the split.

## Evidence

`.claude/qa/shots/ides-mappings-empty-after-delete.png`,
`.claude/qa/shots/ides-mappings-after-add.png`

## Repro

`node .claude/qa/scripts/ides-verify8.mjs`

## Fix

Root cause: two separate chip renderers — the no-mappings branch (with the edit/delete overlay)
and the has-mappings branch's `unmappedByCategory` (bare buttons, no overlay) — plus a mapped
custom framework never appearing in either list at all. Replaced both with one
`{#snippet frameworkChip}` used by both branches, gated on `framework.isCustom` for the overlay,
so an unmapped custom framework keeps its edit/delete affordance regardless of which branch is
showing. For a framework that is itself mapped (never in `unmappedByCategory`), added
Edit/Delete-the-framework buttons directly to its row in the mappings list (next to the
existing edit/delete-the-mapping buttons), gated the same way.

This same pass also deduplicates the chip source (`dedupedFrameworks`, fixes QA-103) and fixes
the invisible-overlay hit-testing (fixes QA-102) and adds the missing `aria-label`s (fixes
QA-095's chip-overlay half) — all four issues shared this one file and one rendering path.

File: `src/lib/domains/settings/components/FrameworkIdeMappings.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-096.mjs` — created "QA Editor" + "QA
Framework", mapped them (making the framework "mapped"), then confirmed the mapping row now
carries 4 icon buttons: edit/delete mapping AND edit/delete framework
(`.claude/qa/shots/QA-096-mapped-framework-still-editable.png` shows all four). Clicked
"Edit QA Framework" from the row, renamed to "QA Framework edited", chip updated correctly —
the exact workflow that was previously impossible once a mapping existed. Fixtures fully
cleaned up afterward. Verified.

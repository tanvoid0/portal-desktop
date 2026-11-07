---
id: QA-096
title: Custom frameworks cannot be edited or deleted once any mapping exists
status: open
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

<!-- coder -->

## Verification

<!-- qa-tester -->

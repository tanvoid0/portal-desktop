---
id: QA-095
title: IDE row and mapping row edit/delete buttons have no accessible name
status: open
severity: minor
kind: a11y
area: settings/ides
mode: tauri
found: 2026-11-07
---

## Use case

A keyboard or screen-reader user edits or removes a configured IDE.

## Steps

1. `/settings/ides` with at least one IDE configured.
2. Read the accessible names of the buttons on the IDE row.

## Expected

"Edit IDE" / "Delete IDE", the way `/logs` labels its Clear and Export icon buttons.

## Actual

```
[{"text":"Set Default","label":null,"title":null},
 {"text":"","label":null,"title":null},      <- pencil
 {"text":"","label":null,"title":null}]      <- trash
```

Both icon-only buttons at
[IdeList.svelte:203-218](../../../src/lib/domains/settings/components/IdeList.svelte#L203) have no
text, no `aria-label` and no `title`. The same applies to the mapping row's edit and delete buttons
at [FrameworkIdeMappings.svelte:339-349](../../../src/lib/domains/settings/components/FrameworkIdeMappings.svelte#L339)
and to the two buttons in the framework chip overlay (line 265ff), which are additionally invisible
(QA-102). Same class of defect as QA-046, QA-050, QA-061, QA-069.

## Evidence

`node .claude/qa/scripts/ides-verify.mjs` — output above.
`.claude/qa/shots/ides-after-add.png`

## Repro

`node .claude/qa/scripts/ides-verify.mjs`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

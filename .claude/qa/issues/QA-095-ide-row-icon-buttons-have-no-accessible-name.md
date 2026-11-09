---
id: QA-095
title: IDE row and mapping row edit/delete buttons have no accessible name
status: verified
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

Added `aria-label` to every icon-only button named in this issue:
- `IdeList.svelte`: "Edit IDE" / "Delete IDE".
- `FrameworkIdeMappings.svelte`: mapping row Edit/Delete (`Edit/Delete mapping for {framework}`),
  the chip overlay's Edit/Delete (`Edit/Delete {framework}` — same edit that fixed QA-102's
  overlay hit-testing), and the new framework-level Edit/Delete added to mapped rows for QA-096.

Files: `src/lib/domains/settings/components/IdeList.svelte`,
`src/lib/domains/settings/components/FrameworkIdeMappings.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors). Also fixes the chip-overlay half of QA-102/QA-096
(same file, same edit pass).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-095.mjs` — created fixtures "QA Editor"
(IDE), "QA Framework" + a mapping, then queried by accessible role name: "Edit IDE", "Delete
IDE", chip overlay "Edit QA Framework" / "Delete QA Framework", mapping row "Edit mapping" /
"Delete mapping" all resolve to exactly one button each. `.claude/qa/shots/QA-095-mapping-row-names.png`.
Fixtures cleaned up (0 remaining). Verified.

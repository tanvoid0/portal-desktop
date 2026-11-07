---
id: UX-007
title: Blocks page stacks five control rows (238px) above cards that are 300px tall for three facts
status: verified
severity: major
kind: ux
area: automation
mode: tauri
found: 2026-11-06
---

## Use case

Find a pipeline block and edit or run it.

## Measured (1280x800)

First content row at **y=278**, i.e. 238px of chrome, in five separate stacked rows:

| row | y | px | file |
| --- | --- | --- | --- |
| section tabs `Quick Run / Blocks / Scripts / Utilities` | 84 | 44 | [src/routes/automation/+layout.svelte:25-35](../../../src/routes/automation/+layout.svelte#L25) |
| PageHeader "Pipeline Blocks & Templates" | 128 | 44 | [BlocksPage.svelte:440](../../../src/lib/domains/automation/components/BlocksPage.svelte#L440) |
| sub-tabs `Blocks / Templates` | 178 | 36 | [BlocksPage.svelte:446-449](../../../src/lib/domains/automation/components/BlocksPage.svelte#L446) |
| `Import Block` + `Create Block` | 222 | 42 | [BlocksPage.svelte:453-463](../../../src/lib/domains/automation/components/BlocksPage.svelte#L453) |
| search + 5 category buttons | 278 | 46 | [BlocksPage.svelte:466-510](../../../src/lib/domains/automation/components/BlocksPage.svelte#L466) |

Three of those rows are half-empty: the Import/Create row holds two buttons against 900px of
whitespace; the sub-tab row holds two 60px tabs against 1050px of whitespace.

The page is also named three times on one screen — breadcrumb "Automation › Blocks", tab
"Blocks", h1 "Pipeline Blocks & Templates".

Below it, each block Card is **~300px tall** and spends most of that on three label:value
lines that are the same on every card:

```
Type:       command     ← identical on all 17 blocks
Version:    1.0.0       ← identical on all 17 blocks
Parameters: 2
```

17 blocks → 4709px of scroll in a 760px viewport.

## Expected

One control row; cards that carry what differs between blocks.

## Actual

Five control rows, then cards whose body is three constants.

## Fix

1. Merge rows 3–5 into one `flex items-center gap-2` row:
   `[Blocks|Templates tabs] [search, flex-1] [All Build Test Deploy Utility] [Import] [Create]`.
   Saves ~90px.
2. Delete the PageHeader (`BlocksPage.svelte:440-443`). The `/automation` tab bar already
   names the page; keep the description nowhere. Saves 44px.
3. Card body: drop `Type:` and `Version:` entirely (they already show as the coloured
   category badge and are constant). Keep name, description (one `line-clamp-1`), tags, and
   a parameter count. Edit / Export / Delete become `size="sm" variant="ghost"` icon buttons
   on the title row rather than a full-width button row. Target card height ~96px — four
   times as many blocks per screen.

## Evidence

`.claude/qa/shots/ux/ux-automation-blocks.png`,
`.claude/qa/shots/ux/ux-min1024-automation-blocks.png`.

## Repro

`node .claude/qa/scripts/app.mjs automation/blocks ux-automation-blocks`

## Fix

<!-- coder -->

## Verification

2026-11-07 — tauri, dev instance 2 (vite :1421 / CDP :9223, own database).
Ran `node .claude/qa/scripts/ux007-verify.mjs` and `qa088-verify2.mjs`.

| | before | 1280x800 now | 1024x600 now |
| --- | --- | --- | --- |
| chrome above the first card | 278px (5 rows) | **186px** | 226px |
| control rows between the section tabs and the grid | 4 | **1** (sub-tabs + search + 5 categories + Import + Create on one line) | 2 (wraps cleanly, Import/Create drop to a second line) |
| block card height | ~300px | **126px** | 126px |
| horizontal overflow | — | none | none |

Exercised on Blocks: category filters All 43 / Build 12 / Test 8 / Deploy 4 / Utility 19, and
search ("pytest" → 1 card). The no-match state names the query and offers a way out:
`No matching blocks — No block matches "zzzznope". The library holds 43 blocks. [Clear filters]`.
Delete is confirmed ("Delete block / Are you sure…? / Cancel / **Delete**") and toasts on success.
Card icon buttons now carry accessible names (`Delete block QA Imported Block`,
`Export template …`).

One new defect at the minimum window, filed as **QA-101**: block titles truncate to a single
letter at 1024 (`I...`, `R...`, `L...`). Import/export findings are under QA-088 and QA-100.

Shots: `.claude/qa/shots/UX-007-after-blocks-1280.png`,
`.claude/qa/shots/UX-007-after-blocks-1024x600.png`,
`.claude/qa/shots/UX-007-after-blocks-search-empty.png`.


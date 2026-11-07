---
id: QA-099
title: SDK sidebar header subtitle is cut in half on every /sdk page
status: open
severity: polish
kind: ux
area: sdk (SDKSidebar)
mode: tauri
found: 2026-11-07
---

## Use case

Read the SDK sidebar header — now a clickable "back to Portal" button rather than a static title.

## Steps

1. Any `/sdk/*` page at 1280x800 with the sidebar expanded.
2. Look at the second line of the header.

## Actual

"Manage your development environment" is sliced horizontally through the middle: the text box runs
to y=88 and its clipping ancestor (`min-w-0 flex-1 group-data-[collapsible=icon]:hidden`,
`overflow: hidden`) also ends at y=88, so the bottom half of the glyphs is cut off. Visible on
every SDK page.

```
{"text":"Manage your development environment","top":70,"bottom":88,
 "clipper":{"cls":"min-w-0 flex-1 group-data-[collapsible=icon]:hidden","bottom":88,"overflow":"hidden"}}
```

The button wrapper is sized for one line; the subtitle is the second.

## Expected

The subtitle fits, or is dropped.

## Evidence

`.claude/qa/shots/sdk-overview-loaded-1280.png`, `.claude/qa/shots/sdk-managers-1280.png`,
`.claude/qa/shots/sdk-sidebar-category-expanded.png` — same clipped line in all three.

## Repro

`node .claude/qa/scripts/sdk-sidebar.mjs` (prints the geometry), or just look at any `/sdk` page.

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

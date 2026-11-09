---
id: QA-099
title: SDK sidebar header subtitle is cut in half on every /sdk page
status: verified
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

Root cause: the header's `SidebarMenuButton size="sm"` has a fixed `h-7` (28px) height with
`overflow-hidden`, but the content is two lines. The same file already has the fix for this
exact shape one block down (`resolvedNavigationItems`'s `SidebarMenuButton` carries
`class="h-auto"`) — just missing on the header button. Added it there too.

File: `src/lib/domains/sdk/components/SDKSidebar.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-099.mjs` on `/sdk/python`: subtitle text
box is `top:78 bottom:96` (18px, fits one line) and its clipping ancestor's bottom is far below
that (1393, the full window) — no longer clipped. `.claude/qa/shots/QA-099-sidebar-header.png`
— "SDK Manager / Development Tools" both lines fully legible. Verified.

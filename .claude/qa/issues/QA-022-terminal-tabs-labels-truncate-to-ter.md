---
id: QA-022
title: With 6 terminal tabs every label truncates to "Ter..." and tabs do not scroll
status: verified
severity: minor
kind: ux
area: terminal
mode: tauri
found: 2026-11-06
---

## Use case

Work with several terminal tabs and switch between them.

## Steps

1. `/terminal`, press "+" five times (6 tabs).

## Expected

Tabs keep a readable minimum width and the strip scrolls, or tab titles reflect the cwd/command.

## Actual

All tabs squeeze to "Ter…" plus the close X; they cannot be told apart. Tabs also persist across reloads, so they accumulate.
Titles are always "Terminal N" (not cwd or last command).

## Evidence

`.claude/qa/shots/QA-021-before-xterm-ghost-text-3.png` shows the six squeezed tabs (top row).

## Repro

manual — steps above.

## Decision

2026-11-06, owner (via lead): keep restoring tabs across reloads (24h cleanup stays).
The accumulation part is by design, not a bug. Verify only the tab-strip scrolling, min width and tooltip.

## Fix

Fixed (UX): tabs now `min-w-28 shrink-0` and the strip scrolls horizontally (`min-w-0` on the scroll container); tooltip is "Title - cwd". Titles stay "Terminal N" (tab has no live cwd/title feed). Files: src/lib/domains/terminal/components/TabBar.svelte. Checks: pnpm check 0 errors, eslint 0 errors, prettier clean.
Needs decision (accumulation only): tabs are persisted to localStorage and restored by design (24h cleanupStaleData); processes are cleared on restore and each restored tab spawns a fresh shell when first shown, so they are not dead tabs, but they never go away unless closed. Whether to drop tabs on reload or keep restoring is a product call. With QA-021 fix, unvisited restored tabs no longer spawn shells.

## Verification

<!-- qa-tester -->

2026-11-06, tauri. Six tabs: labels readable ('Terminal 1'..'Terminal 5' visible, strip has a horizontal scrollbar, tab width ~160px). Tooltip shows the title only ('Terminal 1'; cwd not available, as the fix states). Tabs closed back to Terminal 1. Shot shots/QA-022-after-six-tabs.png.

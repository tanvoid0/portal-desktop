---
id: QA-101
title: Block card titles truncate to a single letter at the minimum window
status: verified
severity: major
kind: ux
area: automation/blocks
mode: tauri
found: 2026-11-07
---

## Use case

Find the block you want at the app's minimum window size (1024x600).

## Steps

1. `/automation/blocks` at 1024x600.
2. Read the block names.

## Expected

Enough of the name to tell the blocks apart.

## Actual

Every title collapses to one character plus an ellipsis — `I...`, `R...`, `L...` — while the
category badge and three icon buttons keep their full width beside it. Nine visible cards,
three distinguishable only by their description line (which is also truncated: "Install Node.js...",
"Install Python...", "Install Rust..."). At 1280x800 the same titles read "Install NPM D...",
"Lint with ESL..." — tight but usable; at 1024 they are not.

The title is in a flex row with `badge + 3 buttons` and loses the squeeze every time.

## Evidence

`.claude/qa/shots/UX-007-after-blocks-1024x600.png` (one letter per title),
`.claude/qa/shots/UX-007-after-blocks-1280.png` (same cards at 1280).

## Repro

`node .claude/qa/scripts/ux007-verify.mjs` — takes both shots.

## Fix

Root cause: `truncate`'s `overflow:hidden` makes the title's flex auto-min-width effectively 0,
so every bit of squeeze pressure (badge + 3 `shrink-0` icon buttons never give up space) lands
entirely on the title — at the 3-column breakpoint (`lg`, 1024px) that leaves it almost no
room. Bumped the grid's 3-column breakpoint from `lg` to `xl` (1280px) so 1024 renders 2 columns
(~490px/card, comfortable) instead of 3 (~330px/card); at 1280 it still goes 3-up, matching the
"tight but usable" state this issue already tolerated there. Applied to both grids sharing this
exact card shape (title + badge + up to 3 icon buttons).

Files: `src/lib/domains/automation/components/BlockList.svelte`,
`src/lib/domains/automation/components/TemplateList.svelte` (same squeeze risk, same fix).
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-101.mjs` at 1024x600 (CDP viewport
emulation, confirmed `window.innerWidth/innerHeight` actually change on the real app):
`BlockList` now renders 2 columns and every title is fully readable ("Install NPM
Dependencies", "Install Python Dependencies", "Run Cargo Tests", etc. — no one-letter
truncation). `.claude/qa/shots/QA-101-blocks-1024x600.png`. Verified.

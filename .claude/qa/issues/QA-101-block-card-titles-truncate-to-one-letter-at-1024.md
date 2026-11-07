---
id: QA-101
title: Block card titles truncate to a single letter at the minimum window
status: open
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

<!-- coder -->

## Verification

<!-- qa-tester -->

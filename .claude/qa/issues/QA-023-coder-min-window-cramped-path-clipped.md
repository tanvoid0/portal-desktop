---
id: QA-023
title: Coder page at 1024x600: composer buttons cut off, header wraps; workspace path select is clipped at any size
status: verified
severity: minor
kind: ux
area: ai/coder
mode: tauri
found: 2026-11-06
---

## Use case

Use the coder on a small laptop window.

## Steps

1. Open `/ai/coder` at 1024x600 (and at 1280x800 for the path select).

## Expected

Whole composer (input + send + mode chips) visible without scrolling; long workspace path truncated with an ellipsis inside its pane.

## Actual

- 1024x600: toolbar wraps to three rows ("Catalog error" drops under the provider), the empty-state heading wraps to 3 lines, and the
  composer is pushed to the bottom edge: the textarea starts at y~520 and its mode chips and send button are cut off by the 600px window.
- Any size: the workspace path select (`D:\projects\...\.claude\qa\fi...`) runs past the centre pane and is cut off by the
  right workspace panel, no ellipsis (select trigger is `flex-1` with `font-mono`, no `min-w-0`/truncate).
- Dark theme: the composer textarea has a lighter inner box than the card, and the red "Catalog error" text is low contrast.

## Evidence

`.claude/qa/shots/QA-023-before-coder-min-window.png`, `sweep-ai-coder-dark.png`.

## Repro

`node .claude/qa/scripts/QA-023.mjs`

## Fix

Root cause: unbounded toolbar/selector widths and tall empty state.
Fix: `CoderProjectSelector.svelte` min-w-0 so the path select truncates; `CoderPanel.svelte` header selector can shrink, Terminal/Commit labels icon-only below xl, empty-state icon/paragraph hidden and padding reduced when window height <= 700px, content scrolls; `AIComposerShell.svelte` textarea `dark:bg-transparent` (base Textarea's `dark:bg-input/30` made the lighter inner box). The red "Catalog error" is replaced (see QA-018 line) by a text-destructive message with a Settings link.
Nit: token-rejected text now `font-medium text-destructive dark:text-red-400`.
Checks: pnpm check 0 errors; eslint 0 errors, prettier clean on touched files. Not run in app (tester verifies).

## Verification

<!-- qa-tester -->

2026-11-06, tauri. 1024x600: path select truncates with chevron, composer fully visible, toolbar uses icon-only Terminal/Commit (shots/QA-023-after-coder-min-window.png). Dark: composer inner box fixed (shots/QA-023-after-coder-dark.png). Nit left: the red 'token rejected' text is still dim on the dark background.

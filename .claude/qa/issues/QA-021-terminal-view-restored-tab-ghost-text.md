---
id: QA-021
title: Terminal view of a restored tab shows ghost shell-path text on the prompt line and a black bar
status: verified
severity: minor
kind: bug
area: terminal
mode: tauri
found: 2026-11-06
---

## Use case

Use the interactive Terminal view (xterm) in a tab that survived a page reload.

## Steps

1. `/terminal`: have at least one extra tab (New tab), reload the page.
2. Switch the active tab to "Terminal view (interactive apps)", click in the terminal and type `echo QA-xterm`, Enter.

## Expected

`PS C:\Users\tanvo> echo QA-xterm` and clean output.

## Actual

Fragments of the shell path ("Shell\v1.0\powershell.exe", "C:\WINDOWS\System32") are left on the prompt line, interleaved with
the typed command; a solid black bar with a small arrow is drawn at the bottom of the terminal pane. A tab freshly created
in the same session (no reload) is clean, so it looks like a restore/replay or resize artefact. Reproduced 3 of 3 runs on restored tabs.

## Evidence

`.claude/qa/shots/QA-021-before-xterm-ghost-text-3.png` (also `sweep-terminal-xterm-typed.png`). No console errors.

## Repro

`node .claude/qa/scripts/QA-021.mjs` (needs an extra tab, run twice if the first output is clean).

## Fix

Root cause (inferred from code; not reproduced live, CDP off limits): TerminalWorkspace mounted a TerminalSession for every restored tab at once; background tabs are `display:none` (0x0), so xterm fit no-ops, the PTY spawns at 80x24, and on first show the resize makes ConPTY reflow/repaint the prompt line (title/path fragments, mis-sized rows = black bar). `savedOutput`/`restoreSession` are not involved (restoreSession is never called). Fix: mount a tab's session only on first activation, so xterm and PTY start at real size. Also stops spawning N shells on reload.
Files: src/lib/domains/terminal/components/TerminalWorkspace.svelte (`visited` + `{#if visited[tab.id]}`; prettier reindented the block).
SECOND ROOT CAUSE (ghost text, found statically): useXtermSession.handleOutput runs every PTY chunk through injectOsc8Links (utils/osc8.ts) which linkified paths inside the shell OSC window-title sequence (ESC]0;C:WINDOWSSystem32WindowsPowerShell1.0powershell.exe BEL); the BEL that osc8Wrap inserts ended the title early and the rest of the path printed as text. Fix: linkify now skips existing OSC sequences (osc8.ts oscRe). Added osc8.test.ts. Residual: an OSC split across the 256-char tail boundary is not protected. Checks: pnpm check 0 errors; eslint 0 errors; prettier clean; vitest osc8 passes. Repro not rerun (tester is on the app). Tester: reload with 2+ tabs, switch to a non-initial tab, Terminal view.

## Verification

<!-- qa-tester -->

2026-11-06, tauri. REOPENED. Fix not effective. Steps: /terminal, add 5 tabs, reload, click Terminal 4, Terminal view, click, type 'echo QA-xterm' + Enter. Terminal text: 'C:\WINDOWS\System32echo QA-xtermShell\v1.0\powershell.exePS C:\Users\tanvo> ec' - same ghost shell-path fragments as before (shots/QA-021-after-restored-tab-xterm.png). The black bar at the bottom is gone. Rerun: node .claude/qa/scripts/QA-021.mjs (needs another tab, run twice).

2026-11-06 (final round), tauri after rebuild. New 3 tabs, reload, select a non-initial tab, Terminal view, type: 'PS C:\Users\tanvo> echo QA-xterm / QA-xterm', no ghost text, no black bar; QA-021.mjs clean twice. shots/QA-021-after-final-restored-tab.png. Extra tabs closed.

---
id: QA-159
title: Agent tab with Files/Git panel open squeezes the chat column to ~220px at 1280x800
status: verified
severity: minor
kind: ux
area: ai/coder/claude-code (AgentsWorkspace)
mode: tauri
found: 2026-11-17
---

## Use case

Read the agent transcript and reply while browsing the session's files or git changes.

## Steps

1. Open `/ai/coder/claude-code?session=<id>` at 1280x800 (session list 320px + app nav visible).
2. Workspace sidebar is open by default; click Files (or Git changes).

## Expected

Chat stays usable next to the panel (or the panel/sidebar yields space).

## Actual

Three fixed columns (session list w-80, workspace panel, sidebar w-56) leave the chat ~220px wide: the Ended/Transcript/Timeline tabs are clipped ("nv" fragment of a tab cut off) and the composer wraps "Send a message to resume..." over three lines.

## Evidence

`.claude/qa/shots/sweep-agent-tab-files.png`

## Repro

`node .claude/qa/scripts/QA-870-verify.mjs` then view the shot.

## Fix

Root cause: four columns with no width budget - session list `w-80`, chat `flex-1`, workspace panel `flex-1`, sidebar `w-56` - so the two `flex-1` columns split what the fixed two left (1024 - 320 - 224 = 480 -> 240 each at 1280 with the 16rem app nav open). The window width is the wrong input (the nav eats 16rem of it), so the layout follows the pane's own width.

- `src/lib/domains/agents/components/AgentsWorkspace.svelte`: `bind:clientWidth` on the workspace root; the session list docks only while chat keeps 24rem beside the sidebar (14rem) and, with the panel open, an equal-width panel: pane >= 58rem, or >= 82rem (20+24+24+14) with the panel open (~1568px viewport, about 2xl). Below that it folds into the existing Sessions Sheet + toggle (the old `md:hidden` bar, now shown by the same condition). Chat column `min-w-0` -> `min-w-80` (20rem hard floor); the panel stays `min-w-0 flex-1`, so it is what gives way, never chat.
- `src/lib/components/shell/responsive-panel.svelte`: optional `collapsed` prop forces the Sheet form on a desktop viewport (default false; other callers unchanged).
- Measured, not `@container`/`@5xl:`: the fold is a mounted Sheet, not a style, so a class-only container query cannot pick it.
- Pane widths (nav open, 16rem), panel + sidebar open, session list / chat / panel / sidebar: 1280 folded/400/400/224 (was 320/240/240/224); 1440 folded/480/480/224 (was 320/320/320/224); 1920 and 2560 unchanged at 320/560/560/224 and 320/880/880/224. Panel closed: unchanged from 1184 up (1280: 320/480/-/224). Min window 1024: chat 320 (floor), panel 224.
- Not touched: `CoderPanel.svelte` (Portal tab) has the same fixed-column layout around the same `CoderWorkspacePanel`.
- Checks: `pnpm check` exit 0 (0 errors), `pnpm lint` exit 0 (prettier clean, eslint 0 errors, shadcn check passed). Not run in the live app - needs a live look at 1280x800.

## Verification

2026-11-20, tauri (dev instance). `scripts/QA-159-verify.mjs`: claude-code session focused, workspace sidebar + Files panel open. Chat column / composer width: 1280 -> 400 / 346, 1440 -> 480 / 426, 1920 -> 552 / 498 (was ~220 at 1280). Session list folds into the Sheet at 1280 and 1440 and docks (320) at 1920. Tabs not clipped, composer placeholder on one line at 1280. Shots: `.claude/qa/shots/QA-159-after-agent-files-1280.png`, `-1440.png`, `-1920.png`. Result: gone. (A "Vault locked" dialog overlays the dev instance in the shots; unrelated.)
